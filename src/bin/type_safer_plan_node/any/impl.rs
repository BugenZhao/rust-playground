use std::rc::Rc;

use crate::common::*;

use super::{AnyPlanNode, DynBaseRef};

#[derive(enum_as_inner::EnumAsInner)]
pub enum BaseImpl<'a> {
    Logical(&'a Base<Logical>),
    Stream(&'a Base<Stream>),
    Batch(&'a Base<Batch>),
}

#[derive(Clone)]
pub struct PlanRef(Rc<dyn AnyPlanNode>);

impl PlanRef {
    pub fn make<P>(plan: P) -> Self
    where
        P: AnyPlanNode,
    {
        Self(Rc::new(plan))
    }
}

impl AnyPlanNode for PlanRef {
    fn convention(&self) -> Convention {
        self.0.convention()
    }

    fn dyn_base(&self) -> DynBaseRef<'_> {
        unimplemented!()
    }

    fn base_impl(&self) -> BaseImpl<'_> {
        self.0.base_impl()
    }
}

impl LogicalAccess for PlanRef {
    fn id(&self) -> i32 {
        (self.0.base_impl().as_logical().map(|b| b.id()))
            .or_else(|| self.0.base_impl().as_stream().map(|s| s.id()))
            .or_else(|| self.0.base_impl().as_batch().map(|l| l.id()))
            .expect("accessing logical properties on physical plan node")
    }
}

impl PhysicalSpecificAccess for PlanRef {
    fn distribution(&self) -> &Distribution {
        (self.0.base_impl().as_stream().map(|b| b.distribution()))
            .or_else(|| self.0.base_impl().as_batch().map(|s| s.distribution()))
            .expect("accessing physical properties on logical plan node")
    }
}

impl StreamSpecificAccess for PlanRef {
    fn append_only(&self) -> bool {
        (self.0.base_impl().as_stream().map(|s| s.append_only()))
            .expect("accessing stream properties on non-stream plan node")
    }
}

impl BatchSpecificAccess for PlanRef {
    fn order(&self) -> &Order {
        (self.0.base_impl().as_batch().map(|b| b.order()))
            .expect("accessing batch properties on non-batch plan node")
    }
}
