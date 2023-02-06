## Local Setup

### Prerequisites for build tools

On Windows: 
```
cargo install -f cargo-binutils
rustup component add llvm-tools-preview
```

On Linux:
 - Ubuntu, `sudo apt-get install lld clang`
 - Arch, `sudo pacman -S lld clang`

On MacOS, `brew install michaeleisel/zld/zld`

### Development tools

Install cargo watch:
`cargo install cargo-watch`
