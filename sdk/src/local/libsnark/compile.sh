#!/bin/bash

cd "$(dirname "$0")"

# Determine the operating system
OS="$(uname)"

case "$OS" in
    Linux)
        echo "Running on Linux"
        # Compile for Linux
        go build -o libsnark.so -buildmode=c-shared *.go
        ;;
    Darwin)
        echo "Running on macOS"
        # Compile for macOS
        go build -o libsnark.dylib -buildmode=c-shared *.go
        ;;
    *)
        echo "Unsupported OS: $OS"
        exit 1
        ;;
esac
