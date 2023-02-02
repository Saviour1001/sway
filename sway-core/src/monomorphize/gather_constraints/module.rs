use sway_error::handler::{ErrorEmitted, Handler};

use crate::{language::ty, monomorphize::priv_prelude::*};

pub(crate) fn gather_from_root(
    mut ctx: Context,
    handler: &Handler,
    module: &ty::TyModule,
) -> Result<(), ErrorEmitted> {
    for (_, submod) in module.submodules_recursive() {
        // gather_from_module(ctx, handler, name.clone(), &submod.module)?;
        gather_from_module(ctx.by_ref(), handler, &submod.module)?;
    }
    for node in module.all_nodes.iter() {
        gather_from_node(ctx.by_ref(), handler, &node.content)?;
    }
    Ok(())
}

pub(crate) fn gather_from_module(
    ctx: Context,
    handler: &Handler,
    // dep_name: Ident,
    module: &ty::TyModule,
) -> Result<(), ErrorEmitted> {
    // parent_ctx.enter_submodule(dep_name, |mut submod_ctx| {
    //     module
    //         .submodules_recursive()
    //         .try_for_each(|(name, submod)| {
    //             gather_from_module(submod_ctx, handler, name.clone(), &submod.module)
    //         })
    //         .and_then(|_| {
    //             module.all_nodes.iter().try_for_each(|node| {
    //                 gather_from_node(submod_ctx, handler, &node.content)
    //             })
    //         })
    // })
    let module_namespace = Namespace::new_with_module(ctx.namespace, &module.namespace);
    let mut ctx = ctx.scoped(&module_namespace);
    for (_, submod) in module.submodules_recursive() {
        gather_from_module(ctx.by_ref(), handler, &submod.module)?;
    }
    for node in module.all_nodes.iter() {
        gather_from_node(ctx.by_ref(), handler, &node.content)?;
    }
    Ok(())
}
