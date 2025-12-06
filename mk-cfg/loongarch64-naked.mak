export QEMU			:= qemu-system-loongarch64
export TARGET		:= loongarch64-unknown-none
export FEATURES 	:= naked
export DTB_FILE		:= $(RUNTIME)/qemu-loongarch64.dtb
export DTS_FILE		:= $(RUNTIME)/qemu-loongarch64.dts

# QEMU

BIOS			:= $(CUR)/bootloader/qemu-efi-loongarch.fd
MACHINE			:= virt
include mk-cfg/run.mak 
export QEMU_ARGS	:= 	-machine $(MACHINE) \
						-kernel "$(ELF_FILE)" \
						-smp $(CPU_INFO) \
						-m $(MEM_SIZE) \
						-display gtk -monitor stdio

pre_run:

pre_build:
	@rm -rf $(RUNTIME)
	@mkdir -p $(RUNTIME)
	
	@echo Exporting DTB
	$(QEMU) -machine $(MACHINE),dumpdtb=$(DTB_FILE) \
			-smp $(CPU_INFO) \
			-m $(MEM_SIZE)
	dtc -I dtb $(DTB_FILE) -O dts -o $(DTS_FILE)