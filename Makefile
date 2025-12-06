include config.mak

export CUR	:= $(shell pwd)

build: $(DEFAULT_BUILD)

run:
	make -f build.mak run BUILD_NAME=$(DEFAULT_BUILD)

clean:
	@rm -rf bin target runtime

$(ALL_BUILDS):
	make -f build.mak build BUILD_NAME=$@

.PHONY: $(ALL_BUILDS) help build