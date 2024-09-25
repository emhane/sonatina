use std::fmt::Write;

use dot2::label;

use crate::{
    ir_writer::{DisplayableWithFunc, ValueWithTy},
    BlockId, ControlFlowGraph, Function,
};

use super::function::DUMMY_BLOCK;

#[derive(Clone, Copy)]
pub(super) struct BlockNode<'a> {
    pub(super) func: &'a Function,
    pub(super) cfg: &'a ControlFlowGraph,
    pub(super) block: BlockId,
}

impl<'a> BlockNode<'a> {
    pub(super) fn new(func: &'a Function, cfg: &'a ControlFlowGraph, block: BlockId) -> Self {
        Self { func, cfg, block }
    }

    pub(super) fn succs(self) -> Vec<Self> {
        self.cfg
            .succs_of(self.block)
            .map(|block| BlockNode::new(self.func, self.cfg, *block))
            .collect()
    }
}

impl<'a> BlockNode<'a> {
    pub(super) fn label(self) -> label::Text<'static> {
        let Self { block, func, .. } = self;
        let Function { sig, layout, .. } = func;
        if block == DUMMY_BLOCK {
            let sig = DisplayableWithFunc(sig, &self.func);
            return label::Text::LabelStr(format!("{sig}").into());
        }

        let mut label = r#"<table border="0" cellborder="1" cellspacing="0">"#.to_string();

        // Write block header.
        write!(
            &mut label,
            r#"<tr><td bgcolor="gray" align="center" colspan="1">{}</td></tr>"#,
            block
        )
        .unwrap();

        // Write block body.
        write!(label, r#"<tr><td align="left" balign="left">"#).unwrap();
        for inst in layout.iter_inst(self.block) {
            let mut inst_string = String::new();
            if let Some(result) = self.func.dfg.inst_result(inst) {
                let result_with_ty = ValueWithTy(result);
                write!(
                    &mut inst_string,
                    "{} = ",
                    DisplayableWithFunc(result_with_ty, self.func)
                )
                .unwrap();
            }
            let inst = DisplayableWithFunc(inst, self.func);
            write!(&mut inst_string, "{inst};").unwrap();

            write!(label, "{}", dot2::escape_html(&inst_string)).unwrap();
            write!(label, "<br/>").unwrap();
        }
        write!(label, r#"</td></tr>"#).unwrap();

        write!(label, "</table>").unwrap();

        label::Text::HtmlStr(label.into())
    }
}
