SHELL := bash
.ONESHELL:
.SHELLFLAGS := -eu -o pipefail -c
.DELETE_ON_ERROR:
MAKEFLAGS += --warn-undefined-variables
MAKEFLAGS += --no-builtin-rules

all:
.PHONY: all

clean:
	$(RM) -r target
.PHONY: clean

setup: tools
.PHONY: setup

tools: target/nand2tetris/
.PHONY: tools

%/:
	mkdir -p $@

target/nand2tetris.zip: | target/
	curl -L -o $@ "https://drive.google.com/uc?export=download&id=1xZzcMIUETv3u3sdpM_oTJSTetpVee3KZ"
# Original URL: https://drive.google.com/file/d/1xZzcMIUETv3u3sdpM_oTJSTetpVee3KZ/view?usp=sharing

target/nand2tetris/: target/nand2tetris.zip
	bsdtar -C target/ -xmf $<
