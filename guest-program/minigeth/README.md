<a>English</a>    <a href='./README_CN.md' >中文</a> 

The cannon-mips is a MIPS VM (ported from cannon project) to generate execution records for MIPS programs.

## Directory Layout

```
minigeth -- A standalone "geth" capable of computing a block transition
mipsevm -- A MIPS VM to generate execution records for MIPS program
```

## Prerequisite

Reference [prerequisite of mips_circuit](https://github.com/zkMIPS/mips_circuit#prerequisite)

## Build


```
make build
```

## Generate execution records

The following commands should be run from the root directory unless otherwise specified:

```
#Example: compute the transition from 13284469 -> 13284470 on PC
$ export BASEDIR=/tmp/cannon

$ export NODE=<mainnet-node> # default: https://mainnet.infura.io/v3/9aa3d95b3bc440fa88ea12eaa4456161

$ mkdir -p /tmp/cannon
$ minigeth/go-ethereum 13284469

$ export POSTGRES_CONFIG="sslmode=<sslmode> user=<user> password=<password> host=<ip> port=<port> dbname=<db>"
   # default: sslmode=disable user=postgres password=postgres host=localhost port=5432 dbname=postgres

# generate 1 MIPS trace
$ cd mipsevm
$ ./mipsevm -b 13284469 -s 1
```

**NOTE: You may cannot access the default NODE URL and trigger nil pointer dereference fault when running "minigeth/go-ethereum" ,   you can register one free NODE from https://www.alchemy.com/ to replace it**

## Options for mipsevm

Command: 

```
mipsevm [-h] [-b blocknum] [-e elf-path] [-s stepnum] [-r rate] [-d]
```

Options:

```
  -h             help info
  -b <blocknum>  blocknum for minigeth
  -e <elf-path>  MIPS program elf path(default minigeth when blocknum is specified)
  -s <stepnum>   program steps number to be run (default 4294967295)
  -r <rate>      randomly generate trace rate (1/100000) (default 100000)
  -d             enable debug output for the instrution sequences
```

Example:

- Generate records for the first 1000 instructions of block 13284469

```
./mipsevm -b 13284469 -s 1000   //[-r 100000] and [-e minigeth] can be used as default
```

- Generate records with 1% rate for the first 1000 instructions of block 13284469

```
./mipsevm -b 13284469 -s 1000 -r 1000
```

## License

Most of this code is MIT licensed, minigeth is LGPL3.

Note: This code is unaudited. It in NO WAY should be used to secure any money until a lot more
testing and auditing are done. I have deployed this nowhere, have advised against deploying it, and
make no guarantees of security of ANY KIND.
