package main

import (
	"bytes"
	"debug/elf"
	"flag"
	"fmt"
	"io"
	"os"
	"time"
)

var (
	h          bool
	block      string
	program    string
	totalsteps uint
	rate       int
	debug      bool
)

func usage() {
	fmt.Fprintf(os.Stderr, `
Usage: [-h] [-b blocknum] [-e elf-path] [-s stepnum] [-r rate] [-d]

Options:
`)
	flag.PrintDefaults()
}

func init() {
	flag.BoolVar(&h, "h", false, "help info")

	flag.StringVar(&block, "b", "", "blocknum for minigeth")
	flag.StringVar(&program, "e", "", "MIPS program elf path(default minigeth when blocknum is specified)")
	flag.UintVar(&totalsteps, "s", 0xFFFFFFFF, "program steps number to be run (default 4294967295)")
	flag.IntVar(&rate, "r", 100000, "randomly generate trace rate (1/100000)")
	flag.BoolVar(&debug, "d", false, "enable debug output for the instrution sequences")

	// 改变默认的 Usage
	flag.Usage = usage
}

var block_root string

func start_elf(path string) {
	elfProgram, err := elf.Open(path)

	state, err := LoadELF(elfProgram)

	if err != nil {
		fmt.Println(err)
		return
	}

	err = PatchGo(elfProgram, state)

	if err != nil {
		fmt.Println(err)
		return
	}

	err = PatchStack(state)

	if err != nil {
		fmt.Println(err)
		return
	}

	if block != "" {
		basedir := os.Getenv("BASEDIR")
		if len(basedir) == 0 {
			basedir = "/tmp/cannon"
		}
		block_root = fmt.Sprintf("%s/0_%s", basedir, block)
		block_input := fmt.Sprintf("%s/input", block_root)
		state, err = LoadMappedFile(state, block_input, 0x30000000)
		if err != nil {
			fmt.Println(err)
			return
		}
	}

	var stdOutBuf, stdErrBuf bytes.Buffer
	goState := NewInstrumentedState(state, nil, io.MultiWriter(&stdOutBuf, os.Stdout), io.MultiWriter(&stdErrBuf, os.Stderr))

	goState.SetBlockRoot(block_root)
	goState.InitialMemRoot()
	goState.SetDebug(debug)

	err = InitDB()

	if err != nil {
		fmt.Println(err)
		return
	}

	start := time.Now()
	step := uint(0)
	for !goState.IsExited() {

		_, err = goState.StepTrace(rate)

		if err != nil {
			fmt.Println(err)
			return
		}

		step++
		if step >= totalsteps {
			break
		}

	}

	fmt.Println("Can ignore ", goState.getIgnoredStep(), " instructions")
	end := time.Now()
	delta := end.Sub(start)
	fmt.Println("test took", delta, ",", state.Step, "instructions, ", delta/time.Duration(state.Step), "per instruction")
}

func start_minigeth() {
	start_elf("minigeth")
}

func main() {

	flag.Parse()

	if h {
		flag.Usage()
	} else if block != "" {
		start_minigeth()
	} else if program != "" {
		start_elf(program)
	}
}
