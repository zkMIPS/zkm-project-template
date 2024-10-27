# ZKM SDK usage

## Use the libsnark

1. The  compile.sh in the path sdk/src/local/libsnark only supports X86_64 linux.
   
```
cd zkm-project-template/sdk/src/local/libsnark
./compile.sh
```
    If successful, it will generate the libsnark.so in sdk/src/local/libsnark/

2. To instruct your Rust environment on the location of the libsnark.so , you can set the LD_LIBRARY_PATH environment variable. For example:

```
export LD_LIBRARY_PATH=Your BASEDIR/zkm-project-template/sdk/src/local/libsnark:$LD_LIBRARY_PATH  
```

3. Import the SDK
   
```
// Cargo.toml
[dependencies]
zkm-sdk = { git = "https://github.com/zkMIPS/zkm-project-template", branch = "main", features = ["snark"] }
```

## Don't use the libsnark

1. Set the environment variable `NO_USE_SNARK=true` .
  
2. Import the SDK
   
```
// Cargo.toml
[dependencies]
zkm-sdk = { git = "https://github.com/zkMIPS/zkm-project-template", branch = "main" }
```
