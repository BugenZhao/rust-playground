#![feature(trait_alias)]

use std::{any::Any, ops::Deref, rc::Rc};

struct Distribution;
struct Order;

enum Convention {
    Logical,
    Batch,
    Stream,
}

trait ConventionMarker: 'static {
    type Extra: 'static;

    fn value() -> Convention;
}

struct Logical;
impl ConventionMarker for Logical {
    type Extra = NoExtra;

    fn value() -> Convention {
        Convention::Logical
    }
}

struct Batch;
impl ConventionMarker for Batch {
    type Extra = BatchExtra;

    fn value() -> Convention {
        Convention::Batch
    }
}

struct Stream;
impl ConventionMarker for Stream {
    type Extra = StreamExtra;

    fn value() -> Convention {
        Convention::Stream
    }
}

trait LogicalAccess {
    fn id(&self) -> i32;
}

trait PhysicalSpecificAccess {
    fn distribution(&self) -> &Distribution;
}
trait PhysicalAccess = LogicalAccess + PhysicalSpecificAccess;

trait StreamSpecificAccess: PhysicalSpecificAccess {
    fn append_only(&self) -> bool;
}
trait StreamAccess = PhysicalAccess + StreamSpecificAccess;

trait BatchSpecificAccess: PhysicalSpecificAccess {
    fn order(&self) -> &Order;
}
trait BatchAccess = PhysicalAccess + BatchSpecificAccess;

trait AnyAccess: StreamAccess + BatchAccess {}

struct PhysicalExtra {
    distribution: Distribution,
}

struct StreamExtra {
    physical: PhysicalExtra,
    append_only: bool,
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

struct BatchExtra {
    physical: PhysicalExtra,
    order: Order,
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

struct NoExtra;

struct Base<C: ConventionMarker> {
    id: i32,
    extra: C::Extra,
}

impl<C: ConventionMarker> Base<C> {
    fn extra_as_any(&self) -> &dyn Any {
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

trait PlanNode: 'static {
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

type AnyBaseRef<'a> = Rc<dyn AnyAccess + 'a>;

trait AnyPlanNode: 'static {
    fn convention(&self) -> Convention;

    fn any_base(&self) -> AnyBaseRef<'_>;
}

impl<P> AnyPlanNode for P
where
    P: PlanNode,
{
    fn convention(&self) -> Convention {
        <P::Convention as ConventionMarker>::value()
    }

    fn any_base(&self) -> AnyBaseRef<'_> {
        Rc::new(AnyBaseAccessor(PlanNode::base(self)))
    }
}

struct AnyBaseAccessor<'a, C: ConventionMarker>(&'a Base<C>);

impl<C: ConventionMarker> LogicalAccess for AnyBaseAccessor<'_, C> {
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

impl<C: ConventionMarker> PhysicalSpecificAccess for AnyBaseAccessor<'_, C> {
    fn distribution(&self) -> &Distribution {
        access_by_downcast!(self, distribution, StreamExtra, BatchExtra)
            .expect("accessing physical properties on logical plan node")
    }
}
impl<C: ConventionMarker> StreamSpecificAccess for AnyBaseAccessor<'_, C> {
    fn append_only(&self) -> bool {
        access_by_downcast!(self, append_only, StreamExtra)
            .expect("accessing stream properties on non-stream plan node")
    }
}
impl<C: ConventionMarker> BatchSpecificAccess for AnyBaseAccessor<'_, C> {
    fn order(&self) -> &Order {
        access_by_downcast!(self, order, BatchExtra)
            .expect("accessing batch properties on non-batch plan node")
    }
}
impl<C: ConventionMarker> AnyAccess for AnyBaseAccessor<'_, C> {}

fn assert_access_object_safe(_: &dyn AnyAccess) {}
fn assert_plan_node_object_safe(_: &dyn AnyPlanNode) {}

#[ouroboros::self_referencing]
struct PlanRef {
    plan: Rc<dyn AnyPlanNode>,

    #[borrows(plan)]
    #[covariant]
    base: AnyBaseRef<'this>,
}

const _: &[u8; std::mem::size_of::<PlanRef>()] = &[0; 24];

impl Clone for PlanRef {
    fn clone(&self) -> Self {
        Self::new_inner(Rc::clone(self.borrow_plan()))
    }
}

impl PlanRef {
    fn new_inner(plan: Rc<dyn AnyPlanNode>) -> Self {
        Self::new(plan, |plan| plan.any_base())
    }

    fn make<P>(plan: P) -> Self
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

    fn any_base(&self) -> AnyBaseRef<'_> {
        Rc::clone(self.borrow_base())
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

#[allow(unused_variables)]
fn main() {
    struct Filter;

    struct LogicalFilter {
        base: Base<Logical>,
    }

    impl LogicalFilter {
        fn new() -> Self {
            Self {
                base: Base {
                    id: 0,
                    extra: NoExtra,
                },
            }
        }
    }

    impl PlanNode for LogicalFilter {
        type Convention = Logical;

        fn base(&self) -> &Base<Self::Convention> {
            &self.base
        }
    }

    struct StreamFilter {
        base: Base<Stream>,
        additional: (),
    }

    impl StreamFilter {
        fn new() -> Self {
            Self {
                base: Base {
                    id: 0,
                    extra: StreamExtra {
                        physical: PhysicalExtra {
                            distribution: Distribution,
                        },
                        append_only: false,
                    },
                },
                additional: (),
            }
        }
    }

    impl PlanNode for StreamFilter {
        type Convention = Stream;

        fn base(&self) -> &Base<Self::Convention> {
            &self.base
        }
    }

    // -- Concrete plan node: with compile-time convention check
    let sf = StreamFilter::new();

    sf.id(); // through `LogicalAccess` on `Base<Stream>`
    sf.append_only(); // through `StreamSpecificAccess` on `Base<Stream>`
    sf.convention(); // through `AnyPlanNode` then `PlanNode`

    #[cfg(fail)]
    {
        // doesn't satisfy `<StreamFilter as PlanNode>::Convention = Batch`
        sf.order();
        // Not satisfied: `<&Base<Stream> as PlanNode>::Convention = Batch`
        sf.base().order();
    }

    // -- Similar for logical node trying to access physical properties
    let lf = LogicalFilter::new();

    lf.id(); // through `LogicalAccess` on `Base<Logical>`

    #[cfg(fail)]
    {
        // doesn't satisfy `LogicalFilter: PhysicalSpecificAccess`
        lf.distribution();
        // trait bound `<LogicalFilter as PlanNode>::Convention = Stream` was not satisfied
        lf.append_only();
        // trait bound `<LogicalFilter as PlanNode>::Convention = Batch` was not satisfied
        lf.order();
    }

    // -- Type-erased plan node: with runtime convention check
    let sf_any = PlanRef::make(sf);

    sf_any.id(); // through `LogicalAccess` on `AnyBase` then `LogicalAccess` on `Base<Stream>`
    sf_any.append_only(); // through `StreamSpecificAccess` on `AnyBase` then `StreamSpecificAccess` on `Base<Stream>`
    sf_any.convention(); // through `Deref` into `AnyPlanNode` then `PlanNode`

    // Compiles, but panics at runtime
    {
        sf_any.order(); // `AnyBase` is not `BatchAccess`
        sf_any.any_base().order(); // same as above, just desugared
    }
}
