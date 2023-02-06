use sway_error::handler::{ErrorEmitted, Handler};

use crate::{language::ty, monomorphize::priv_prelude::*, Engines};

pub(crate) fn apply_instructions(
    engines: Engines<'_>,
    handler: &Handler,
    module: &mut ty::TyModule,
    instructions: Vec<Instruction>,
) -> Result<(), ErrorEmitted> {
    todo!()
}
