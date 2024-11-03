package main

import (
	"database/sql"
	"database/sql/driver"
	"encoding/binary"
	"errors"
	"log"
	"os"
	"strconv"

	_ "github.com/lib/pq"

	"encoding/json"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/hexutil"
)

type State struct {
	Memory *Memory `json:"memory"`

	PreimageKey    common.Hash `json:"preimageKey"`
	PreimageOffset uint32      `json:"preimageOffset"` // note that the offset includes the 8-byte length prefix

	PC     uint32 `json:"pc"`
	NextPC uint32 `json:"nextPC"`
	LO     uint32 `json:"lo"`
	HI     uint32 `json:"hi"`
	Heap   uint32 `json:"heap"` // to handle mmap growth

	ExitCode uint8 `json:"exit"`
	Exited   bool  `json:"exited"`

	Step uint64 `json:"step"`

	Registers [32]uint32 `json:"registers"`

	// LastHint is optional metadata, and not part of the VM state itself.
	// It is used to remember the last pre-image hint,
	// so a VM can start from any state without fetching prior pre-images,
	// and instead just repeat the last hint on setup,
	// to make sure pre-image requests can be served.
	// The first 4 bytes are a uin32 length prefix.
	// Warning: the hint MAY NOT BE COMPLETE. I.e. this is buffered,
	// and should only be read when len(LastHint) > 4 && uint32(LastHint[:4]) >= len(LastHint[4:])
	LastHint hexutil.Bytes `json:"lastHint,omitempty"`
}

type traceState struct {
	Step   uint64 `json:"cycle"`
	PC     uint32 `json:"pc"`
	NextPC uint32 `json:"nextPC"`

	LO uint32 `json:"lo"`
	HI uint32 `json:"hi"`

	Registers [32]uint32 `json:"regs"`

	Heap uint32 `json:"heap"` // to handle mmap growth

	ExitCode uint8     `json:"exitCode"`
	Exited   bool      `json:"exited"`
	MemRoot  [32]uint8 `json:"memRoot"`
}

type trace struct {
	curState *traceState `json:"cur_state"`

	Insn_proof   [28 * 32]uint8 `json:"insn_proof"`
	Memory_proof [28 * 32]uint8 `json:"mem_proof"`
	nextState    *traceState    `json:"next_state"`
}

type traceJson struct {
	Step   string `json:"cycle"`
	PC     string `json:"pc"`
	NextPC string `json:"nextPC"`

	LO string `json:"lo"`
	HI string `json:"hi"`

	Registers [32]string `json:"regs"`

	Heap string `json:"heap"` // to handle mmap growth

	ExitCode     string          `json:"exitCode"`
	Exited       bool            `json:"exited"`
	MemRoot      [32]string      `json:"memRoot"`
	Insn_proof   [28 * 32]string `json:"insn_proof"`
	Memory_proof [28 * 32]string `json:"mem_proof"`

	NewStep   string `json:"newCycle"`
	NewPC     string `json:"newPc"`
	NewNextPC string `json:"newNextPC"`

	NewLO string `json:"newLo"`
	NewHI string `json:"newHi"`

	NewRegisters [32]string `json:"newRegs"`

	NewHeap string `json:"newHeap"` // to handle mmap growth

	NewExitCode string     `json:"newExitCode"`
	NewExited   bool       `json:"newExited"`
	NewMemRoot  [32]string `json:"newMemRoot"`
}

var (
	DB *sql.DB
)

func (a traceJson) Value() (driver.Value, error) {
	return json.Marshal(a)
}

// Make the Attrs struct implement the sql.Scanner interface. This method
// simply decodes a JSON-encoded value into the struct fields.
func (a *traceJson) Scan(value interface{}) error {
	b, ok := value.([]byte)
	if !ok {
		return errors.New("type assertion to []byte failed")
	}

	return json.Unmarshal(b, &a)
}

func InitDB() (err error) {
	postconfig := os.Getenv("POSTGRES_CONFIG")

	if len(postconfig) == 0 {
		postconfig = "sslmode=disable user=postgres password=postgres host=localhost port=5432 dbname=postgres"
	}

	db, err := sql.Open("postgres", postconfig)
	if err != nil {
		return err
	}

	_, err = db.Exec("TRUNCATE f_traces")

	if err != nil {
		return err
	}

	DB = db
	return nil
}

func (s *trace) insertToDB() {
	json := &traceJson{
		Step:   strconv.FormatUint(uint64(s.curState.Step), 10),
		PC:     strconv.FormatUint(uint64(s.curState.PC), 10),
		NextPC: strconv.FormatUint(uint64(s.curState.NextPC), 10),

		LO:   strconv.FormatUint(uint64(s.curState.LO), 10),
		HI:   strconv.FormatUint(uint64(s.curState.HI), 10),
		Heap: strconv.FormatUint(uint64(s.curState.Heap), 10),

		ExitCode: strconv.FormatUint(uint64(s.curState.ExitCode), 10),
		Exited:   s.curState.Exited,

		NewStep:   strconv.FormatUint(uint64(s.nextState.Step), 10),
		NewPC:     strconv.FormatUint(uint64(s.nextState.PC), 10),
		NewNextPC: strconv.FormatUint(uint64(s.nextState.NextPC), 10),

		NewLO:   strconv.FormatUint(uint64(s.nextState.LO), 10),
		NewHI:   strconv.FormatUint(uint64(s.nextState.HI), 10),
		NewHeap: strconv.FormatUint(uint64(s.nextState.Heap), 10),

		NewExitCode: strconv.FormatUint(uint64(s.nextState.ExitCode), 10),
		NewExited:   s.nextState.Exited,
	}

	for i := int(0); i < 32; i++ {
		json.MemRoot[i] = strconv.FormatUint(uint64(s.curState.MemRoot[i]), 10)
		json.Registers[i] = strconv.FormatUint(uint64(s.curState.Registers[i]), 10)
		json.NewMemRoot[i] = strconv.FormatUint(uint64(s.nextState.MemRoot[i]), 10)
		json.NewRegisters[i] = strconv.FormatUint(uint64(s.nextState.Registers[i]), 10)
	}

	for i := int(0); i < 32*28; i++ {
		json.Insn_proof[i] = strconv.FormatUint(uint64(s.Insn_proof[i]), 10)
		json.Memory_proof[i] = strconv.FormatUint(uint64(s.Memory_proof[i]), 10)
	}

	_, err := DB.Exec("INSERT INTO f_traces (f_trace) VALUES($1)", json)

	if err != nil {
		log.Fatal(err)
	}
}

func (s *State) EncodeWitness() []byte {
	out := make([]byte, 0)
	memRoot := s.Memory.MerkleRoot()
	out = append(out, memRoot[:]...)
	out = append(out, s.PreimageKey[:]...)
	out = binary.BigEndian.AppendUint32(out, s.PreimageOffset)
	out = binary.BigEndian.AppendUint32(out, s.PC)
	out = binary.BigEndian.AppendUint32(out, s.NextPC)
	out = binary.BigEndian.AppendUint32(out, s.LO)
	out = binary.BigEndian.AppendUint32(out, s.HI)
	out = binary.BigEndian.AppendUint32(out, s.Heap)
	out = append(out, s.ExitCode)
	if s.Exited {
		out = append(out, 1)
	} else {
		out = append(out, 0)
	}
	out = binary.BigEndian.AppendUint64(out, s.Step)
	for _, r := range s.Registers {
		out = binary.BigEndian.AppendUint32(out, r)
	}
	return out
}
