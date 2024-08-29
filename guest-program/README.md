# Guest Program Examples

ZKM can generate proof for  Go and Rust (guest) Programs.

> [!NOTE]
> In the mips-elf directory, We have prepared the relative mips ELF which are ready for proof use.  

We will provide users with Go and Rust tools to facilitate the building of guest programs. 
But you should use the zkm repo to build the Go/Rust guest programs at present. Please refer to  https://github.com/zkMIPS/zkm/blob/main/prover/examples/README.md 
 
```
cd prover/examples/add-go
GOOS=linux GOARCH=mips GOMIPS=softfloat go build .
```
or
```
cargo build --target=mips-unknown-linux-musl
```


