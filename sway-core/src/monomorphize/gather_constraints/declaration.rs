use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::Spanned;

use crate::{language::ty, monomorphize::priv_prelude::*, TypeSubstList};

pub(crate) fn gather_from_decl(
    ctx: Context,
    handler: &Handler,
    decl: &ty::TyDeclaration,
) -> Result<(), ErrorEmitted> {
    let decl_engine = ctx.decl_engine;
    match decl {
        ty::TyDeclaration::VariableDeclaration(decl) => {
            gather_from_var_decl(ctx, handler, decl)?;
        }
        ty::TyDeclaration::ConstantDeclaration(_) => todo!(),
        ty::TyDeclaration::FunctionDeclaration(decl_id, type_subst_list) => {
            let fn_decl = decl_engine
                .get_function(decl_id.clone(), &decl_id.span())
                .map_err(|err| handler.emit_err(err))?;
            gather_from_fn_decl(ctx, handler, &fn_decl, type_subst_list)?;
        }
        ty::TyDeclaration::TraitDeclaration(_) => todo!(),
        ty::TyDeclaration::StructDeclaration(_, _) => todo!(),
        ty::TyDeclaration::EnumDeclaration(_, _) => todo!(),
        ty::TyDeclaration::ImplTrait(_) => todo!(),
        ty::TyDeclaration::AbiDeclaration(_) => todo!(),
        ty::TyDeclaration::GenericTypeForFunctionScope { .. } => todo!(),
        ty::TyDeclaration::StorageDeclaration(_) => todo!(),
        ty::TyDeclaration::ErrorRecovery(_) => {}
    }

    Ok(())
}

fn gather_from_var_decl(
    ctx: Context,
    handler: &Handler,
    decl: &ty::TyVariableDeclaration,
) -> Result<(), ErrorEmitted> {
    gather_from_exp(ctx, handler, &decl.body)?;
    Ok(())
}

fn gather_from_fn_decl(
    mut ctx: Context,
    handler: &Handler,
    decl: &ty::TyFunctionDeclaration,
    type_subst_list: &TypeSubstList,
) -> Result<(), ErrorEmitted> {
    if !type_subst_list.is_empty() {
        unimplemented!("{}", decl.name);
    }

    // decl.type_parameters
    //     .iter()
    //     .try_for_each(|type_param| gather_from_type_param(ctx.by_ref(), handler, type_param))?;
    decl.parameters
        .iter()
        .try_for_each(|param| gather_from_fn_param(ctx.by_ref(), handler, param))?;
    gather_from_code_block(ctx.by_ref(), handler, &decl.body)?;

    Ok(())
}

fn gather_from_fn_param(
    ctx: Context,
    handler: &Handler,
    param: &ty::TyFunctionParameter,
) -> Result<(), ErrorEmitted> {
    todo!()
}
