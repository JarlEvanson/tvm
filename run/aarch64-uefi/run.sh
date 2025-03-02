set -e

cargo build --package tvm_loader-aarch64-uefi --target aarch64-unknown-uefi

# Boot partition setup
mkdir -p run/aarch64-uefi/boot-partition/
cp target/aarch64-unknown-uefi/debug/tvm_loader-aarch64-uefi.efi run/aarch64-uefi/boot-partition/tvm

# Command setup/execution
cp /home/jarl/projects/ovmf-binaries/aarch64/vars.fd run/aarch64-uefi/vars.fd

qemu-system-aarch64 \
    -drive if=pflash,format=raw,readonly=on,file=/home/jarl/projects/ovmf-binaries/aarch64/code.fd \
    -drive if=pflash,format=raw,readonly=off,file=run/aarch64-uefi/vars.fd \
    -drive format=raw,file=fat:rw:run/aarch64-uefi/boot-partition \
    -device virtio-gpu-pci \
    -device qemu-xhci \
    -device usb-kbd \
    -serial file:run/aarch64-uefi/serial.txt \
    -D run/aarch64-uefi/qemu_log.txt \
    -d int \
    -machine virt \
    -cpu max
