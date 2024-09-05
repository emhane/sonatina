use std::{
    fmt,
    sync::{Arc, RwLock},
};

use cranelift_entity::{entity_impl, PrimaryMap};

use crate::Function;

use crate::{global_variable::GlobalVariableStore, isa::TargetIsa, types::TypeStore};

use super::Linkage;

#[derive(Debug)]
pub struct Module {
    /// Holds all function declared in the contract.
    pub funcs: PrimaryMap<FuncRef, Function>,

    pub ctx: ModuleCtx,
}

impl Module {
    #[doc(hidden)]
    pub fn new(isa: TargetIsa) -> Self {
        Self {
            funcs: PrimaryMap::default(),
            ctx: ModuleCtx::new(isa),
        }
    }

    /// Returns `func_ref` in the module.
    pub fn iter_functions(&self) -> impl Iterator<Item = FuncRef> {
        self.funcs.keys()
    }

    /// Returns `true` if the function has external linkage.
    pub fn is_external(&self, func_ref: FuncRef) -> bool {
        self.funcs[func_ref].sig.linkage() == Linkage::External
    }
}

#[derive(Debug, Clone)]
pub struct ModuleCtx {
    pub isa: TargetIsa,
    type_store: Arc<RwLock<TypeStore>>,
    gv_store: Arc<RwLock<GlobalVariableStore>>,
}

impl ModuleCtx {
    pub fn new(isa: TargetIsa) -> Self {
        Self {
            isa,
            type_store: Arc::new(RwLock::new(TypeStore::default())),
            gv_store: Arc::new(RwLock::new(GlobalVariableStore::default())),
        }
    }

    pub fn with_ty_store<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&TypeStore) -> R,
    {
        f(&self.type_store.read().unwrap())
    }

    pub fn with_ty_store_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut TypeStore) -> R,
    {
        f(&mut self.type_store.write().unwrap())
    }

    pub fn with_gv_store<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&GlobalVariableStore) -> R,
    {
        f(&self.gv_store.read().unwrap())
    }

    pub fn with_gv_store_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut GlobalVariableStore) -> R,
    {
        f(&mut self.gv_store.write().unwrap())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FuncRef(u32);
entity_impl!(FuncRef, "%");

pub struct DisplayCalleeFuncRef<'a> {
    callee: FuncRef,
    func: &'a Function,
}

impl<'a> DisplayCalleeFuncRef<'a> {
    pub fn new(callee: FuncRef, func: &'a Function) -> Self {
        Self { callee, func }
    }
}

impl<'a> fmt::Display for DisplayCalleeFuncRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self { callee, func } = *self;
        let sig = func.callees.get(&callee).unwrap();
        write!(f, "{}", sig.name())
    }
}
