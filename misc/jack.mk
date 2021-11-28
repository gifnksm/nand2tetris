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

OS_DIR=$(GIT_CDUP)/target/nand2tetris/tools/OS
OS_JACK=$(wildcard $(OS_DIR)/*.vm)
OS_VM=$(addprefix $(TARGET_DIR)/,$(notdir $(OS_JACK)))

TARGET_DIR=$(GIT_CDUP)/target/nand2tetris/projects/$(GIT_PREFIX)
TARGET_JACK=$(wildcard $(TARGET_DIR)/*.jack) $(addprefix $(TARGET_DIR)/,$(wildcard *.jack))
TARGET_VM=$(patsubst %.jack,%.vm,$(TARGET_JACK)) $(OS_VM)
TARGET_TEST = $(wildcard $(TARGET_DIR)/*.tst)

TARGET=$(TARGET_DIR)/$(DIRNAME).asm $(TARGET_DIR)/$(DIRNAME).hack  $(TARGET_DIR)/$(DIRNAME).dasm $(TARGET_JACK) $(TARGET_VM)

HASM=$(GIT_CDUP)/target/release/hasm
VMTRANS=$(GIT_CDUP)/target/release/vmtrans
HDISASM=$(GIT_CDUP)/target/release/hdisasm

all: $(TARGET)
.PHONY: all

clean:
.PHONY: clean

test: $(patsubst $(TARGET_DIR)/%.tst,test-%,$(TARGET_TEST)) $(TARGET)
.PHONY: test

test-%: $(TARGET_DIR)/%.tst $(TARGET)
	$(GIT_CDUP)/misc/run_test $<
.PHONY: test-%

$(TARGET_DIR)/%.jack: %.jack | $(TARGET_DIR)/
	cp $< $@

$(TARGET_DIR)/%.vm: $(TARGET_DIR)/%.jack | $(TARGET_DIR)/
	$(GIT_CDUP)/tools/JackCompiler $<

$(TARGET_DIR)/%.vm: $(OS_DIR)/%.vm | $(TARGET_DIR)/
	ln -sf $(GIT_CDUP)/../tools/OS/$(notdir $<) $@

$(TARGET_DIR)/$(DIRNAME).asm: $(TARGET_VM) $(VMTRANS)
	$(VMTRANS) $(TARGET_DIR)

%.hack: %.asm $(HASM)
	$(HASM) $<

%.dasm: %.hack $(HDISASM)
	$(HDISASM) $<

$(VMTRANS):
	cargo build --release --bin vmtrans
.PHONY: $(VMTRANS)

$(HASM):
	cargo build --release --bin hasm
.PHONY: $(HASM)

$(HDISASM):
	cargo build --release --bin hdisasm
.PHONY: $(HDISASM)

$(TARGET_DIR)/:
	mkdir -p $@
