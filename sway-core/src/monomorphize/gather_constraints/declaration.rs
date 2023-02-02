use sway_error::handler::{ErrorEmitted, Handler};

use crate::{language::ty, monomorphize::priv_prelude::*};

pub(crate) fn gather_from_decl(
    ctx: Context,
    handler: &Handler,
    decl: &ty::TyDeclaration,
) -> Result<Vec<Constraint>, ErrorEmitted> {
    match decl {
        ty::TyDeclaration::VariableDeclaration(decl) => todo!(),
        ty::TyDeclaration::ConstantDeclaration(_) => todo!(),
        ty::TyDeclaration::FunctionDeclaration(_, _) => todo!(),
        ty::TyDeclaration::TraitDeclaration(_) => todo!(),
        ty::TyDeclaration::StructDeclaration(_, _) => todo!(),
        ty::TyDeclaration::EnumDeclaration(_, _) => todo!(),
        ty::TyDeclaration::ImplTrait(_) => todo!(),
        ty::TyDeclaration::AbiDeclaration(_) => todo!(),
        ty::TyDeclaration::GenericTypeForFunctionScope { name, type_id } => todo!(),
        ty::TyDeclaration::ErrorRecovery(_) => Ok(vec![]),
        ty::TyDeclaration::StorageDeclaration(_) => todo!(),
    }
}

pub(crate) fn gather_from_var_decl(
    ctx: Context,
    handler: &Handler,
    var_decl: &ty::TyVariableDeclaration,
) -> Result<Vec<Constraint>, ErrorEmitted> {
    todo!()
}
