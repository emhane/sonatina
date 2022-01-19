use cranelift_entity::PrimaryMap;
use fxhash::FxHashMap;

use crate::{
    ir::{module::FuncRef, Function, Module, Signature},
    TargetIsa,
};

use super::FunctionBuilder;

#[derive(Debug)]
pub struct ModuleBuilder {
    pub isa: TargetIsa,

    pub funcs: PrimaryMap<FuncRef, Function>,

    /// Map function name -> FuncRef to avoid duplicated declaration.
    declared_funcs: FxHashMap<String, FuncRef>,
}

impl ModuleBuilder {
    pub fn new(isa: TargetIsa) -> Self {
        Self {
            isa,
            funcs: PrimaryMap::default(),
            declared_funcs: FxHashMap::default(),
        }
    }

    pub fn declare_function(&mut self, sig: Signature) -> FuncRef {
        if self.declared_funcs.contains_key(sig.name()) {
            panic!("{} is already declared.", sig.name())
        } else {
            let func = Function::new(sig);
            self.funcs.push(func)
        }
    }

    pub fn get_func_ref(&mut self, name: &str) -> Option<FuncRef> {
        self.declared_funcs.get(name).copied()
    }

    pub fn func_builder(&mut self, func: FuncRef) -> FunctionBuilder {
        FunctionBuilder::new(self, func)
    }

    pub fn build(self) -> Module {
        Module {
            isa: self.isa,
            funcs: self.funcs,
        }
    }
}
