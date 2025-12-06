export QEMU		:= qemu-system-riscv64
export TARGET	:= riscv64gc-unknown-none-elf
export FEATURES := naked

# QEMU

BIOS			:= $(CUR)/bootloader/rustsbi-qemu.bin
MACHINE			:= virt
include mk-cfg/run.mak 
export QEMU_ARGS	:= 	-machine $(MACHINE) \
						-kernel "$(ELF_FILE)" \
						-smp $(CPU_INFO) \
						-m $(MEM_SIZE) \
						-bios $(BIOS) \
						-display gtk -monitor stdio

pre_run:
pre_build: