use crate::common::*;
use std::rc::Rc;

use super::{r#impl::BaseImpl, AnyPlanNode};

pub type DynBaseRef<'a> = Rc<dyn AllAccess + 'a>;

pub struct DynBaseAccessor<'a, C: ConventionMarker>(pub &'a Base<C>);

impl<C: ConventionMarker> LogicalAccess for DynBaseAccessor<'_, C> {
    fn id(&self) -> i32 {
        self.0.id()
    }
}

// TODO: lifetime issue if extracting this into a **function**
macro_rules! access_by_downcast {
    ($self:ident, $field:ident, $( $Extra:ident ),+) => {
        loop {
            $(
                if let Some(e) = $self.0.extra_as_any().downcast_ref::<$Extra>() {
                    break Some(e.$field());
                }
            )*;
            #[allow(unreachable_code)]
            break None;
        }
    }
}

impl<C: ConventionMarker> PhysicalSpecificAccess for DynBaseAccessor<'_, C> {
    fn distribution(&self) -> &Distribution {
        access_by_downcast!(self, distribution, StreamExtra, BatchExtra)
            .expect("accessing physical properties on logical plan node")
    }
}
impl<C: ConventionMarker> StreamSpecificAccess for DynBaseAccessor<'_, C> {
    fn append_only(&self) -> bool {
        access_by_downcast!(self, append_only, StreamExtra)
            .expect("accessing stream properties on non-stream plan node")
    }
}
impl<C: ConventionMarker> BatchSpecificAccess for DynBaseAccessor<'_, C> {
    fn order(&self) -> &Order {
        access_by_downcast!(self, order, BatchExtra)
            .expect("accessing batch properties on non-batch plan node")
    }
}
impl<C: ConventionMarker> AllAccess for DynBaseAccessor<'_, C> {}

fn assert_access_object_safe(_: &dyn AllAccess) {}

#[ouroboros::self_referencing]
pub struct PlanRef {
    plan: Rc<dyn AnyPlanNode>,

    #[borrows(plan)]
    #[covariant]
    base: DynBaseRef<'this>,
}

const _: &[u8; std::mem::size_of::<PlanRef>()] = &[0; 24];

impl Clone for PlanRef {
    fn clone(&self) -> Self {
        Self::new_inner(Rc::clone(self.borrow_plan()))
    }
}

impl PlanRef {
    fn new_inner(plan: Rc<dyn AnyPlanNode>) -> Self {
        Self::new(plan, |plan| plan.dyn_base())
    }

    pub fn make<P>(plan: P) -> Self
    where
        P: AnyPlanNode,
    {
        Self::new_inner(Rc::new(plan))
    }
}

impl AnyPlanNode for PlanRef {
    fn convention(&self) -> Convention {
        self.borrow_plan().convention()
    }

    fn dyn_base(&self) -> DynBaseRef<'_> {
        Rc::clone(self.borrow_base())
    }

    fn base_impl(&self) -> BaseImpl<'_> {
        unimplemented!()
    }
}

impl LogicalAccess for PlanRef {
    fn id(&self) -> i32 {
        self.borrow_base().id()
    }
}

impl PhysicalSpecificAccess for PlanRef {
    fn distribution(&self) -> &Distribution {
        self.borrow_base().distribution()
    }
}

impl StreamSpecificAccess for PlanRef {
    fn append_only(&self) -> bool {
        self.borrow_base().append_only()
    }
}

impl BatchSpecificAccess for PlanRef {
    fn order(&self) -> &Order {
        self.borrow_base().order()
    }
}
