mod constraint;
mod context;
mod gather;
mod instructions;
mod priv_prelude;
mod solver;

use std::{collections::BTreeSet, sync::RwLock};

use hashbrown::HashMap;

use crate::{engine_threading::*, language::ty, CompileResult};

use priv_prelude::*;

pub(super) fn monomorphize(engines: Engines<'_>, module: &mut ty::TyModule) -> CompileResult<()> {
    CompileResult::with_handler(|h| {
        let root_namespace = Namespace::init_root(&module.namespace);
        let constraints = RwLock::new(HashMap::new());
        let ctx = Context::from_root(&root_namespace, engines, &constraints);

        gather_constraints(ctx, h, module)?;

        let constraints = order_constraints(engines, constraints.into_inner().unwrap());
        let mut solver = Solver::new(engines, constraints);
        let instructions = solver.solve(h)?;

        todo!()
    })
}

fn order_constraints(
    engines: Engines<'_>,
    constraints: HashMap<Constraint, usize>,
) -> Vec<Constraint> {
    let mut set = BTreeSet::new();
    for (constraint, _) in constraints.into_iter() {
        set.insert(WithEngines {
            engines,
            thing: constraint,
        });
    }
    set.into_iter()
        .map(|with_engines| with_engines.thing)
        .collect()
}
