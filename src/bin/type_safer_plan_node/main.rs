#![feature(trait_alias)]

use any::*;
use common::*;

pub mod any;
pub mod common;

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
                    physical: PhysicalInner {
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

fn main() {
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
    let sf_any = DynPlanRef::make(sf);

    sf_any.id(); // through `LogicalAccess` on `DynBase` then `LogicalAccess` on `Base<Stream>`
    sf_any.append_only(); // through `StreamSpecificAccess` on `DynBase` then `StreamSpecificAccess` on `Base<Stream>`
    sf_any.convention(); // through `AnyPlanNode` then `PlanNode`

    // Compiles, but panics at runtime
    {
        sf_any.order(); // `AnyBase` is not `BatchAccess`
        sf_any.dyn_base().order(); // same as above, just desugared
    }

    // -- Type-erased plan node: with runtime convention check
    let sf_any = PlanImplRef::make(StreamFilter::new());

    sf_any.id(); // through `LogicalAccess` on all variants of `Base` then `LogicalAccess` on `Base<Stream>`
    sf_any.append_only(); // through `StreamSpecificAccess` on all variants of `Base` then `StreamSpecificAccess` on `Base<Stream>`
    sf_any.convention(); // through `AnyPlanNode` then `PlanNode`

    // Compiles, but panics at runtime
    {
        sf_any.order(); // `BaseImpl` is not in the variant of `Batch` convention
        sf_any.base_impl().as_batch().unwrap().order(); // same as above, just desugared
    }
}
