FULL_USER_DIR	:= $(CUR)/$(USER_DIR)

OUTPUT_DIR		:= $(CUR)/target/$(TARGET)/$(BUILD_TYPE)
%:
	make -C $^ build
	@mkdir -p $(FULL_USER_DIR)
	@cp $(OUTPUT_DIR)/$^ $(FULL_USER_DIR)/$^
build: init
