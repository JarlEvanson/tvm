set -e

cargo build --package tvm_loader-x86-uefi --target i686-unknown-uefi --release
cargo build -Zbuild-std=core --package tvm-x86-generic --target target_specs/i686-unknown-none.json --release

# Boot partition setup
mkdir -p run/x86-uefi/boot-partition/
cp target/i686-unknown-uefi/release/tvm_loader-x86-uefi.efi run/x86-uefi/boot-partition/tvm
llvm-objcopy \
    --add-section TVM_BIN=target/i686-unknown-none/release/tvm-x86-generic \
    --set-section-flags TVM_BIN=alloc,readonly,data \
    run/x86-uefi/boot-partition/tvm

# Command setup/execution
cp /home/jarl/projects/ovmf-binaries/ia32/vars.fd run/x86-uefi/vars.fd

qemu-system-x86_64 \
    -machine q35 -cpu max,la57=on -m 2G \
    -drive if=pflash,format=raw,readonly=on,file=/home/jarl/projects/ovmf-binaries/ia32/code.fd \
    -drive if=pflash,format=raw,readonly=off,file=run/x86-uefi/vars.fd \
    -drive format=raw,file=fat:rw:run/x86-uefi/boot-partition \
    -serial file:run/x86-uefi/serial.txt \
    -D run/x86-uefi/qemu_log.txt \
    -d int \
    -s
