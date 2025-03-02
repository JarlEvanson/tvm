set -e

cargo build --package tvm_loader-x86_64-uefi --target x86_64-unknown-uefi --release
cargo build --package tvm-x86_64-generic --target x86_64-unknown-none --release

# Boot partition setup
mkdir -p run/x86_64-uefi/boot-partition/
cp target/x86_64-unknown-uefi/release/tvm_loader-x86_64-uefi.efi run/x86_64-uefi/boot-partition/tvm
llvm-objcopy \
    --add-section TVM_BIN=target/x86_64-unknown-none/release/tvm-x86_64-generic \
    --set-section-flags TVM_BIN=alloc,readonly,data \
    run/x86_64-uefi/boot-partition/tvm


# Command setup/execution
cp /home/jarl/projects/ovmf-binaries/x64/vars.fd run/x86_64-uefi/vars.fd

qemu-system-x86_64 \
    -machine q35 -cpu max,la57=on -m 20G \
    -drive if=pflash,format=raw,readonly=on,file=/home/jarl/projects/ovmf-binaries/x64/code.fd \
    -drive if=pflash,format=raw,readonly=off,file=run/x86_64-uefi/vars.fd \
    -drive format=raw,file=fat:rw:run/x86_64-uefi/boot-partition \
    -net none \
    -serial file:run/x86_64-uefi/serial.txt \
    -D run/x86_64-uefi/qemu_log.txt \
    -d int \
    -s
