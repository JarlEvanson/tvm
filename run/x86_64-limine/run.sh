set -e

cargo build --package tvm_loader-x86_64-limine --target x86_64-unknown-none

# Boot partition setup
mkdir -p run/x86_64-limine/boot-partition/EFI/BOOT
cp target/x86_64-unknown-none/debug/tvm_loader-x86_64-limine run/x86_64-limine/boot-partition/tvm_loader

## Limine configuration
cp ~/projects/limine-binaries/BOOTX64.EFI run/x86_64-limine/boot-partition/EFI/BOOT/
cat > run/x86_64-limine/boot-partition/limine.conf <<- EOM
serial: yes

timeout: 0

/tvm
kernel_path: boot():/tvm_loader
protocol: limine
EOM

# Command setup/execution
cp /home/jarl/projects/ovmf-binaries/x64/vars.fd run/x86_64-limine/vars.fd

qemu-system-x86_64 \
    -drive if=pflash,format=raw,readonly=on,file=/home/jarl/projects/ovmf-binaries/x64/code.fd \
    -drive if=pflash,format=raw,readonly=on,file=run/x86_64-limine/vars.fd \
    -drive format=raw,file=fat:rw:run/x86_64-limine/boot-partition \
    -serial file:run/x86_64-limine/serial.txt \
    -D run/x86_64-limine/qemu_log.txt \
    -d int \
    -machine q35 \
    -cpu max
