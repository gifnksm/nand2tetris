use super::*;
use crate::{
    control_flow_graph::{BasicBlock, BbId, CfgClass, CfgStatement, CfgSubroutine, Exit},
    token::Location,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ToCfgError {}

pub trait ToControlFlowGraph {
    type Output;
    fn to_control_flow_graph(&self) -> Result<Self::Output, ToCfgError>;
}

impl<T> ToControlFlowGraph for WithLoc<T>
where
    T: ToControlFlowGraph,
{
    type Output = WithLoc<T::Output>;
    fn to_control_flow_graph(&self) -> Result<Self::Output, ToCfgError> {
        Ok(WithLoc {
            loc: self.loc,
            data: self.data.to_control_flow_graph()?,
        })
    }
}

impl ToControlFlowGraph for TypedClass {
    type Output = CfgClass;
    fn to_control_flow_graph(&self) -> Result<Self::Output, ToCfgError> {
        Ok(CfgClass {
            name: self.name.clone(),
            static_vars: self.static_vars.clone(),
            fields: self.fields.clone(),
            subs: self
                .subs
                .iter()
                .map(|sub| sub.to_control_flow_graph())
                .collect::<Result<_, _>>()?,
        })
    }
}

impl ToControlFlowGraph for TypedSubroutine {
    type Output = CfgSubroutine;
    fn to_control_flow_graph(&self) -> Result<Self::Output, ToCfgError> {
        let mut blocks = vec![];
        let entry_block = new_block(self.name.loc, "entry");
        let entry_id = entry_block.data.id;
        blocks.push(entry_block);

        generate_blocks(&self.stmts, &mut blocks)?;

        Ok(CfgSubroutine {
            name: self.name.clone(),
            kind: self.kind,
            return_type: self.return_type.clone(),
            params: self.params.clone(),
            vars: self.vars.clone(),
            entry_id,
            blocks,
        })
    }
}

fn new_block(loc: Location, label: &'static str) -> WithLoc<BasicBlock> {
    WithLoc {
        loc,
        data: BasicBlock {
            id: BbId {
                line_num: loc.line_num,
                label,
            },
            stmts: vec![],
            exit: Exit::Unreachable,
        },
    }
}

fn push_stmt(blocks: &mut Vec<WithLoc<BasicBlock>>, stmt: CfgStatement) {
    let last_block = blocks.last_mut().unwrap();
    last_block.data.stmts.push(stmt);
}

fn update_exit(blocks: &mut [WithLoc<BasicBlock>], exit: Exit) {
    assert!(!matches!(exit, Exit::Unreachable));
    let mut last = blocks.last_mut().unwrap();
    assert!(
        matches!(last.data.exit, Exit::Unreachable),
        "unexpected exit at {}: {:?}",
        last.loc,
        last.data.exit
    );
    last.data.exit = exit;
}

fn generate_blocks(
    stmts: &[WithLoc<TypedStatement>],
    blocks: &mut Vec<WithLoc<BasicBlock>>,
) -> Result<(), ToCfgError> {
    for stmt in stmts {
        match &stmt.data {
            TypedStatement::Let(stmt) => {
                push_stmt(blocks, CfgStatement::Let(stmt.clone()));
            }
            TypedStatement::Do(stmt) => {
                push_stmt(blocks, CfgStatement::Do(stmt.clone()));
            }
            TypedStatement::If(stmt) => {
                let end_block = new_block(stmt.loc, "if_end");
                let then_block = new_block(stmt.loc, "if_then");
                let else_block = stmt
                    .data
                    .else_stmts
                    .is_some()
                    .then(|| new_block(stmt.loc, "if_else"));

                let end_id = end_block.data.id;
                let then_id = then_block.data.id;
                let else_id = else_block.as_ref().map(|b| b.data.id).unwrap_or(end_id);

                update_exit(blocks, Exit::If(stmt.data.cond.clone(), then_id, else_id));

                blocks.push(then_block);
                generate_blocks(&stmt.data.then_stmts, blocks)?;
                update_exit(blocks, Exit::Goto(end_id));

                if let Some(else_stmts) = &stmt.data.else_stmts {
                    blocks.push(else_block.unwrap());
                    generate_blocks(else_stmts, blocks)?;
                    update_exit(blocks, Exit::Goto(end_id));
                }

                blocks.push(end_block);
            }
            TypedStatement::While(stmt) => {
                let end_block = new_block(stmt.loc, "while_end");
                let body_block = new_block(stmt.loc, "while_body");
                let cond_block = new_block(stmt.loc, "while_cond");

                let end_id = end_block.data.id;
                let body_id = body_block.data.id;
                let cond_id = cond_block.data.id;

                update_exit(blocks, Exit::If(stmt.data.cond.clone(), cond_id, end_id));

                blocks.push(cond_block);
                update_exit(blocks, Exit::If(stmt.data.cond.clone(), body_id, end_id));

                blocks.push(body_block);
                generate_blocks(&stmt.data.stmts, blocks)?;
                update_exit(blocks, Exit::Goto(cond_id));

                blocks.push(end_block);
            }
            TypedStatement::Return(stmt) => {
                update_exit(blocks, Exit::Return(stmt.data.expr.clone()));
                blocks.push(new_block(stmt.loc, "unreachable"));
            }
        }
    }
    Ok(())
}
