# Guest Program Examples

ZKM can generate proof for  Go and Rust (guest) Programs.

> [!NOTE]
> In the mips-elf directory, We have prepared the relative mips ELF which are ready for proof use.  


##1. Go program
The SDK provide Read and Commit interface to read input and commit output.
Take add-go for example:

* Build the add-go to mips ELF.

```
cd guest-program/add-go
GOOS=linux GOARCH=mips GOMIPS=softfloat go build  -o ../mips-elf/zkm-mips-elf-add-go .

```


##2. Rust program

* Download and install toolchain for mips

```
wget http://musl.cc/mips-linux-muslsf-cross.tgz
tar -zxvf mips-linux-muslsf-cross.tgz
```

* Modify ~/.cargo/config:

```
[target.mips-unknown-linux-musl]
linker = "<path-to>/mips-linux-muslsf-cross/bin/mips-linux-muslsf-gcc"
rustflags = ["--cfg", 'target_os="zkvm"',"-C", "target-feature=+crt-static", "-C", "link-arg=-g"]
```

* Build the Revme

```
cd guest-program/revme
cargo build --target=mips-unknown-linux-musl --release
```

You will see  zkm-mips-elf-revme-rust in target/mips-unknown-linux-musl/release/ .
