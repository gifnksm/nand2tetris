lSHELL := bash
.ONESHELL:
.SHELLFLAGS := -eu -o pipefail -c
.DELETE_ON_ERROR:
MAKEFLAGS += --warn-undefined-variables
MAKEFLAGS += --no-builtin-rules

all:
.PHONY: all

clean:
	$(RM) -r classes packages
.PHONY: clean

dist:
.PHONY: dist

packages:
.PHONY: packages

classes:
.PHONY: classes

%/:
	mkdir -p $@

dist/:
	cp -r src/InstallDir $@
	chmod +x $@/*.sh

PACKAGES   = packages/Hack.jar packages/HackGUI.jar packages/Compilers.jar packages/Simulators.jar packages/SimulatorsGUI.jar
CLASS_PATH = packages/Hack.jar:packages/HackGUI.jar:packages/Compilers.jar:packages/Simulators.jar:packages/SimulatorsGUI.jar

packages: $(PACKAGES)

HACK_SRCS              = $(shell find src/HackPackageSource -name "*.java")
HACK_CLASSES           = $(patsubst src/HackPackageSource/%.java,classes/Hack/%.class,$(HACK_SRCS))
HACK_GUI_SRCS          = $(shell find src/HackGUIPackageSource -name "*.java")
HACK_GUI_CLASSES       = $(patsubst src/HackGUIPackageSource/%.java,classes/HackGUI/%.class,$(HACK_GUI_SRCS))
COMPILERS_SRCS         = $(shell find src/CompilersPackageSource -name "*.java")
COMPILERS_CLASSES      = $(patsubst src/CompilersPackageSource/%.java,classes/Compilers/%.class,$(COMPILERS_SRCS))
SIMULATORS_SRCS        = $(shell find src/SimulatorsPackageSource -name "*.java")
SIMULATORS_CLASSES     = $(patsubst src/SimulatorsPackageSource/%.java,classes/Simulators/%.class,$(SIMULATORS_SRCS))
SIMULATORS_GUI_SRCS    = $(shell find src/SimulatorsGUIPackageSource -name "*.java")
SIMULATORS_GUI_CLASSES = $(patsubst src/SimulatorsGUIPackageSource/%.java,classes/SimulatorsGUI/%.class,$(SIMULATORS_GUI_SRCS))

BUILT_IN_CHIPS_SRCS      = $(shell find src/BuiltInChipsSource -name "*.java")
BUILT_IN_CHIPS_CLASSES   = $(patsubst src/BuiltInChipsSource/%.java,classes/BuiltInChips/builtInChips/%.class,$(BUILT_IN_CHIPS_SRCS))
BUILT_IN_VM_CODE_SRCS    = $(shell find src/BuiltInVMCodeSource -name "*.java")
BUILT_IN_VM_CODE_CLASSES = $(patsubst src/BuiltInVMCodeSource/%.java,classes/BuiltInVMCode/builtInVMCode/%.class,$(BUILT_IN_VM_CODE_SRCS))
MAIN_CLASSES_SRCS        = $(shell find src/MainClassesSource -name "*.java")
MAIN_CLASSES_CLASSES     = $(patsubst src/MainClassesSource/%.java,classes/MainClasses/%.class,$(MAIN_CLASSES_SRCS))


CLASSES = $(BUILT_IN_CHIPS_CLASSES) $(BUILT_IN_VM_CODE_CLASSES) $(MAIN_CLASSES_CLASSES)
classes: $(CLASSES)

DIST = dist/ $(patsubst packages/%.jar,dist/bin/lib/%.jar,$(PACKAGES))
dist: $(DIST)

dist/bin/lib/%.jar: packages/%.jar
	cp $< $@

packages/Hack.jar: $(HACK_CLASSES) | packages/
	jar -c -v -f $@ -M -C classes/Hack .
packages/HackGUI.jar: $(HACK_GUI_CLASSES) | packages/
	jar -c -v -f $@ -M -C classes/HackGUI .
packages/Compilers.jar: $(COMPILERS_CLASSES) | packages/
	jar -c -v -f $@ -M -C classes/Compilers .
packages/Simulators.jar: $(SIMULATORS_CLASSES) | packages/
	jar -c -v -f $@ -M -C classes/Simulators .
packages/SimulatorsGUI.jar: $(SIMULATORS_GUI_CLASSES) | packages/
	jar -c -v -f $@ -M -C classes/SimulatorsGUI .

classes/Hack/%.class: src/HackPackageSource/%.java | classes/
	javac --source-path src/HackPackageSource -d classes/Hack $<
classes/HackGUI/%.class: src/HackGUIPackageSource/%.java packages/Hack.jar | classes/
	javac -cp packages/Hack.jar \
	    --source-path src/HackGUIPackageSource -d classes/HackGUI $<
classes/Compilers/%.class: src/CompilersPackageSource/%.java packages/Hack.jar | classes/
	javac -cp packages/Hack.jar \
	    --source-path src/CompilersPackageSource -d classes/Compilers $<
classes/Simulators/%.class: src/SimulatorsPackageSource/%.java packages/Hack.jar packages/Compilers.jar | classes/
	javac -cp packages/Hack.jar:packages/Compilers.jar \
	    --source-path src/SimulatorsPackageSource -d classes/Simulators $<
classes/SimulatorsGUI/%.class: src/SimulatorsGUIPackageSource/%.java packages/Hack.jar packages/HackGUI.jar packages/Compilers.jar packages/Simulators.jar | classes/
	javac -cp packages/Hack.jar:packages/HackGUI.jar:packages/Compilers.jar:packages/Simulators.jar \
	    --source-path src/SimulatorsGUIPackageSource -d classes/SimulatorsGUI $<

classes/BuiltInChips/builtInChips/%.class: src/BuiltInChipsSource/%.java packages/Hack.jar packages/HackGUI.jar packages/Compilers.jar packages/Simulators.jar packages/SimulatorsGUI.jar | classes/
	javac -cp packages/Hack.jar:packages/HackGUI.jar:packages/Compilers.jar:packages/Simulators.jar:packages/SimulatorsGUI.jar:classes/BuiltInChips \
	    -d classes/BuiltInChips $<

classes/BuiltInChips/builtInChips/RAM8.class: classes/BuiltInChips/builtInChips/RAM.class
classes/BuiltInChips/builtInChips/DRegister.class: classes/BuiltInChips/builtInChips/RegisterWithGUI.class

classes/BuiltInVMCode/builtInVMCode/%.class: src/BuiltInVMCodeSource/%.java packages/Hack.jar packages/HackGUI.jar packages/Compilers.jar packages/Simulators.jar packages/SimulatorsGUI.jar | classes/
	javac -cp packages/Hack.jar:packages/HackGUI.jar:packages/Compilers.jar:packages/Simulators.jar:packages/SimulatorsGUI.jar:classes/BuiltInVMCode \
	    -d classes/BuiltInVMCode $<
classes/BuiltInVMCode/builtInVMCode/String.class: classes/BuiltInVMCode/builtInVMCode/JackOSClass.class
classes/BuiltInVMCode/builtInVMCode/Array.class: classes/BuiltInVMCode/builtInVMCode/JackOSClass.class

classes/MainClasses/%.class: src/MainClassesSource/%.java packages/Hack.jar packages/HackGUI.jar packages/Compilers.jar packages/Simulators.jar packages/SimulatorsGUI.jar | classes/
	javac -cp packages/Hack.jar:packages/HackGUI.jar:packages/Compilers.jar:packages/Simulators.jar:packages/SimulatorsGUI.jar \
	    -d classes/MainClasses $<
