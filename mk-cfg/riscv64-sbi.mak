export QEMU		:= qemu-system-riscv64
export TARGET	:= riscv64gc-unknown-none-elf
export FEATURES := naked

# QEMU

BIOS			:= default
MACHINE			:= virt
include mk-cfg/run.mak 
export QEMU_ARGS	:= 	-machine $(MACHINE),memory-backend=mem0 \
						-kernel "$(ELF_FILE)" \
						-smp $(CPU_INFO) \
						-m $(MEM_SIZE) \
						-bios $(BIOS) \
						-display gtk -monitor stdio \
						-object memory-backend-ram,id=mem0,size=$(MEM_SIZE),share=on,prealloc=off \
						$(EXTRA_QEMU_ARGS)

pre_run:
pre_build: