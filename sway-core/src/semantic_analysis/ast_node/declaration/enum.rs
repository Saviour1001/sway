use crate::{
    error::*,
    language::{parsed::*, ty},
    semantic_analysis::*,
    type_system::*,
};

impl ty::TyEnumDeclaration {
    pub fn type_check(
        ctx: TypeCheckContext,
        decl: EnumDeclaration,
    ) -> CompileResult<(ty::TyEnumDeclaration, TypeSubstList)> {
        let mut errors = vec![];
        let mut warnings = vec![];

        let EnumDeclaration {
            name,
            type_parameters,
            variants,
            span,
            attributes,
            visibility,
            ..
        } = decl;

        // create a namespace for the decl, used to create a scope for generics
        let mut decl_namespace = ctx.namespace.clone();
        let mut ctx = ctx.scoped(&mut decl_namespace);

        // Type check the type parameters, which will insert them into the
        // namespace.
        let (new_type_parameters, type_subst_list) = check!(
            TypeParameter::type_check_type_parameters(ctx.by_ref(), type_parameters, false),
            return err(warnings, errors),
            warnings,
            errors
        );

        // Push the new type subst list onto the stack.
        // NOTE: We don't ever need to pop this off of the stack because this
        // push happens inside of a fresh typing context with a fresh copy of
        // the namespace.
        ctx.namespace
            .get_mut_type_subst_stack()
            .push(type_subst_list.clone());

        // type check the variants
        let mut variants_buf = vec![];
        for variant in variants {
            variants_buf.push(check!(
                ty::TyEnumVariant::type_check(ctx.by_ref(), variant.clone()),
                continue,
                warnings,
                errors
            ));
        }

        // create the enum decl
        let decl = ty::TyEnumDeclaration {
            name,
            type_parameters: new_type_parameters,
            variants: variants_buf,
            span,
            attributes,
            visibility,
        };

        ok((decl, type_subst_list), warnings, errors)
    }
}

impl ty::TyEnumVariant {
    pub(crate) fn type_check(
        mut ctx: TypeCheckContext,
        variant: EnumVariant,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let type_engine = ctx.type_engine;
        let decl_engine = ctx.decl_engine;
        let initial_type_id = type_engine.insert(decl_engine, variant.type_info);
        let enum_variant_type = check!(
            ctx.resolve(
                initial_type_id,
                &variant.span,
                EnforceTypeArguments::Yes,
                None
            ),
            type_engine.insert(decl_engine, TypeInfo::ErrorRecovery),
            warnings,
            errors,
        );
        ok(
            ty::TyEnumVariant {
                name: variant.name.clone(),
                type_id: enum_variant_type,
                initial_type_id,
                type_span: variant.type_span.clone(),
                tag: variant.tag,
                span: variant.span,
                attributes: variant.attributes,
            },
            vec![],
            errors,
        )
    }
}
