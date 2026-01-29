include config.mak
include mk-cfg/$(BUILD_NAME).mak

export MIN_QEMU_VERSION:= 9.0.50		# Older QEMU Versions has FDT bugs on loongarch64

build:
	@echo Bulding "$(BUILD_NAME)"...
	make -C user build
	mkdir -p $(RUNTIME)/user
	cp $(CUR)/$(USER_DIR)/* $(RUNTIME)/user
	make -f mk-cfg/$(BUILD_NAME).mak pre_build
	make -C os all	

run:
	@make build
	@make run_only

run_only:
	make -f mk-cfg/$(BUILD_NAME).mak pre_run
	@echo Checking Qemu Version
	bash scripts/version_check.sh $(MIN_QEMU_VERSION) $(shell $(QEMU) --version | grep version | awk '{print $$4}')
	@echo Launching qemu
	$(QEMU) $(QEMU_ARGS)

exec:
	@echo $(CUR)/$(ELF_FILE)

.PHONY: build run run_only exec