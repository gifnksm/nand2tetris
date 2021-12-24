SHELL := bash
.ONESHELL:
.SHELLFLAGS := -eu -o pipefail -c
.DELETE_ON_ERROR:
MAKEFLAGS += --warn-undefined-variables
MAKEFLAGS += --no-builtin-rules

ifeq ($(shell [ -f .nocompile ] && echo 1),1)
NOCOMPILE=1
endif

DIRNAME=$(notdir $(CURDIR))
HDL=$(wildcard *.hdl)
ASM=$(wildcard *.asm)
GIT_CDUP=$(patsubst %/,%,$(shell git rev-parse --show-cdup))
GIT_PREFIX=$(patsubst %/,%,$(shell git rev-parse --show-prefix))
TSTIGNORE=$(wildcard .tstignore)

OS_DIR=$(GIT_CDUP)/JackOS
OS_JACK=$(wildcard $(OS_DIR)/*.jack)
TARGET_OS_JACK=$(patsubst $(OS_DIR)/%,$(TARGET_DIR)/%,$(OS_JACK))

TARGET_DIR=$(GIT_CDUP)/target/nand2tetris/projects/$(GIT_PREFIX)
TARGET_JACK=$(wildcard $(TARGET_DIR)/*.jack) $(addprefix $(TARGET_DIR)/,$(wildcard *.jack)) $(TARGET_OS_JACK)
TARGET_JACK_ANALYZED=$(TARGET_DIR)/.jack.analyzed
TARGET_COMPILED_VM=$(patsubst %.jack,%.vm,$(TARGET_JACK))
TARGET_REFERENCE_JACK=$(patsubst $(TARGET_DIR)/%.jack,$(TARGET_DIR)/JackCompiler/%.jack,$(TARGET_JACK))
TARGET_REFERENCE_VM=$(patsubst %.jack,%.vm,$(TARGET_REFERENCE_JACK))
TARGET_VM=$(TARGET_COMPILED_VM)
TARGET_TOKEN_XML=$(patsubst %.jack,%.token.xml,$(TARGET_JACK))
TARGET_AST_XML=$(patsubst %.jack,%.ast.xml,$(TARGET_JACK))
TARGET_TYPED_AST_XML=$(patsubst %.jack,%.typed-ast.xml,$(TARGET_JACK))
TARGET_TEST = $(wildcard $(TARGET_DIR)/*.tst)

TEST_TOKEN=$(patsubst $(TARGET_DIR)/%T.xml,test-token-%,$(wildcard $(TARGET_DIR)/*T.xml))
TEST_AST=$(patsubst $(TARGET_DIR)/%T.xml,test-ast-%,$(wildcard $(TARGET_DIR)/*T.xml))

TARGET=\
    $(TARGET_JACK) \
    $(TARGET_JACK_ANALYZED) \
    $(TARGET_TOKEN_XML) \
    $(TARGET_AST_XML) \
    $(TARGET_TYPED_AST_XML) \
    $(TARGET_COMPILED_VM) \
    $(TARGET_REFERENCE_JACK) \
    $(TARGET_REFERENCE_VM)

ifndef NOCOMPILE
TARGET+=\
    $(TARGET_DIR)/$(DIRNAME).asm \
    $(TARGET_DIR)/$(DIRNAME).hack \
    $(TARGET_DIR)/$(DIRNAME).dasm \
    $(TARGET_VM)
endif

HASM=$(GIT_CDUP)/target/release/hasm
VMTRANS=$(GIT_CDUP)/target/release/vmtrans
HDISASM=$(GIT_CDUP)/target/release/hdisasm
JACK_ANALYZER=$(GIT_CDUP)/target/release/jack-analyzer

all: $(TARGET)
.PHONY: all

clean:
.PHONY: clean

test: $(patsubst $(TARGET_DIR)/%.tst,test-%,$(TARGET_TEST)) $(TARGET) $(TEST_TOKEN) $(TEST_AST)
.PHONY: test

test-%: $(TARGET_DIR)/%.tst $(TARGET)
	$(GIT_CDUP)/misc/run_test $<
.PHONY: test-%

test-token-%: $(TARGET_DIR)/%T.xml $(TARGET_DIR)/%.token.xml
	diff -u --strip-trailing-cr $^

test-ast-%: $(TARGET_DIR)/%.xml $(TARGET_DIR)/%.ast.xml
	diff -u --strip-trailing-cr $^


$(TARGET_DIR)/%.jack: %.jack | $(TARGET_DIR)/
	cp $< $@

$(TARGET_DIR)/JackCompiler/%.jack: $(TARGET_DIR)/%.jack | $(TARGET_DIR)/JackCompiler/
	cp $< $@
$(TARGET_DIR)/JackCompiler/%.vm: $(TARGET_DIR)/JackCompiler/%.jack
	$(GIT_CDUP)/tools/JackCompiler $<

$(TARGET_COMPILED_VM): $(TARGET_DIR)/.jack.analyzed
$(TARGET_TOKEN_XML): $(TARGET_DIR)/.jack.analyzed
$(TARGET_AST_XML): $(TARGET_DIR)/.jack.analyzed
$(TARGET_TYPED_AST_XML): $(TARGET_DIR)/.jack.analyzed

$(TARGET_DIR)/.jack.analyzed: $(TARGET_JACK) $(JACK_ANALYZER) | $(TARGET_DIR)/
	$(JACK_ANALYZER) $(TARGET_DIR)
	touch $@

$(TARGET_DIR)/%.jack: $(OS_DIR)/%.jack | $(TARGET_DIR)/
	ln -sf $(GIT_CDUP)/../../../JackOS/$(notdir $<) $@

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

$(JACK_ANALYZER):
	cargo build --release --bin jack-analyzer
.PHONY: $(JACK_ANALYZER)

$(TARGET_DIR)/:
	mkdir -p $@
$(TARGET_DIR)/JackCompiler/:
	mkdir -p $@
