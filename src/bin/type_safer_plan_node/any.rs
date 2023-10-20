mod r#dyn;
mod r#impl;

use std::rc::Rc;

use self::r#dyn::DynBaseAccessor;
use crate::common::*;

pub use self::r#dyn::{DynBaseRef, PlanRef as DynPlanRef};
pub use self::r#impl::{BaseImpl, PlanRef as PlanImplRef};

pub trait AnyPlanNode: 'static {
    fn convention(&self) -> Convention;

    fn dyn_base(&self) -> DynBaseRef<'_>;

    fn base_impl(&self) -> BaseImpl<'_>;
}

impl<P> AnyPlanNode for P
where
    P: PlanNode,
{
    fn convention(&self) -> Convention {
        <P::Convention as ConventionMarker>::value()
    }

    fn dyn_base(&self) -> DynBaseRef<'_> {
        Rc::new(DynBaseAccessor(PlanNode::base(self)))
    }

    fn base_impl(&self) -> BaseImpl<'_> {
        <P::Convention as ConventionMarker>::make_base_impl(self.base())
    }
}

fn assert_plan_node_object_safe(_: &dyn AnyPlanNode) {}
