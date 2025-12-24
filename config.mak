export BUILD_TYPE		:= debug
export DEFAULT_BUILD	:= loongarch64-naked

export ALL_BUILDS		:= riscv64-sbi loongarch64-naked

export ELF_FILE			:= bin/$(BUILD_TYPE)/$(BUILD_NAME).elf