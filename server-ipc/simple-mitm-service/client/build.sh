RUST_TARGET_PATH=$PWD RUSTFLAGS="-Z macro-backtrace" xargo build --release --target aarch64-none-elf
DIR=`basename "$PWD"`
linkle nro target/aarch64-none-elf/release/$DIR.elf target/aarch64-none-elf/release/$DIR.nro --nacp-path=nacp.json