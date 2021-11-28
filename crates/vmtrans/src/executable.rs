use crate::{
    code_gen::CodeGen, Command, Error, FuncName, FuncProp, Module, ModuleName, ParseModuleErrorKind,
};
use hasm::Statement;
use std::{collections::BTreeMap, path::PathBuf};

#[derive(Debug, Clone)]
pub struct Executable {
    modules: BTreeMap<ModuleName, Module>,
    functions: BTreeMap<FuncName, (ModuleName, Vec<Command>)>,
}

impl Executable {
    pub fn open(module_paths: &[PathBuf]) -> Result<Self, Error> {
        let mut functions = FunctionTable::new();
        let modules = module_paths
            .iter()
            .map(|path| {
                Module::open(path, &mut functions).map(|module| (module.name().to_owned(), module))
            })
            .collect::<Result<BTreeMap<_, _>, _>>()?;
        if modules.is_empty() {
            return Err(Error::NoModules);
        }
        let functions = functions.finish()?;
        Ok(Self { modules, functions })
    }

    pub fn translate(&self) -> Vec<Statement> {
        let mut stmts = self.bootstrap();
        for (func_name, (module_name, commands)) in &self.functions {
            for (index, command) in commands.iter().enumerate() {
                stmts.extend(command.translate(module_name, func_name, index));
            }
        }
        stmts
    }

    fn bootstrap(&self) -> Vec<Statement> {
        let module_name = ModuleName::builtin();
        let func_name = FuncName::bootstrap();
        let mut gen = CodeGen::new(&module_name, &func_name, 0);
        if let Some((entry_point, _)) = self.functions.get_key_value(&FuncName::entry_point()) {
            gen.bootstrap(entry_point);
        }
        gen.into_statements()
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct Function {
    called: Option<FuncProp>,
    defined: Option<(FuncProp, ModuleName, Vec<Command>)>,
}

#[derive(Debug, Clone)]
pub(crate) struct FunctionTable {
    functions: BTreeMap<FuncName, Function>,
}

impl FunctionTable {
    pub(crate) fn new() -> Self {
        Self {
            functions: BTreeMap::new(),
        }
    }

    pub(crate) fn call(
        &mut self,
        name: &FuncName,
        prop: FuncProp,
    ) -> Result<(), ParseModuleErrorKind> {
        let f = self.functions.entry(name.clone()).or_default();
        let arity = prop.arity;
        let called = f.called.get_or_insert(prop);
        if called.arity != arity {
            return Err(ParseModuleErrorKind::CallerArityMismatch(
                name.clone(),
                arity,
                called.clone(),
            ));
        }
        Ok(())
    }

    pub(crate) fn define(
        &mut self,
        module_name: &ModuleName,
        func_name: &FuncName,
        prop: FuncProp,
        body: Vec<Command>,
    ) -> Result<(), ParseModuleErrorKind> {
        let f = self.functions.entry(func_name.clone()).or_default();
        if let Some((defined, _, _)) = f.defined.replace((prop, module_name.clone(), body)) {
            return Err(ParseModuleErrorKind::FunctionRedefinition(
                func_name.clone(),
                defined,
            ));
        }
        Ok(())
    }

    pub(crate) fn finish(self) -> Result<BTreeMap<FuncName, (ModuleName, Vec<Command>)>, Error> {
        let mut functions = BTreeMap::new();
        self.functions
            .into_iter()
            .find_map(|(func_name, state)| match (state.defined, state.called) {
                (None, Some(called)) => Some(Err(Error::FunctionNotDefined(func_name, called))),
                (Some((defined, _, _)), Some(called)) if defined.arity != called.arity => {
                    Some(Err(Error::ArityMismatch(func_name, defined, called)))
                }
                (Some((_, module_name, body)), _) => {
                    functions.insert(func_name, (module_name, body));
                    None
                }
                _ => None,
            })
            .unwrap_or(Ok(()))?;

        if !functions.contains_key(&FuncName::entry_point()) && functions.len() > 1 {
            return Err(Error::NoEntryPoint);
        }
        Ok(functions)
    }
}
