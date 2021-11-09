
#! bin/bash

MNT=./mnt
DISK=./disk.img
BOOTLOADER_TARGET=$1
KERNEL_TARGET=$2

# make_image 部分
qemu-img create -f raw $DISK 200M
mkfs.fat -n 'POTATO OS' -s 2 -f 2 -R 32 -F 32 $DISK

# mount_image 部分
mkdir -p $MNT
sudo mount -o loop $DISK $MNT

sudo mkdir -p $MNT/EFI/BOOT

# build bootloader
cd potato_loader && cargo build 
cd ../

sudo cp $BOOTLOADER_TARGET $MNT/EFI/BOOT/BOOTX64.EFI
sudo cp $KERNEL_TARGET $MNT/potatOS.elf

sleep 0.5
sudo umount $MNT

# run_image 部分
qemu-system-x86_64 -bios OVMF.fd -drive format=raw,file=$DISK -s -S &
rust-gdb target/kernel_target/debug/potatOS.elf -ex "target remote :1234"
