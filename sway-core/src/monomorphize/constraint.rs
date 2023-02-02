use sway_types::Ident;

use crate::{
    decl_engine::DeclId,
    language::{ty, CallPath},
    type_system::*,
};

#[derive(Debug, Clone)]
pub(crate) enum Constraint {
    // VarDecl {
    //     name: Ident,
    //     return_type: TypeId,
    // },
    // FnDecl {
    //     name: Ident,
    //     type_params: Vec<(Ident, TypeId)>,
    //     params: Vec<(Ident, TypeId)>,
    //     return_type: TypeId,
    // },
    FnCall {
        call_path: CallPath,
        decl_id: DeclId,
        subst_list: TypeSubstList,
        arguments: Vec<TypeId>,
    },
}

impl From<&ty::TyExpressionVariant> for Constraint {
    fn from(value: &ty::TyExpressionVariant) -> Self {
        match value {
            ty::TyExpressionVariant::FunctionApplication {
                call_path,
                arguments,
                function_decl_id,
                function_type_subst_list,
                ..
            } => Constraint::FnCall {
                call_path: call_path.clone(),
                decl_id: function_decl_id.clone(),
                subst_list: function_type_subst_list.clone(),
                arguments: args_helper(arguments),
            },
            _ => unimplemented!(),
        }
    }
}

fn args_helper(args: &[(Ident, ty::TyExpression)]) -> Vec<TypeId> {
    args.iter().map(|(_, exp)| exp.return_type).collect()
}

// impl From<&ty::TyVariableDeclaration> for Constraint {
//     fn from(value: &ty::TyVariableDeclaration) -> Self {
//         Constraint::VarDecl {
//             name: value.name.clone(),
//             return_type: value.return_type,
//         }
//     }
// }

// impl From<&ty::TyFunctionDeclaration> for Constraint {
//     fn from(value: &ty::TyFunctionDeclaration) -> Self {
//         Constraint::FnDecl {
//             name: value.name.clone(),
//             type_params: type_params_helper(&value.type_parameters),
//             params: fn_params_helper(&value.parameters),
//             return_type: value.return_type,
//         }
//     }
// }

// fn type_params_helper(type_params: &[TypeParameter]) -> Vec<(Ident, TypeId)> {
//     type_params
//         .iter()
//         .map(|type_param| (type_param.name_ident.clone(), type_param.type_id))
//         .collect()
// }

// fn fn_params_helper(params: &[ty::TyFunctionParameter]) -> Vec<(Ident, TypeId)> {
//     params
//         .iter()
//         .map(|param| (param.name.clone(), param.type_id))
//         .collect()
// }
