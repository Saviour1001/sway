use sway_error::handler::{ErrorEmitted, Handler};

use crate::{language::ty, monomorphize::priv_prelude::*};

pub(crate) fn gather_from_exp(
    ctx: Context,
    handler: &Handler,
    exp: &ty::TyExpression,
) -> Result<Vec<Constraint>, ErrorEmitted> {
    todo!()
}
