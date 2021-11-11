use smallvec::SmallVec;

use super::{DataFlowGraph, Layout, Type, Value};

#[derive(Debug, Clone)]
pub struct Function {
    /// Name of the function.
    pub name: String,

    /// Signature of the function.
    pub sig: Signature,
    pub arg_values: smallvec::SmallVec<[Value; 8]>,

    pub dfg: DataFlowGraph,
    pub layout: Layout,
}

impl Function {
    pub fn new(name: String, sig: Signature) -> Self {
        let mut dfg = DataFlowGraph::default();
        let arg_values = sig
            .args()
            .iter()
            .enumerate()
            .map(|(idx, arg_ty)| {
                let value = dfg.make_arg_value(arg_ty, idx);
                dfg.make_value(value)
            })
            .collect();

        Self {
            name,
            sig,
            arg_values,
            dfg,
            layout: Layout::default(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Signature {
    args: SmallVec<[Type; 8]>,
    rets: SmallVec<[Type; 8]>,
}

impl Signature {
    pub fn new(args: &[Type], rets: &[Type]) -> Self {
        Self {
            args: args.into(),
            rets: rets.into(),
        }
    }

    pub fn append_arg(&mut self, arg: Type) {
        self.args.push(arg);
    }

    pub fn append_return(&mut self, ret: Type) {
        self.rets.push(ret);
    }

    pub fn args(&self) -> &[Type] {
        &self.args
    }

    pub fn returns(&self) -> &[Type] {
        &self.rets
    }
}
