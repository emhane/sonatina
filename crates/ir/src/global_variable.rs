use std::fmt;

use cranelift_entity::PrimaryMap;
use rustc_hash::FxHashMap;

use crate::{
    ir_writer::{DisplayWithFunc, DisplayableWithFunc},
    Immediate, Linkage, Type,
};

#[derive(Debug, Default)]
pub struct GlobalVariableStore {
    gv_data: PrimaryMap<GlobalVariable, GlobalVariableData>,
    symbols: FxHashMap<String, GlobalVariable>,
}

impl GlobalVariableStore {
    pub fn make_gv(&mut self, gv_data: GlobalVariableData) -> GlobalVariable {
        match self.symbols.entry(gv_data.symbol.to_string()) {
            std::collections::hash_map::Entry::Occupied(_) => {
                panic!("duplicate global symbol `{}`", gv_data.symbol);
            }
            std::collections::hash_map::Entry::Vacant(v) => {
                let gv = self.gv_data.push(gv_data);
                v.insert(gv);
                gv
            }
        }
    }

    pub fn gv_data(&self, gv: GlobalVariable) -> &GlobalVariableData {
        &self.gv_data[gv]
    }

    pub fn gv_by_symbol(&self, symbol: &str) -> Option<GlobalVariable> {
        self.symbols.get(symbol).copied()
    }

    pub fn init_data(&self, gv: GlobalVariable) -> Option<&ConstantValue> {
        self.gv_data[gv].data.as_ref()
    }

    pub fn is_const(&self, gv: GlobalVariable) -> bool {
        self.gv_data[gv].is_const
    }

    pub fn ty(&self, gv: GlobalVariable) -> Type {
        self.gv_data[gv].ty
    }

    pub fn all_gv_data(&self) -> impl Iterator<Item = &GlobalVariableData> {
        self.gv_data.values()
    }
}

/// An opaque reference to [`GlobalVariableData`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Copy, Hash)]
pub struct GlobalVariable(pub u32);
cranelift_entity::entity_impl!(GlobalVariable);

impl DisplayWithFunc for GlobalVariable {
    fn fmt(&self, func: &crate::Function, formatter: &mut fmt::Formatter) -> fmt::Result {
        func.dfg.ctx.with_gv_store(|s| {
            let gv_data = DisplayableWithFunc(s.gv_data(self.gv), func);
            write!(formatter, "{gv_data}")
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GlobalVariableData {
    pub symbol: String,
    pub ty: Type,
    pub linkage: Linkage,
    pub is_const: bool,
    pub data: Option<ConstantValue>,
}

impl GlobalVariableData {
    pub fn new(
        symbol: String,
        ty: Type,
        linkage: Linkage,
        is_const: bool,
        data: Option<ConstantValue>,
    ) -> Self {
        Self {
            symbol,
            ty,
            linkage,
            is_const,
            data,
        }
    }

    pub fn constant(symbol: String, ty: Type, linkage: Linkage, data: ConstantValue) -> Self {
        Self {
            symbol,
            ty,
            linkage,
            is_const: true,
            data: Some(data),
        }
    }
}

impl DisplayWithFunc for GlobalVariableData {
    fn fmt(&self, func: &crate::Function, formatter: &mut fmt::Formatter) -> fmt::Result {
        let GlobalVariableData {
            symbol: _,
            ty,
            linkage,
            is_const,
            data,
        } = &self.gv_data;

        let ty = DisplayableWithFunc(*ty, func);
        ty.fmt(formatter)?;
        if *is_const {
            write!(formatter, " const")?;
        }
        write!(formatter, " {linkage}")?;
        if let Some(data) = data {
            write!(formatter, " {}", data)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConstantValue {
    Immediate(Immediate),
    Array(Vec<ConstantValue>),
    Struct(Vec<ConstantValue>),
}

impl ConstantValue {
    pub fn make_imm(data: impl Into<Immediate>) -> Self {
        Self::Immediate(data.into())
    }

    pub fn make_array(data: Vec<ConstantValue>) -> Self {
        Self::Array(data)
    }

    pub fn make_struct(data: Vec<ConstantValue>) -> Self {
        Self::Struct(data)
    }
}

impl fmt::Display for ConstantValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Immediate(data) => write!(f, "{}", data),
            Self::Array(data) => {
                write!(f, "[")?;
                for (i, v) in data.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            Self::Struct(data) => {
                write!(f, "{{")?;
                for (i, v) in data.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "}}")
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{builder::test_util::build_test_isa, module::ModuleCtx};

    use super::*;

    #[test]
    fn display_gv() {
        let ctx = ModuleCtx::new(build_test_isa());

        let cv = ConstantValue::make_imm(1618i32);
        let gv = ctx.with_gv_store_mut(|s| {
            s.make_gv(GlobalVariableData::new(
                String::from("foo"),
                Type::I32,
                Linkage::Public,
                true,
                Some(cv),
            ))
        });

        let sig = crate::Signature::defualt();
        let func = crate::Function::new(&ctx, sig);

        let display_gv = DisplayableWithFunc(gv, &func);
        assert_eq!(display_gv.to_string(), "i32 const public 1618");
    }

    #[test]
    fn display_gv_array() {
        let ctx = ModuleCtx::new(build_test_isa());

        let cv0 = ConstantValue::make_imm(8i32);
        let cv1 = ConstantValue::make_imm(4i32);
        let cv2 = ConstantValue::make_imm(2i32);
        let const_arr = ConstantValue::make_array(vec![cv0, cv1, cv2]);
        let ty = ctx.with_ty_store_mut(|s| s.make_array(Type::I32, 3));
        let gv = ctx.with_gv_store_mut(|s| {
            s.make_gv(GlobalVariableData::new(
                String::from("foo"),
                ty,
                Linkage::Private,
                true,
                Some(const_arr),
            ))
        });

        let sig = crate::Signature::defualt();
        let func = crate::Function::new(&ctx, sig);

        let display_gv = DisplayableWithFunc(gv, &func);
        assert_eq!(display_gv.to_string(), "[i32;3] const private [8, 4, 2]");
    }
}
