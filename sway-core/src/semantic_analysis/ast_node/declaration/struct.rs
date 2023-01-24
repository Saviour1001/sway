use crate::{
    error::*,
    language::{parsed::*, ty},
    semantic_analysis::*,
    type_system::*,
};

impl ty::TyStructDeclaration {
    pub(crate) fn type_check(
        ctx: TypeCheckContext,
        decl: StructDeclaration,
    ) -> CompileResult<(ty::TyStructDeclaration, TypeSubstList)> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let StructDeclaration {
            name,
            fields,
            type_parameters,
            visibility,
            span,
            attributes,
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

        // type check the fields
        let mut new_fields = vec![];
        for field in fields.into_iter() {
            new_fields.push(check!(
                ty::TyStructField::type_check(ctx.by_ref(), field),
                return err(warnings, errors),
                warnings,
                errors
            ));
        }

        // create the struct decl
        let decl = ty::TyStructDeclaration {
            name,
            type_parameters: new_type_parameters,
            fields: new_fields,
            visibility,
            span,
            attributes,
        };

        ok((decl, type_subst_list), warnings, errors)
    }
}

impl ty::TyStructField {
    pub(crate) fn type_check(mut ctx: TypeCheckContext, field: StructField) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let type_engine = ctx.type_engine;
        let decl_engine = ctx.decl_engine;
        let initial_type_id = type_engine.insert(decl_engine, field.type_info);
        let r#type = check!(
            ctx.resolve_type_with_self(
                initial_type_id,
                &field.type_span,
                EnforceTypeArguments::Yes,
                None
            ),
            type_engine.insert(decl_engine, TypeInfo::ErrorRecovery),
            warnings,
            errors,
        );
        let field = ty::TyStructField {
            name: field.name,
            type_id: r#type,
            initial_type_id,
            span: field.span,
            type_span: field.type_span,
            attributes: field.attributes,
        };
        ok(field, warnings, errors)
    }
}
