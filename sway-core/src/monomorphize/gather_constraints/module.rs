use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::Ident;

use crate::{language::ty, monomorphize::priv_prelude::*};

pub(crate) fn gather_from_root(
    mut ctx: Context,
    handler: &Handler,
    module: &ty::TyModule,
) -> Result<(), ErrorEmitted> {
    for (name, submod) in module.submodules_recursive() {
        gather_from_module(ctx.by_ref(), handler, name.clone(), &submod.module)?;
    }
    for node in module.all_nodes.iter() {
        gather_from_node(ctx.by_ref(), handler, &node.content)?;
    }
    Ok(())
}

pub(crate) fn gather_from_module(
    parent_ctx: Context,
    handler: &Handler,
    dep_name: Ident,
    module: &ty::TyModule,
) -> Result<(), ErrorEmitted> {
    parent_ctx.enter_submodule(dep_name, |mut submod_ctx| {
        module
            .submodules_recursive()
            .try_for_each(|(name, submod)| {
                gather_from_module(submod_ctx.by_ref(), handler, name.clone(), &submod.module)
            })
            .and_then(|_| {
                module.all_nodes.iter().try_for_each(|node| {
                    gather_from_node(submod_ctx.by_ref(), handler, &node.content)
                })
            })
    })
}
