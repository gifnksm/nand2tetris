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

TARGET_DIR=$(GIT_CDUP)/target/nand2tetris/projects/$(GIT_PREFIX)
HACK=$(patsubst %.asm,$(TARGET_DIR)/%.hack,$(ASM))
TARGET_ASM=$(addprefix $(TARGET_DIR)/,$(ASM))
TARGET_HDL=$(addprefix $(TARGET_DIR)/,$(ASM))
TARGET=$(HACK) $(TARGET_ASM) $(TARGET_HDL)

all: $(TARGET)
.PHONY: all

clean:
.PHONY: clean

test: $(patsubst %.hdl,test-%,$(HDL)) $(patsubst %.asm,test-%,$(ASM))
.PHONY: test

test-%: $(TARGET_DIR)/%.hdl $(addprefix $(TARGET_DIR)/,$(HDL))
	echo "[TEST HDL $(GIT_PREFIX)/$(notdir $(basename $<))] running ..."
	$(GIT_CDUP)/tools/HardwareSimulator $(abspath $(TARGET_DIR)/$(notdir $(basename $<)).tst)
	echo [TEST HDL $(GIT_PREFIX)/$(notdir $(basename $<))] done
.PHONY: test-%
.PRECIOUS: $(TARGET_DIR)/%.ok

test-%: $(TARGET_DIR)/%.hack
	echo "[TEST ASM $(GIT_PREFIX)/$(notdir $(basename $<))] running ..."
	$(GIT_CDUP)/tools/CPUEmulator $(abspath $(TARGET_DIR)/$(notdir $(basename $<)).tst)
	echo [TEST HDL $(GIT_PREFIX)/$(notdir $(basename $<))] done

$(TARGET_DIR)/%.hack: $(TARGET_DIR)/%.asm
	$(GIT_CDUP)/tools/Assembler $(abspath $<)

$(TARGET_DIR)/%.hdl: %.hdl
	cp $< $@
$(TARGET_DIR)/%.asm: %.asm
	cp $< $@
