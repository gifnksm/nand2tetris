use super::Executable;
use crate::{code_gen::CodeGen, Command, FuncName, ModuleName};
use asm::Statement;
use std::collections::{BTreeSet, VecDeque};

impl Executable {
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
