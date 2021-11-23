SHELL := bash
.ONESHELL:
.SHELLFLAGS := -eu -o pipefail -c
.DELETE_ON_ERROR:
MAKEFLAGS += --warn-undefined-variables
MAKEFLAGS += --no-builtin-rules

DIRNAME=$(notdir $(CURDIR))
HDL=$(wildcard *.hdl)
ASM=$(wildcard *.asm)
GIT_CDUP=$(patsubst %/,%,$(shell git rev-parse --show-cdup))
GIT_PREFIX=$(patsubst %/,%,$(shell git rev-parse --show-prefix))
TSTIGNORE=$(wildcard .tstignore)

TARGET_DIR=$(GIT_CDUP)/target/nand2tetris/projects/$(GIT_PREFIX)
TARGET=$(TARGET_DIR)/$(DIRNAME).asm $(TARGET_DIR)/$(DIRNAME).hack
TARGET_VM=$(wildcard $(TARGET_DIR)/*.vm)
TARGET_TEST = $(wildcard $(TARGET_DIR)/*.tst)

HASM=$(GIT_CDUP)/target/release/hasm
VMTRANS=$(GIT_CDUP)/target/release/vmtrans

all: $(TARGET)
.PHONY: all

clean:
.PHONY: clean

test: $(patsubst $(TARGET_DIR)/%.tst,test-%,$(TARGET_TEST)) $(TARGET)
.PHONY: test

test-%: $(TARGET_DIR)/%.tst $(TARGET)
	$(GIT_CDUP)/misc/run_test $<
.PHONY: test-%

$(TARGET_DIR)/$(DIRNAME).asm: $(TARGET_VM) $(VMTRANS)
	$(VMTRANS) $(TARGET_DIR)

%.hack: %.asm $(HASM)
	$(HASM) $<

$(VMTRANS):
	cargo build --release --bin vmtrans
.PHONY: $(VMTRANS)

$(HASM):
	cargo build --release --bin hasm
.PHONY: $(HASM)
