include config.mak

export CUR	:= $(shell pwd)

all: $(ALL_BUILDS)

build: 
	make $(DEFAULT_BUILD)

run:
	make -f build.mak run BUILD_NAME=$(DEFAULT_BUILD)

run_only:
	make -f build.mak run_only BUILD_NAME=$(DEFAULT_BUILD) 

clean:
	@rm -rf bin target runtime

$(ALL_BUILDS):
	make -f build.mak build BUILD_NAME=$@

.PHONY: $(ALL_BUILDS) help build run run_only clean

exec:
	@make -s -f build.mak exec BUILD_NAME=$(DEFAULT_BUILD) 