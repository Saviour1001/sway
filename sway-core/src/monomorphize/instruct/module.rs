use sway_error::handler::{ErrorEmitted, Handler};

use crate::{language::ty, monomorphize::priv_prelude::*};

pub(crate) fn instruct_root(
    mut ctx: InstructContext,
    handler: &Handler,
    module: &ty::TyModule,
) -> Result<(), ErrorEmitted> {
    for (_, submod) in module.submodules_recursive() {
        instruct_module(ctx.by_ref(), handler, &submod.module)?;
    }
    for node in module.all_nodes.iter() {
        instruct_node(ctx.by_ref(), handler, &node.content)?;
    }
    Ok(())
}

pub(crate) fn instruct_module(
    ctx: InstructContext,
    handler: &Handler,
    module: &ty::TyModule,
) -> Result<(), ErrorEmitted> {
    let module_namespace = Namespace::new_with_module(ctx.namespace, &module.namespace);
    let mut ctx = ctx.scoped(&module_namespace);
    for (_, submod) in module.submodules_recursive() {
        instruct_module(ctx.by_ref(), handler, &submod.module)?;
    }
    for node in module.all_nodes.iter() {
        instruct_node(ctx.by_ref(), handler, &node.content)?;
    }
    Ok(())
}
