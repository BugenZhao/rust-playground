use std::any::Any;

use crate::any::BaseImpl;

pub struct Distribution;
pub struct Order;

pub enum Convention {
    Logical,
    Batch,
    Stream,
}

pub trait ConventionMarker: 'static + Sized {
    type Extra: 'static;

    fn value() -> Convention;

    fn make_base_impl(base: &Base<Self>) -> BaseImpl<'_>;
}

pub struct Logical;
impl ConventionMarker for Logical {
    type Extra = NoExtra;

    fn value() -> Convention {
        Convention::Logical
    }

    fn make_base_impl(base: &Base<Self>) -> BaseImpl<'_> {
        BaseImpl::Logical(base)
    }
}

pub struct Batch;
impl ConventionMarker for Batch {
    type Extra = BatchExtra;

    fn value() -> Convention {
        Convention::Batch
    }

    fn make_base_impl(base: &Base<Self>) -> BaseImpl<'_> {
        BaseImpl::Batch(base)
    }
}

pub struct Stream;
impl ConventionMarker for Stream {
    type Extra = StreamExtra;

    fn value() -> Convention {
        Convention::Stream
    }

    fn make_base_impl(base: &Base<Self>) -> BaseImpl<'_> {
        BaseImpl::Stream(base)
    }
}

pub trait LogicalAccess {
    fn id(&self) -> i32;
}

pub trait PhysicalSpecificAccess {
    fn distribution(&self) -> &Distribution;
}
pub trait PhysicalAccess = LogicalAccess + PhysicalSpecificAccess;

pub trait StreamSpecificAccess: PhysicalSpecificAccess {
    fn append_only(&self) -> bool;
}
pub trait StreamAccess = PhysicalAccess + StreamSpecificAccess;

pub trait BatchSpecificAccess: PhysicalSpecificAccess {
    fn order(&self) -> &Order;
}
pub trait BatchAccess = PhysicalAccess + BatchSpecificAccess;

pub trait AllAccess: StreamAccess + BatchAccess {}

pub struct PhysicalInner {
    pub distribution: Distribution,
}

pub struct StreamExtra {
    pub physical: PhysicalInner,
    pub append_only: bool,
}

// Delegate to the physical properties.
impl PhysicalSpecificAccess for StreamExtra {
    fn distribution(&self) -> &Distribution {
        &self.physical.distribution
    }
}

impl StreamSpecificAccess for StreamExtra {
    fn append_only(&self) -> bool {
        self.append_only
    }
}

pub struct BatchExtra {
    pub physical: PhysicalInner,
    pub order: Order,
}

// Delegate to the physical properties.
impl PhysicalSpecificAccess for BatchExtra {
    fn distribution(&self) -> &Distribution {
        &self.physical.distribution
    }
}

impl BatchSpecificAccess for BatchExtra {
    fn order(&self) -> &Order {
        &self.order
    }
}

pub struct NoExtra;

pub struct Base<C: ConventionMarker> {
    pub id: i32,
    pub extra: C::Extra,
}

impl<C: ConventionMarker> Base<C> {
    pub fn extra_as_any(&self) -> &dyn Any {
        &self.extra
    }
}

impl<C: ConventionMarker> LogicalAccess for Base<C> {
    fn id(&self) -> i32 {
        self.id
    }
}

impl<C: ConventionMarker> PhysicalSpecificAccess for Base<C>
where
    C::Extra: PhysicalSpecificAccess,
{
    fn distribution(&self) -> &Distribution {
        self.extra.distribution()
    }
}

impl<C: ConventionMarker> StreamSpecificAccess for Base<C>
where
    C::Extra: StreamSpecificAccess,
{
    fn append_only(&self) -> bool {
        self.extra.append_only()
    }
}

impl<C: ConventionMarker> BatchSpecificAccess for Base<C>
where
    C::Extra: BatchSpecificAccess,
{
    fn order(&self) -> &Order {
        self.extra.order()
    }
}

pub trait PlanNode: 'static {
    type Convention: ConventionMarker;

    fn base(&self) -> &Base<Self::Convention>;
}

impl<P> LogicalAccess for P
where
    P: PlanNode,
{
    fn id(&self) -> i32 {
        self.base().id()
    }
}

impl<P> PhysicalSpecificAccess for P
where
    P: PlanNode,
    <P::Convention as ConventionMarker>::Extra: PhysicalSpecificAccess,
{
    fn distribution(&self) -> &Distribution {
        self.base().distribution()
    }
}

impl<P> StreamSpecificAccess for P
where
    P: PlanNode<Convention = Stream>,
{
    fn append_only(&self) -> bool {
        self.base().append_only()
    }
}

impl<P> BatchSpecificAccess for P
where
    P: PlanNode<Convention = Batch>,
{
    fn order(&self) -> &Order {
        self.base().order()
    }
}
