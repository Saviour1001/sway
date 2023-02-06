pub(crate) mod context;
// pub(crate) mod namespace;

use sway_error::handler::{ErrorEmitted, Handler};

use crate::{language::ty, monomorphize::priv_prelude::*, Engines};

pub(crate) fn apply_instructions(
    engines: Engines<'_>,
    handler: &Handler,
    instructions: Vec<Instruction>,
    module: &mut ty::TyModule,
) -> Result<(), ErrorEmitted> {
    let mut root_namespace = InstructNamespace::init_root(&mut module.namespace);
    let ctx = InstructContext::from_root(&mut root_namespace, engines, &instructions);

    todo!()
}
