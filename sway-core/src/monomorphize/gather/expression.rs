use sway_error::handler::{ErrorEmitted, Handler};

use crate::{language::ty, monomorphize::priv_prelude::*, type_system::*};

pub(crate) fn gather_from_exp(
    ctx: Context,
    handler: &Handler,
    exp: &ty::TyExpression,
) -> Result<(), ErrorEmitted> {
    gather_from_exp_inner(ctx, handler, &exp.expression, exp.return_type)
}

pub(crate) fn gather_from_exp_inner(
    mut ctx: Context,
    handler: &Handler,
    exp: &ty::TyExpressionVariant,
    return_type: TypeId,
) -> Result<(), ErrorEmitted> {
    ctx.add_constraint(return_type.into());
    match exp {
        ty::TyExpressionVariant::FunctionApplication {
            arguments,
            contract_call_params,
            ..
        } => {
            arguments
                .iter()
                .try_for_each(|(_, arg)| gather_from_exp(ctx.by_ref(), handler, arg))?;
            contract_call_params
                .iter()
                .try_for_each(|(_, arg)| gather_from_exp(ctx.by_ref(), handler, arg))?;
            ctx.add_constraint(exp.into());
        }
        ty::TyExpressionVariant::LazyOperator { lhs, rhs, .. } => {
            gather_from_exp(ctx.by_ref(), handler, lhs)?;
            gather_from_exp(ctx.by_ref(), handler, rhs)?;
        }
        ty::TyExpressionVariant::VariableExpression { .. } => {
            // NOTE: may need to do something here later
        }
        ty::TyExpressionVariant::Tuple { fields } => {
            fields
                .iter()
                .try_for_each(|field| gather_from_exp(ctx.by_ref(), handler, field))?;
        }
        ty::TyExpressionVariant::Array { contents } => {
            todo!();
            contents
                .iter()
                .try_for_each(|elem| gather_from_exp(ctx.by_ref(), handler, elem))?;
        }
        ty::TyExpressionVariant::ArrayIndex { prefix, index } => {
            todo!();
            gather_from_exp(ctx.by_ref(), handler, prefix)?;
            gather_from_exp(ctx.by_ref(), handler, index)?;
        }
        ty::TyExpressionVariant::StructExpression {
            struct_name,
            fields,
            span,
            type_binding,
        } => todo!(),
        ty::TyExpressionVariant::CodeBlock(block) => {
            gather_from_code_block(ctx, handler, block)?;
        }
        ty::TyExpressionVariant::IfExp {
            condition,
            then,
            r#else,
        } => todo!(),
        ty::TyExpressionVariant::AsmExpression {
            registers,
            body,
            returns,
            whole_block_span,
        } => todo!(),
        ty::TyExpressionVariant::StructFieldAccess {
            prefix,
            field_to_access,
            field_instantiation_span,
            resolved_type_of_parent,
        } => todo!(),
        ty::TyExpressionVariant::TupleElemAccess {
            prefix,
            elem_to_access_num,
            resolved_type_of_parent,
            elem_to_access_span,
        } => {
            gather_from_exp(ctx, handler, prefix)?;
        }
        ty::TyExpressionVariant::EnumInstantiation {
            type_id,
            variant_name,
            tag,
            contents,
            enum_instantiation_span,
            variant_instantiation_span,
            type_binding,
        } => todo!(),
        ty::TyExpressionVariant::AbiCast {
            abi_name,
            address,
            span,
        } => todo!(),
        ty::TyExpressionVariant::StorageAccess(_) => todo!(),
        ty::TyExpressionVariant::IntrinsicFunction(_) => todo!(),
        ty::TyExpressionVariant::AbiName(_) => todo!(),
        ty::TyExpressionVariant::EnumTag { exp } => {
            gather_from_exp(ctx.by_ref(), handler, exp)?;
        }
        ty::TyExpressionVariant::UnsafeDowncast { exp, variant } => todo!(),
        ty::TyExpressionVariant::WhileLoop { condition, body } => todo!(),
        ty::TyExpressionVariant::Reassignment(_) => todo!(),
        ty::TyExpressionVariant::StorageReassignment(_) => todo!(),
        ty::TyExpressionVariant::Return(exp) => {
            gather_from_exp(ctx.by_ref(), handler, exp)?;
        }
        ty::TyExpressionVariant::Literal(_) => {}
        ty::TyExpressionVariant::Break => {}
        ty::TyExpressionVariant::Continue => {}
        ty::TyExpressionVariant::FunctionParameter => {}
    }

    Ok(())
}
