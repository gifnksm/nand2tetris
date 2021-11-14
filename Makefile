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

test: $(patsubst %,test-%,$(SUBDIRS))
.PHONY: test

setup: tools
.PHONY: setup

tools: target/nand2tetris
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

target/build/nand2tetris.zip: | target/build/
	curl -L -o $@ "https://drive.google.com/uc?export=download&id=1xZzcMIUETv3u3sdpM_oTJSTetpVee3KZ"
# Original URL: https://drive.google.com/file/d/1xZzcMIUETv3u3sdpM_oTJSTetpVee3KZ/view?usp=sharing

target/nand2tetris: target/build/nand2tetris.zip
	bsdtar -C target/ -xf $<
	chmod +x $@/tools/*.sh
	touch $@

target/build/nand2tetris-open-source-2.5.7.zip: | target/build/
	curl -L -o $@ "https://drive.google.com/uc?export=download&id=1stcWUSeAixCRHWOjc9sgBhq5voSLvun8"
# Original URL: https://drive.google.com/file/d/1stcWUSeAixCRHWOjc9sgBhq5voSLvun8/view

my-tools: target/build/tools/Makefile target/build/tools/src
	$(MAKE) -C target/build/tools

target/build/tools/src.orig: target/build/nand2tetris-open-source-2.5.7.zip
	$(RM) -r $@ $@.tmp
	mkdir -p $@.tmp
	bsdtar -C $@.tmp -xf $<
	mv $@.tmp $@

target/build/tools/src: target/build/tools/src.orig misc/patches/tool/*.patch
	$(RM) -r $@ $@.tmp
	cp -r $< $@.tmp
	cat misc/patches/tool/*.patch | (cd $@.tmp && patch -p1)
	mv $@.tmp $@

target/build/tools/Makefile: | target/build/tools/
	ln -sf ../../../misc/tools.mk $@
