mod constraint;
mod gather_constraints;
mod priv_prelude;

use crate::{language::ty, CompileResult, Engines};

use priv_prelude::*;

pub(super) fn monomorphize(engines: Engines<'_>, module: &mut ty::TyModule) -> CompileResult<()> {
    CompileResult::with_handler(|h| {
        gather_from_module(h, engines, module).and_then(|constraints| todo!())
    })
}
