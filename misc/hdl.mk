SHELL := bash
.ONESHELL:
.SHELLFLAGS := -eu -o pipefail -c
.DELETE_ON_ERROR:
MAKEFLAGS += --warn-undefined-variables
MAKEFLAGS += --no-builtin-rules

DIRNAME=$(notdir $(CURDIR))
HDL=$(wildcard *.hdl)
TARGET_DIR=../target/nand2tetris/projects/$(DIRNAME)

all:
.PHONY: all

clean:
.PHONY: clean

test:
.PHONY: test

test: $(patsubst %.hdl,test-%,$(HDL))
.PHONY: test

test-%: $(TARGET_DIR)/%.ok
	@echo "[TEST $(DIRNAME)/$(notdir $(basename $<))] OK"
.PHONY: test-%
.PRECIOUS: %

$(TARGET_DIR)/%.ok: $(TARGET_DIR)/%.hdl $(addprefix $(TARGET_DIR)/,$(HDL))
	@echo "[TEST $(DIRNAME)/$(notdir $(basename $<))] running ..."
	../tools/HardwareSimulator $(TARGET_DIR)/$(notdir $(basename $@)).tst
	@touch $@

$(TARGET_DIR)/%.hdl: %.hdl
	cp $< $@
