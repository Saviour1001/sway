use sway_error::handler::{ErrorEmitted, Handler};

use crate::{monomorphize::priv_prelude::*, TypeParameter};

pub(crate) fn gather_from_type_param(
    ctx: Context,
    handler: &Handler,
    type_param: &TypeParameter,
) -> Result<(), ErrorEmitted> {
    todo!()
}
