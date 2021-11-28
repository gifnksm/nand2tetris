use crate::{
    code_gen::CodeGen, Command, Error, FuncName, FuncProp, Module, ModuleName, ParseModuleErrorKind,
};
use hasm::Statement;
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    path::PathBuf,
};

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
        let mut stmts = Vec::new();
        let entry_point = self.bootstrap(&mut stmts);

        let mut visited = BTreeSet::new();
        let mut to_visit = VecDeque::new();
        if let Some(entry_point) = entry_point {
            to_visit.push_front(entry_point);
        }
        while let Some(func_name) = to_visit.pop_front() {
            visited.insert(func_name);
            let (_, commands) = self.functions.get(func_name).unwrap();
            for command in commands {
                if let Command::Call(callee, _) = command {
                    if !visited.contains(callee) {
                        to_visit.push_back(callee);
                    }
                }
            }
        }

        for func_name in visited {
            let (module_name, commands) = self.functions.get(func_name).unwrap();
            for (index, command) in commands.iter().enumerate() {
                command.translate(module_name, func_name, index, &mut stmts);
            }
        }

        stmts
    }

    fn bootstrap(&self, stmts: &mut Vec<Statement>) -> Option<&FuncName> {
        let module_name = ModuleName::builtin();
        let func_name = FuncName::bootstrap();
        let mut gen = CodeGen::new(&module_name, &func_name, 0, stmts);
        let entry_point = if let Some((entry_point, _)) =
            self.functions.get_key_value(&FuncName::entry_point())
        {
            gen.bootstrap(entry_point);
            Some(entry_point)
        } else {
            assert!(self.functions.len() <= 1);
            self.functions.keys().next()
        };
        entry_point
    }
}

#[derive(Debug, Clone, Default)]
struct FunctionState {
    called: Option<FuncProp>,
    defined: Option<(FuncProp, ModuleName, Vec<Command>)>,
}

#[derive(Debug, Clone)]
pub(crate) struct FunctionTable {
    functions: BTreeMap<FuncName, FunctionState>,
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
