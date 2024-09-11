# Guest Program Examples

ZKM can generate proof for  Go and Rust (guest) Programs.

> [!NOTE]
> In the mips-elf directory, we have prepared the relative mips ELF which are ready for proof use.  
> If you want to compile the guest programs, you should use a x86 Ubuntu22 machine with Rust: 1.81.0-nightly and Go : 1.22.1

* Install the mips-rust tool(the cargo should be ~/.cargo).

```
cd zkm-project-template
chmod +x install_mips_rust_tool
./install_mips_rust_tool
```

* Compile the go guest program
 
```
cd zkm-project-template/guest-program/add-go
GOOS=linux GOARCH=mips GOMIPS=softfloat go build .
```

* Compile the rust guest program
  
```
cd zkm-project-template/guest-program/revme
cargo build --target=mips-unknown-linux-musl --release
```
The compiled mips ELF is in the zkm-project-template/guest-program/revme/target/mips-unknown-linux-musl/release/zkm-mips-elf-revme-rust

