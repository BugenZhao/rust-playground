#![feature(trait_alias)]

use std::{any::Any, ops::Deref, rc::Rc};

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
    fn distribution(&self) -> bool;
}
trait PhysicalAccess = LogicalAccess + PhysicalSpecificAccess;

trait StreamSpecificAccess: PhysicalSpecificAccess {
    fn append_only(&self) -> bool;
}
trait StreamAccess = PhysicalAccess + StreamSpecificAccess;

trait BatchSpecificAccess: PhysicalSpecificAccess {
    fn order(&self) -> bool;
}
trait BatchAccess = PhysicalAccess + BatchSpecificAccess;

trait AnyAccess: StreamAccess + BatchAccess {}

struct PhysicalExtra {
    distribution: bool,
}

struct StreamExtra {
    physical: PhysicalExtra,
    append_only: bool,
}

// Delegate to the physical properties.
impl PhysicalSpecificAccess for StreamExtra {
    fn distribution(&self) -> bool {
        self.physical.distribution
    }
}

impl StreamSpecificAccess for StreamExtra {
    fn append_only(&self) -> bool {
        self.append_only
    }
}

struct BatchExtra {
    physical: PhysicalExtra,
    order: bool,
}

// Delegate to the physical properties.
impl PhysicalSpecificAccess for BatchExtra {
    fn distribution(&self) -> bool {
        self.physical.distribution
    }
}

impl BatchSpecificAccess for BatchExtra {
    fn order(&self) -> bool {
        self.order
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
    fn distribution(&self) -> bool {
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
    fn order(&self) -> bool {
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
    fn distribution(&self) -> bool {
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
    fn order(&self) -> bool {
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
        Rc::new(AnyBase(PlanNode::base(self)))
    }
}

struct AnyBase<'a, C: ConventionMarker>(&'a Base<C>);

impl<C: ConventionMarker> LogicalAccess for AnyBase<'_, C> {
    fn id(&self) -> i32 {
        self.0.id()
    }
}

fn physical_any_access<C: ConventionMarker, R>(
    base: &Base<C>,
    f: impl FnOnce(&dyn PhysicalSpecificAccess) -> R,
) -> R {
    let extra = base.extra_as_any();

    if let Some(e) = extra.downcast_ref::<BatchExtra>() {
        f(e)
    } else if let Some(e) = extra.downcast_ref::<StreamExtra>() {
        f(e)
    } else {
        panic!("accessing physical properties on logical plan node")
    }
}

fn stream_any_access<C: ConventionMarker, R>(
    base: &Base<C>,
    f: impl FnOnce(&dyn StreamSpecificAccess) -> R,
) -> R {
    let extra = base.extra_as_any();

    if let Some(e) = extra.downcast_ref::<StreamExtra>() {
        f(e)
    } else {
        panic!("accessing stream properties on non-stream plan node")
    }
}

fn batch_any_access<C: ConventionMarker, R>(
    base: &Base<C>,
    f: impl FnOnce(&dyn BatchSpecificAccess) -> R,
) -> R {
    let extra = base.extra_as_any();

    if let Some(e) = extra.downcast_ref::<BatchExtra>() {
        f(e)
    } else {
        panic!("accessing batch properties on non-batch plan node")
    }
}

impl<C: ConventionMarker> PhysicalSpecificAccess for AnyBase<'_, C> {
    fn distribution(&self) -> bool {
        physical_any_access(self.0, |e| e.distribution())
    }
}
impl<C: ConventionMarker> StreamSpecificAccess for AnyBase<'_, C> {
    fn append_only(&self) -> bool {
        stream_any_access(self.0, |e| e.append_only())
    }
}
impl<C: ConventionMarker> BatchSpecificAccess for AnyBase<'_, C> {
    fn order(&self) -> bool {
        batch_any_access(self.0, |e| e.order())
    }
}
impl<C: ConventionMarker> AnyAccess for AnyBase<'_, C> {}

fn assert_access_object_safe(_: &dyn AnyAccess) {}
fn assert_plan_node_object_safe(_: &dyn AnyPlanNode) {}

struct PlanRef(Rc<dyn AnyPlanNode>);

impl PlanRef {
    fn new<P>(plan: P) -> Self
    where
        P: AnyPlanNode,
    {
        Self(Rc::new(plan))
    }
}

impl Deref for PlanRef {
    type Target = dyn AnyPlanNode;

    fn deref(&self) -> &dyn AnyPlanNode {
        &*self.0
    }
}

impl LogicalAccess for PlanRef {
    fn id(&self) -> i32 {
        self.0.any_base().id()
    }
}

impl PhysicalSpecificAccess for PlanRef {
    fn distribution(&self) -> bool {
        self.0.any_base().distribution()
    }
}

impl StreamSpecificAccess for PlanRef {
    fn append_only(&self) -> bool {
        self.0.any_base().append_only()
    }
}

impl BatchSpecificAccess for PlanRef {
    fn order(&self) -> bool {
        self.0.any_base().order()
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
                            distribution: false,
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
    let sf_any = PlanRef::new(sf);

    sf_any.id(); // through `LogicalAccess` on `AnyBase` then `LogicalAccess` on `Base<Stream>`
    sf_any.append_only(); // through `StreamSpecificAccess` on `AnyBase` then `StreamSpecificAccess` on `Base<Stream>`
    sf_any.convention(); // through `Deref` into `AnyPlanNode` then `PlanNode`

    // Compiles, but panics at runtime
    {
        sf_any.order(); // `AnyBase` is not `BatchAccess`
        sf_any.any_base().order(); // same as above, just de-delegating
    }
}
