use super::*;
use crate::{
    control_flow_graph::{BasicBlock, BbId, CfgClass, Exit},
    token::Location,
};
use std::{
    collections::{HashMap, HashSet},
    mem,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OptimizeError {
    #[error("no return statement at end of subroutine: {}", _0)]
    NoReturnStatement(Location),
}

impl WithLoc<CfgClass> {
    pub fn optimize(&mut self) -> Result<(), OptimizeError> {
        for sub in &mut self.data.subs {
            let mut blocks = mem::take(&mut sub.data.blocks);
            let mut block_count = usize::MAX;
            let mut updated = false;
            while sub.data.blocks.len() < block_count || updated {
                updated = false;
                block_count = sub.data.blocks.len();
                updated |= replace_empty(&mut blocks);
                updated |= concat_unique(&mut blocks);
                blocks = remove_unreachable(sub.data.entry_id, blocks)?;
            }
            sub.data.blocks = blocks;
            sub.data.update_bb_links();
        }
        Ok(())
    }
}

fn replace_empty(blocks: &mut [WithLoc<BasicBlock>]) -> bool {
    let mut updated = false;
    let mut replace_block = HashMap::new();

    for block in &mut *blocks {
        match &block.data.exit {
            Exit::Return(_) => {}
            Exit::Goto(dest) => {
                if block.data.stmts.is_empty() {
                    replace_block.insert(block.data.id, *dest);
                }
            }
            Exit::If(_, _, _) => {}
            Exit::Unreachable => {}
        }
    }

    for block in &mut *blocks {
        match &mut block.data.exit {
            Exit::Return(_) => {}
            Exit::Goto(dest) => {
                if let Some(new_dest) = replace_block.get(dest) {
                    *dest = *new_dest;
                    updated = true;
                }
            }
            Exit::If(_cond, then_bb, else_bb) => {
                if let Some(new_then_bb) = replace_block.get(then_bb) {
                    *then_bb = *new_then_bb;
                    updated = true;
                }
                if let Some(new_else_bb) = replace_block.get(else_bb) {
                    *else_bb = *new_else_bb;
                    updated = true;
                }
            }
            Exit::Unreachable => {}
        }
    }
    updated
}

fn concat_unique(blocks: &mut [WithLoc<BasicBlock>]) -> bool {
    let mut updated = false;
    let mut entry_count = HashMap::new();
    let block_map = blocks
        .iter()
        .enumerate()
        .map(|(idx, b)| (b.data.id, idx))
        .collect::<HashMap<_, _>>();

    for block in &mut *blocks {
        match &block.data.exit {
            Exit::Return(_) => {}
            Exit::Goto(dest) => {
                *entry_count.entry(*dest).or_insert(0) += 1;
            }
            Exit::If(_, then_id, else_id) => {
                *entry_count.entry(*then_id).or_insert(0) += 1;
                *entry_count.entry(*else_id).or_insert(0) += 1;
            }
            Exit::Unreachable => {}
        }
    }

    for i in 0..blocks.len() {
        let block = &mut blocks[i];
        if let Exit::Goto(dest) = &block.data.exit {
            if entry_count[dest] == 1 {
                let next_idx = block_map[dest];
                let append_stmts = mem::take(&mut blocks[next_idx].data.stmts);
                let new_exit = mem::replace(&mut blocks[next_idx].data.exit, Exit::Unreachable);
                blocks[i].data.stmts.extend(append_stmts);
                blocks[i].data.exit = new_exit;
                updated = true;
            }
        }
    }
    updated
}

fn remove_unreachable(
    entry_id: BbId,
    blocks: Vec<WithLoc<BasicBlock>>,
) -> Result<Vec<WithLoc<BasicBlock>>, OptimizeError> {
    let mut reachable = HashSet::new();
    let block_map = blocks
        .iter()
        .map(|b| (b.data.id, b))
        .collect::<HashMap<_, _>>();

    let mut visit_list = vec![entry_id];
    while let Some(id) = visit_list.pop() {
        if reachable.contains(&id) {
            continue;
        }
        reachable.insert(id);

        let block = block_map[&id];
        match &block.data.exit {
            Exit::Goto(id) => visit_list.push(*id),
            Exit::If(_cond, then_id, else_id) => {
                visit_list.push(*then_id);
                visit_list.push(*else_id);
            }
            Exit::Return(_) => {}
            Exit::Unreachable => return Err(OptimizeError::NoReturnStatement(block.loc)),
        }
    }

    Ok(blocks
        .into_iter()
        .filter(|b| reachable.contains(&b.data.id))
        .collect())
}
