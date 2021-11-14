SHELL := bash
.ONESHELL:
.SHELLFLAGS := -eu -o pipefail -c
.DELETE_ON_ERROR:
MAKEFLAGS += --warn-undefined-variables
MAKEFLAGS += --no-builtin-rules

SUBDIRS = a b

all: $(patsubst %,all-%,$(SUBDIRS))
.PHONY: all

clean: $(patsubst %,clean-%,$(SUBDIRS))
	$(RM) -r target
.PHONY: clean

test: $(patsubst %,test-%,$(SUBDIRS))
.PHONY: test

all-%:
	$(MAKE) -C $(patsubst all-%,%,$@) all
.PHONY: all-%

clean-%:
	$(MAKE) -C $(patsubst clean-%,%,$@) clean
.PHONY: clean-%

test-%:
	$(MAKE) -C $(patsubst test-%,%,$@) test
.PHONY: test-%
