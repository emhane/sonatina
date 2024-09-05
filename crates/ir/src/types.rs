//! This module contains Sonatina IR types definitions.
use std::{cmp, fmt};

use cranelift_entity::{EntityRef, PrimaryMap};
use indexmap::IndexMap;
use rustc_hash::FxHashMap;

use crate::DataFlowGraph;

#[derive(Debug, Default)]
pub struct TypeStore {
    compounds: PrimaryMap<CompoundType, CompoundTypeData>,
    rev_types: FxHashMap<CompoundTypeData, CompoundType>,
    struct_types: IndexMap<String, CompoundType>,
}

impl TypeStore {
    pub fn make_ptr(&mut self, ty: Type) -> Type {
        let ty = self.make_compound(CompoundTypeData::Ptr(ty));
        Type::Compound(ty)
    }

    pub fn make_array(&mut self, elem: Type, len: usize) -> Type {
        let ty = self.make_compound(CompoundTypeData::Array { elem, len });
        Type::Compound(ty)
    }

    pub fn make_struct(&mut self, name: &str, fields: &[Type], packed: bool) -> Type {
        let compound_data = CompoundTypeData::Struct(StructData {
            name: name.to_string(),
            fields: fields.to_vec(),
            packed,
        });
        let compound = self.make_compound(compound_data);
        debug_assert!(
            !self.struct_types.contains_key(name),
            "struct {name} is already defined"
        );
        self.struct_types.insert(name.to_string(), compound);
        Type::Compound(compound)
    }

    /// Returns `[StructDef]` if the given type is a struct type.
    pub fn struct_def(&self, ty: Type) -> Option<&StructData> {
        match ty {
            Type::Compound(compound) => match self.compounds[compound] {
                CompoundTypeData::Struct(ref def) => Some(def),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn array_def(&self, ty: Type) -> Option<(Type, usize)> {
        match ty {
            Type::Compound(compound) => match self.compounds[compound] {
                CompoundTypeData::Array { elem, len } => Some((elem, len)),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn struct_type_by_name(&self, name: &str) -> Option<Type> {
        self.struct_types.get(name).map(|ty| Type::Compound(*ty))
    }

    pub fn all_struct_data(&self) -> impl Iterator<Item = &StructData> {
        self.struct_types
            .values()
            .map(|compound_type| match self.compounds[*compound_type] {
                CompoundTypeData::Struct(ref def) => def,
                _ => unreachable!(),
            })
    }

    pub fn deref(&self, ptr: Type) -> Option<Type> {
        match ptr {
            Type::Compound(ty) => {
                let ty_data = &self.compounds[ty];
                match ty_data {
                    CompoundTypeData::Ptr(ty) => Some(*ty),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    pub fn is_integral(&self, ty: Type) -> bool {
        ty.is_integral()
    }

    pub fn is_ptr(&self, ty: Type) -> bool {
        match ty {
            Type::Compound(compound) => self.compounds[compound].is_ptr(),
            _ => false,
        }
    }

    pub fn is_array(&self, ty: Type) -> bool {
        match ty {
            Type::Compound(compound) => self.compounds[compound].is_array(),
            _ => false,
        }
    }

    pub fn make_compound(&mut self, data: CompoundTypeData) -> CompoundType {
        if let Some(compound) = self.rev_types.get(&data) {
            *compound
        } else {
            let compound = self.compounds.push(data.clone());
            self.rev_types.insert(data, compound);
            compound
        }
    }

    pub fn resolve_compound(&self, compound: CompoundType) -> &CompoundTypeData {
        &self.compounds[compound]
    }
}

/// Sonatina IR types definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Type {
    I1,
    I8,
    I16,
    I32,
    I64,
    I128,
    I256,
    Compound(CompoundType),
    #[default]
    Void,
}

impl EntityRef for Type {
    fn new(i: usize) -> Self {
        if i == 0 {
            Type::Void
        } else if i == 1 {
            Type::I1
        } else if i == 8 {
            Type::I8
        } else if i == 16 {
            Type::I16
        } else if i == 32 {
            Type::I32
        } else if i == 64 {
            Type::I64
        } else if i == 128 {
            Type::I128
        } else if i == 256 {
            Type::I256
        } else if i > 256 {
            Type::Compound(CompoundType::new(i - 256))
        } else {
            unreachable!()
        }
    }

    fn index(self) -> usize {
        match self {
            Type::Void => 0,
            Type::I1 => 1,
            Type::I8 => 8,
            Type::I16 => 16,
            Type::I32 => 32,
            Type::I64 => 64,
            Type::I128 => 128,
            Type::I256 => 256,
            Type::Compound(cmpd_ty) => 256 + cmpd_ty.index(),
        }
    }
}

/// An opaque reference to [`CompoundTypeData`].
#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash, PartialOrd, Ord)]
pub struct CompoundType(u32);
cranelift_entity::entity_impl!(CompoundType);

struct DisplayCompoundType<'a> {
    cmpd_ty: CompoundType,
    dfg: &'a DataFlowGraph,
}

impl<'a> fmt::Display for DisplayCompoundType<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use CompoundTypeData::*;
        let dfg = self.dfg;
        dfg.ctx
            .with_ty_store(|s| match s.resolve_compound(self.cmpd_ty) {
                Array { elem: ty, len } => {
                    let ty = DisplayType::new(*ty, dfg);
                    write!(f, "[{ty};{len}]")
                }
                Ptr(ty) => {
                    let ty = DisplayType::new(*ty, dfg);
                    write!(f, "*{ty}")
                }
                Struct(StructData { name, packed, .. }) => {
                    if *packed {
                        write!(f, "<{{{name}}}>")
                    } else {
                        write!(f, "{{{name}}}")
                    }
                }
            })
    }
}

pub struct DisplayType<'a> {
    ty: Type,
    dfg: &'a DataFlowGraph,
}

impl<'a> DisplayType<'a> {
    pub fn new(ty: Type, dfg: &'a DataFlowGraph) -> Self {
        Self { ty, dfg }
    }
}

impl<'a> fmt::Display for DisplayType<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Type::*;
        match self.ty {
            I1 => write!(f, "i1"),
            I8 => write!(f, "i8"),
            I16 => write!(f, "i16"),
            I32 => write!(f, "i32"),
            I64 => write!(f, "i64"),
            I128 => write!(f, "i128"),
            I256 => write!(f, "i256"),
            Compound(cmpd_ty) => {
                let dfg = self.dfg;
                write!(f, "{}", DisplayCompoundType { cmpd_ty, dfg })
            }
            Void => write!(f, "()"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CompoundTypeData {
    Array { elem: Type, len: usize },
    Ptr(Type),
    Struct(StructData),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StructData {
    pub name: String,
    pub fields: Vec<Type>,
    pub packed: bool,
}

impl CompoundTypeData {
    pub fn is_array(&self) -> bool {
        matches!(self, Self::Array { .. })
    }

    pub fn is_ptr(&self) -> bool {
        matches!(self, Self::Ptr(_))
    }
}

impl Type {
    pub fn is_integral(&self) -> bool {
        matches!(
            self,
            Self::I1 | Self::I8 | Self::I16 | Self::I32 | Self::I64 | Self::I128 | Self::I256
        )
    }

    pub fn to_string(&self, dfg: &DataFlowGraph) -> String {
        DisplayType { ty: *self, dfg }.to_string()
    }
}

impl cmp::PartialOrd for Type {
    fn partial_cmp(&self, rhs: &Self) -> Option<cmp::Ordering> {
        use Type::*;

        if self == rhs {
            return Some(cmp::Ordering::Equal);
        }

        if !self.is_integral() || !rhs.is_integral() {
            return None;
        }

        match (self, rhs) {
            (I1, _) => Some(cmp::Ordering::Less),
            (I8, I1) => Some(cmp::Ordering::Greater),
            (I8, _) => Some(cmp::Ordering::Less),
            (I16, I1 | I8) => Some(cmp::Ordering::Greater),
            (I16, _) => Some(cmp::Ordering::Less),
            (I32, I1 | I8 | I16) => Some(cmp::Ordering::Greater),
            (I32, _) => Some(cmp::Ordering::Less),
            (I64, I128 | I256) => Some(cmp::Ordering::Less),
            (I64, _) => Some(cmp::Ordering::Greater),
            (I128, I256) => Some(cmp::Ordering::Less),
            (I128, _) => Some(cmp::Ordering::Greater),
            (I256, _) => Some(cmp::Ordering::Greater),
            (_, _) => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use cranelift_entity::SecondaryMap;

    use super::*;

    #[test]
    fn type_as_entity() {
        let mut map = SecondaryMap::new();
        assert_eq!(map.capacity(), 0);

        map[Type::I1] = 1;
        map[Type::I32] = 32;

        let cmpd_ty = CompoundType(1);
        map[Type::Compound(cmpd_ty)] = 257;

        assert_eq!(map[Type::Void], 0);
        assert_eq!(map[Type::I32], 32);
        assert_eq!(map[Type::Compound(cmpd_ty)], 257);
    }
}
