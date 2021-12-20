use super::*;

impl CfgSubroutine {
    pub(crate) fn update_bb_links(&mut self) {
        self.block_index_map.clear();
        self.block_index_map.extend(
            self.blocks
                .iter()
                .enumerate()
                .map(|(i, bb)| (bb.data.id, i)),
        );

        let mut src_map = HashMap::<BbId, Vec<BbId>>::new();
        for bb in &self.blocks {
            match &bb.data.exit {
                Exit::Return(_) => {}
                Exit::Goto(id) => src_map.entry(*id).or_default().push(bb.data.id),
                Exit::If(_, then_id, else_id) => {
                    src_map.entry(*then_id).or_default().push(bb.data.id);
                    src_map.entry(*else_id).or_default().push(bb.data.id);
                }
                Exit::Unreachable => {}
            }
        }

        for bb in &mut self.blocks {
            bb.data.src_ids.clear();
            bb.data
                .src_ids
                .extend(src_map.remove(&bb.data.id).unwrap_or_default());
        }
    }
}
