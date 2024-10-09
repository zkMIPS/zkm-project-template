# Guest Program Examples

ZKM can generate proof for  Go and Rust (guest) Programs.

* sha2-go
  
A simple program that takes struct Data   as input, and operate the elements  as an output.

* sha2-rust
It takes a public input and a private input ,then check the hash(private input)= public input.

* mem-allox-vec

> [!NOTE]
> If you want to compile the guest programs, you should use a x86 Ubuntu22 machine with Rust: 1.81.0-nightly and Go : 1.22.1

* Install the mips-rust tool(the cargo should be ~/.cargo).

```
cd zkm-project-template
chmod +x install_mips_rust_tool
./install_mips_rust_tool
```

* Compile the go guest program
 
```
cd zkm-project-template/guest-program/sha2-go
GOOS=linux GOARCH=mips GOMIPS=softfloat go build  -o sha2-go
```
The compiled mips ELF is in the current path.

* Compile the rust guest program
  
```
cd zkm-project-template/guest-program/sha2-rust
cargo build --target=mips-unknown-linux-musl --release
```

or
```
cd zkm-project-template/guest-program/mem-alloc-vec
cargo build --target=mips-unknown-linux-musl --release
```

The compiled mips ELF is in the zkm-project-template/guest-program/revme/target/mips-unknown-linux-musl/release/ .

