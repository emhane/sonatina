pub mod arith;
pub mod cast;
pub mod cmp;
pub mod control_flow;
pub mod data;
pub mod evm;
pub mod inst_set;
pub mod logic;

use std::any::{Any, TypeId};

use smallvec::SmallVec;

use crate::{InstSetBase, Value};

pub trait Inst: inst_set::sealed::Registered + Any {
    fn visit_values(&self, f: &mut dyn Fn(Value));
    fn visit_values_mut(&mut self, f: &mut dyn Fn(&mut Value));
    fn has_side_effect(&self) -> bool;
    fn as_text(&self) -> &'static str;
    fn is_terminator(&self) -> bool;
}

/// This trait works as a "proof" that a specific ISA contains `I`,
/// and then allows a construction and reflection of type `I` in that specific ISA context.
pub trait HasInst<I: Inst> {
    fn is(&self, inst: &dyn Inst) -> bool {
        inst.type_id() == TypeId::of::<I>()
    }
}

pub(crate) trait ValueVisitable {
    fn visit_with(&self, f: &mut dyn Fn(Value));
    fn visit_mut_with(&mut self, f: &mut dyn Fn(&mut Value));
}

impl ValueVisitable for Value {
    fn visit_with(&self, f: &mut dyn Fn(Value)) {
        f(*self)
    }

    fn visit_mut_with(&mut self, f: &mut dyn Fn(&mut Value)) {
        f(self)
    }
}

impl<V> ValueVisitable for Option<V>
where
    V: ValueVisitable,
{
    fn visit_with(&self, f: &mut dyn Fn(Value)) {
        if let Some(value) = self {
            value.visit_with(f)
        }
    }

    fn visit_mut_with(&mut self, f: &mut dyn Fn(&mut Value)) {
        if let Some(value) = self.as_mut() {
            value.visit_mut_with(f)
        }
    }
}

impl<V, T> ValueVisitable for (V, T)
where
    V: ValueVisitable,
{
    fn visit_with(&self, f: &mut dyn Fn(Value)) {
        self.0.visit_with(f)
    }

    fn visit_mut_with(&mut self, f: &mut dyn Fn(&mut Value)) {
        self.0.visit_mut_with(f)
    }
}

impl<V> ValueVisitable for Vec<V>
where
    V: ValueVisitable,
{
    fn visit_with(&self, f: &mut dyn Fn(Value)) {
        self.iter().for_each(|v| v.visit_with(f))
    }

    fn visit_mut_with(&mut self, f: &mut dyn Fn(&mut Value)) {
        self.iter_mut().for_each(|v| v.visit_mut_with(f))
    }
}

impl<V> ValueVisitable for [V]
where
    V: ValueVisitable,
{
    fn visit_with(&self, f: &mut dyn Fn(Value)) {
        self.iter().for_each(|v| v.visit_with(f))
    }

    fn visit_mut_with(&mut self, f: &mut dyn Fn(&mut Value)) {
        self.iter_mut().for_each(|v| v.visit_mut_with(f))
    }
}

impl<V, const N: usize> ValueVisitable for SmallVec<[V; N]>
where
    V: ValueVisitable,
    [V; N]: smallvec::Array<Item = V>,
{
    fn visit_with(&self, f: &mut dyn Fn(Value)) {
        self.iter().for_each(|v| v.visit_with(f))
    }

    fn visit_mut_with(&mut self, f: &mut dyn Fn(&mut Value)) {
        self.iter_mut().for_each(|v| v.visit_mut_with(f))
    }
}

pub trait InstCast: Inst + Sized {
    fn downcast<'i>(is: &dyn InstSetBase, inst: &'i dyn Inst) -> Option<&'i Self>;
    fn downcast_mut<'i>(is: &dyn InstSetBase, inst: &'i mut dyn Inst) -> Option<&'i mut Self>;

    fn upcast(self) -> Box<dyn Inst> {
        Box::new(self)
    }

    fn map<'i, F, R>(is: &dyn InstSetBase, inst: &'i dyn Inst, f: F) -> Option<R>
    where
        F: Fn(&'i Self) -> R,
    {
        let data = Self::downcast(is, inst)?;
        Some(f(data))
    }

    fn map_mut<'i, F, R>(is: &dyn InstSetBase, inst: &'i mut dyn Inst, f: F) -> Option<R>
    where
        F: Fn(&'i mut Self) -> R,
    {
        let data = Self::downcast_mut(is, inst)?;
        Some(f(data))
    }
}
