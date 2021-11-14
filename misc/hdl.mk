SHELL := bash
.ONESHELL:
.SHELLFLAGS := -eu -o pipefail -c
.DELETE_ON_ERROR:
MAKEFLAGS += --warn-undefined-variables
MAKEFLAGS += --no-builtin-rules

DIRNAME=$(notdir $(CURDIR))
HDL=$(wildcard *.hdl)
GIT_CDUP=$(shell git rev-parse --show-cdup)
GIT_PREFIX=$(patsubst %/,%,$(shell git rev-parse --show-prefix))
TARGET_DIR=$(GIT_CDUP)/target/nand2tetris/projects/$(GIT_PREFIX)

all:
.PHONY: all

clean:
.PHONY: clean

test: $(patsubst %.hdl,test-%,$(HDL))
.PHONY: test

test-%: $(TARGET_DIR)/%.ok
	@echo "[TEST $(GIT_PREFIX)/$(notdir $(basename $<))] OK"
.PHONY: test-%
.PRECIOUS: $(TARGET_DIR)/%.ok

$(TARGET_DIR)/%.ok: $(TARGET_DIR)/%.hdl $(addprefix $(TARGET_DIR)/,$(HDL))
	@echo "[TEST $(GIT_PREFIX)/$(notdir $(basename $<))] running ..."
	$(GIT_CDUP)tools/HardwareSimulator $(TARGET_DIR)/$(notdir $(basename $@)).tst
	@touch $@

$(TARGET_DIR)/%.hdl: %.hdl
	cp $< $@
