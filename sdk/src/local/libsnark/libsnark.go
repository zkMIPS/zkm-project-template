package main

import (
	"C"
)
import "fmt"

//export Stark2Snark
func Stark2Snark(inputdir *C.char, outputdir *C.char) C.int {
	// Convert C strings to Go strings
	inputDir := C.GoString(inputdir)
	outputDir := C.GoString(outputdir)
	var prover SnarkProver
	err := prover.Prove(inputDir, outputDir)
	if err != nil {
		fmt.Printf("Stark2Snark error: %v\n", err)
		return -1
	}
	return 0
}

func main() {}
