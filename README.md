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

Install cargo tarpaulin
`cargo install cargo-tarpaulin`

### Run coverage tests
`make coverage`

### Add lint component
`rustup component add clippy`

`rustup component add rustfmt`

`cargo install cargo-audit`

### Add SQLX Utilities
`cargo install sqlx-cli --no-default-features --features postgres,runtime-actix-native-tls`


### For local development, add a .env file 
```
DATABASE_URL="postgres://postgres:password@localhost:5432/amrit_cms"
```