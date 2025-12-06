BUILD_FLAGS_debug	:= 
BUILD_FLAGS_release	:= --release

OUTPUT				:= $(CUR)/target/$(TARGET)/$(BUILD_TYPE)/os

all: 
	@make build
	@make copy

build:
	@echo Building kernel on architecture "$(BUILD_NAME)"
	cargo build $(BUILD_FLAGS_$(BUILD_TYPE)) --target $(TARGET) --features $(FEATURES) --no-default-features
	
FULL_ELF_FILE = $(CUR)/$(ELF_FILE)

copy:
	@echo Copying generated kernel binary from $(OUTPUT) to $(FULL_ELF_FILE)
	@mkdir -p $(dir $(FULL_ELF_FILE))
	@rm -f $(FULL_ELF_FILE)
	@cp $(OUTPUT) $(FULL_ELF_FILE)

.PHONY: copy build all