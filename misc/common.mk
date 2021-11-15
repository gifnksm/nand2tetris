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
TARGET_HACK=$(patsubst %.asm,$(TARGET_DIR)/%.hack,$(ASM))
TARGET_ASM=$(addprefix $(TARGET_DIR)/,$(ASM))
TARGET_HDL=$(addprefix $(TARGET_DIR)/,$(HDL))
TARGET_TSTIGNORE=$(addprefix $(TARGET_DIR)/,$(TSTIGNORE))
TARGET=$(TARGET_HACK) $(TARGET_ASM) $(TARGET_HDL) $(TARGET_TSTIGNORE)

TEST = $(wildcard $(TARGET_DIR)/*.tst)

all: $(TARGET)
.PHONY: all

clean:
.PHONY: clean

test: $(patsubst $(TARGET_DIR)/%.tst,test-%,$(TEST))
.PHONY: test

test-%: $(TARGET_DIR)/%.tst $(TARGET)
	$(GIT_CDUP)/misc/run_test $<
.PHONY: test-%
.PRECIOUS: $(TARGET_DIR)/%.ok

$(TARGET_DIR)/%.hack: $(TARGET_DIR)/%.asm
	$(GIT_CDUP)/tools/Assembler $(abspath $<)

$(TARGET_DIR)/%.hdl: %.hdl
	cp $< $@
$(TARGET_DIR)/%.asm: %.asm
	cp $< $@

$(TARGET_DIR)/.tstignore: .tstignore
	cp $< $@
