set -e

cargo build --package tvm_loader-aarch64-limine --target aarch64-unknown-none

# Boot partition setup
mkdir -p run/aarch64-limine/boot-partition/EFI/BOOT
cp target/aarch64-unknown-none/debug/tvm_loader-aarch64-limine run/aarch64-limine/boot-partition/tvm_loader

## Limine configuration
cp ~/projects/limine-binaries/BOOTAA64.EFI run/aarch64-limine/boot-partition/EFI/BOOT/
cat > run/aarch64-limine/boot-partition/limine.conf <<- EOM
serial: yes

timeout: 0

/tvm
kernel_path: boot():/tvm_loader
protocol: limine
EOM

# Command setup/execution
cp /home/jarl/projects/ovmf-binaries/aarch64/vars.fd run/aarch64-limine/vars.fd

qemu-system-aarch64 \
    -drive if=pflash,format=raw,readonly=on,file=/home/jarl/projects/ovmf-binaries/aarch64/code.fd \
    -drive if=pflash,format=raw,readonly=off,file=run/aarch64-limine/vars.fd \
    -drive format=raw,file=fat:rw:run/aarch64-limine/boot-partition \
    -device virtio-gpu-pci \
    -device qemu-xhci \
    -device usb-kbd \
    -serial file:run/aarch64-limine/serial.txt \
    -D run/aarch64-limine/qemu_log.txt \
    -d int \
    -machine virt \
    -cpu max
