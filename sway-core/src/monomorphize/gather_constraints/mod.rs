pub(crate) mod declaration;
pub(crate) mod expression;
pub(crate) mod node;

use sway_error::handler::{ErrorEmitted, Handler};

use crate::{language::ty, monomorphize::priv_prelude::*, Engines};

pub(crate) fn gather_from_module(
    handler: &Handler,
    engines: Engines<'_>,
    module: &ty::TyModule,
) -> Result<Vec<Constraint>, ErrorEmitted> {
    let mut constraints = module
        .submodules_recursive()
        .map(|(_, submod)| gather_from_module(handler, engines, &submod.module))
        .collect::<Result<Vec<Vec<Constraint>>, ErrorEmitted>>()
        .map(|v| v.into_iter().flatten().collect::<Vec<Constraint>>())?;
    let node_constraints = module
        .all_nodes
        .iter()
        .map(|node| gather_from_node(handler, engines, &node.content))
        .collect::<Result<Vec<Vec<Constraint>>, ErrorEmitted>>()
        .map(|v| v.into_iter().flatten().collect::<Vec<Constraint>>())?;
    constraints.extend(node_constraints);
    Ok(constraints)
}
