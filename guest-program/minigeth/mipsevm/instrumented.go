package main

import (
	"io"
	"math/rand"
)

type PreimageOracle interface {
	Hint(v []byte)
	GetPreimage(k [32]byte) []byte
}

type InstrumentedState struct {
	state *State

	stdOut io.Writer
	stdErr io.Writer

	lastMemAccess   uint32
	memProofEnabled bool
	memProof        [28 * 32]byte

	preimageOracle PreimageOracle

	// cached pre-image data, including 8 byte length prefix
	lastPreimage []byte
	// key for above preimage
	lastPreimageKey [32]byte
	// offset we last read from, or max uint32 if nothing is read this step
	lastPreimageOffset uint32

	blockroot     string
	debug         bool
	ignored_steps uint64
	ignored       bool
	saved_step    uint64
	memRoot       [32]uint8
}

const (
	fdStdin         = 0
	fdStdout        = 1
	fdStderr        = 2
	fdHintRead      = 3
	fdHintWrite     = 4
	fdPreimageRead  = 5
	fdPreimageWrite = 6
)

const (
	MipsEBADF  = 0x9
	MipsEINVAL = 0x16
)

func NewInstrumentedState(state *State, po PreimageOracle, stdOut, stdErr io.Writer) *InstrumentedState {
	return &InstrumentedState{
		state:          state,
		stdOut:         stdOut,
		stdErr:         stdErr,
		preimageOracle: po,
	}
}

func (m *InstrumentedState) SetBlockRoot(root string) {
	m.blockroot = root
}

func (m *InstrumentedState) SetDebug(debug bool) {
	m.debug = debug
}

func (m *InstrumentedState) getIgnoredStep() uint64 {
	return m.ignored_steps
}

func (m *InstrumentedState) Step(proof bool) (wit *StepWitness, err error) {
	m.memProofEnabled = proof
	m.lastMemAccess = ^uint32(0)
	m.lastPreimageOffset = ^uint32(0)

	if proof {
		insnProof := m.state.Memory.MerkleProof(m.state.PC)
		wit = &StepWitness{
			State:    m.state.EncodeWitness(),
			MemProof: insnProof[:],
		}
	}
	err = m.mipsStep()

	if err != nil {
		return nil, err
	}

	if proof {
		wit.MemProof = append(wit.MemProof, m.memProof[:]...)
		if m.lastPreimageOffset != ^uint32(0) {
			wit.PreimageOffset = m.lastPreimageOffset
			wit.PreimageKey = m.lastPreimageKey
			wit.PreimageValue = m.lastPreimage
		}
	}
	return
}

func (m *InstrumentedState) InitialMemRoot() {
	m.memRoot = m.state.Memory.MerkleRoot()
}

func (m *InstrumentedState) StepTrace(rate int) (wit *trace, err error) {
	m.lastMemAccess = ^uint32(0)
	m.lastPreimageOffset = ^uint32(0)

	curState := &traceState{
		PC:        m.state.PC,
		NextPC:    m.state.NextPC,
		HI:        m.state.HI,
		LO:        m.state.LO,
		Heap:      m.state.Heap,
		Registers: m.state.Registers,
		ExitCode:  m.state.ExitCode,
		Exited:    m.state.Exited,
		Step:      m.state.Step,
	}

	wit = &trace{
		curState: curState,
	}

	wit.curState.MemRoot = m.memRoot

	wit.Insn_proof = m.state.Memory.MerkleProof(m.state.PC)
	m.memProofEnabled = true

	err = m.mipsStep()

	if err != nil {
		return nil, err
	}

	m.memRoot = m.state.Memory.MerkleRoot()
	wit.Memory_proof = m.memProof
	wit.nextState = &traceState{
		PC:        m.state.PC,
		NextPC:    m.state.NextPC,
		HI:        m.state.HI,
		LO:        m.state.LO,
		Heap:      m.state.Heap,
		Registers: m.state.Registers,
		ExitCode:  m.state.ExitCode,
		Exited:    m.state.Exited,
		Step:      m.state.Step,
	}

	wit.nextState.MemRoot = m.memRoot
	if rand.Intn(100000) < rate {
		wit.insertToDB()
	}
	return
}

func (m *InstrumentedState) IsExited() bool {
	return m.state.Exited
}

func (m *InstrumentedState) GetPC() uint32 {
	return m.state.PC
}
