pub(crate) mod declaration;
pub(crate) mod expression;
pub(crate) mod module;
pub(crate) mod node;

use sway_error::handler::{ErrorEmitted, Handler};

use crate::{language::ty, monomorphize::priv_prelude::*};

use self::module::gather_from_root;

pub(super) fn gather_constraints(
    ctx: Context,
    handler: &Handler,
    module: &ty::TyModule,
) -> Result<(), ErrorEmitted> {
    gather_from_root(ctx, handler, module)?;
    Ok(())
}
