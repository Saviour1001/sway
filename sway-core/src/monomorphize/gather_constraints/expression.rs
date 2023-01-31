use sway_error::handler::{ErrorEmitted, Handler};

use crate::{language::ty, monomorphize::priv_prelude::*, Engines};

pub(crate) fn gather_from_exp(
    handler: &Handler,
    engines: Engines<'_>,
    exp: &ty::TyExpression,
) -> Result<Vec<Constraint>, ErrorEmitted> {
    todo!()
}
