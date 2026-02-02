export BUILD_TYPE		:= release
export DEFAULT_BUILD	:= riscv64-sbi

export ALL_BUILDS		:= riscv64-sbi loongarch64-naked

export ELF_FILE			:= bin/$(BUILD_TYPE)/$(BUILD_NAME).elf
export USER_DIR 		:= bin/$(BUILD_TYPE)/user