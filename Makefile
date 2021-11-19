SHELL := bash
.ONESHELL:
.SHELLFLAGS := -eu -o pipefail -c
.DELETE_ON_ERROR:
MAKEFLAGS += --warn-undefined-variables
MAKEFLAGS += --no-builtin-rules

SUBDIRS = $(wildcard [0-9][0-9])

all: $(patsubst %,all-%,$(SUBDIRS))
.PHONY: all

clean: $(patsubst %,clean-%,$(SUBDIRS))
	$(RM) -r target
.PHONY: clean

test: $(patsubst %,test-%,$(SUBDIRS)) test-cargo
.PHONY: test

setup: tools
.PHONY: setup

tools: target/nand2tetris/
.PHONY: tools

%/:
	mkdir -p $@

all-%:
	$(MAKE) -C $(patsubst all-%,%,$@) all
.PHONY: all-%

clean-%:
	$(MAKE) -C $(patsubst clean-%,%,$@) clean
.PHONY: clean-%

test-%: tools
	$(MAKE) -C $(patsubst test-%,%,$@) test
.PHONY: test-%

test-cargo:
	cargo test
.PHONY: test-cargo

target/nand2tetris.zip: | target/
	curl -L -o $@ "https://drive.google.com/uc?export=download&id=1xZzcMIUETv3u3sdpM_oTJSTetpVee3KZ"
# Original URL: https://drive.google.com/file/d/1xZzcMIUETv3u3sdpM_oTJSTetpVee3KZ/view?usp=sharing

target/nand2tetris/: target/nand2tetris.zip
	bsdtar -C target/ -xf $<
	touch $@
