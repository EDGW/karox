include config.mak
include mk-cfg/$(BUILD_NAME).mak

export MIN_QEMU_VERSION:= 9.0.50		# Older QEMU Versions has FDT bugs on loongarch64

build:
	@echo Bulding "$(BUILD_NAME)"...
	make -f mk-cfg/$(BUILD_NAME).mak pre_build
	make -C os all

run: build

	make -f mk-cfg/$(BUILD_NAME).mak pre_run
	@echo Checking Qemu Version
	dpkg --compare-versions $(shell $(QEMU) --version | grep version | awk '{print $$4}') gt $(MIN_QEMU_VERSION)
	@echo Launching qemu
	$(QEMU) $(QEMU_ARGS)

.PHONY: build run