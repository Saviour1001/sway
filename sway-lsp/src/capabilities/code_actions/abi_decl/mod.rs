pub(crate) mod abi_impl;

use sway_core::decl_engine::DeclId;
use sway_types::Spanned;
use tower_lsp::lsp_types::CodeActionOrCommand;

use self::abi_impl::AbiImplCodeAction;

use super::{CodeAction, CodeActionContext};

pub(crate) fn code_actions(
    decl_id: &DeclId,
    ctx: CodeActionContext,
) -> Option<Vec<CodeActionOrCommand>> {
    let decl = ctx
        .engines
        .de()
        .get_abi(decl_id.clone(), &decl_id.span())
        .ok()?;
    Some(vec![AbiImplCodeAction::new(ctx, &decl).code_action()])
}
