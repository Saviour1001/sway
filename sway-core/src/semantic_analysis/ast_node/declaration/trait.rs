use std::collections::BTreeMap;

use sway_error::warning::{CompileWarning, Warning};
use sway_types::{style::is_upper_camel_case, Ident, Spanned};

use crate::{
    error::*,
    language::{parsed::*, ty, CallPath},
    semantic_analysis::{declaration::insert_supertraits_into_namespace, Mode, TypeCheckContext},
    type_system::*,
};

type MethodMap = BTreeMap<Ident, ty::TyMethodValue>;

impl ty::TyTraitDeclaration {
    pub(crate) fn type_check(
        ctx: TypeCheckContext,
        trait_decl: TraitDeclaration,
    ) -> CompileResult<Self> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        let TraitDeclaration {
            name,
            type_parameters,
            attributes,
            interface_surface,
            methods,
            supertraits,
            visibility,
            span,
        } = trait_decl;

        if !is_upper_camel_case(name.as_str()) {
            warnings.push(CompileWarning {
                span: name.span(),
                warning_content: Warning::NonClassCaseTraitName { name: name.clone() },
            })
        }

        let type_engine = ctx.type_engine;
        let decl_engine = ctx.decl_engine;
        let engines = ctx.engines();

        // A temporary namespace for checking within the trait's scope.
        let self_type = type_engine.insert(decl_engine, TypeInfo::SelfType);
        let mut trait_namespace = ctx.namespace.clone();
        let mut ctx = ctx.scoped(&mut trait_namespace).with_self_type(self_type);

        // Type check the type parameters, which will insert them into the
        // namespace.
        let (new_type_parameters, _) = check!(
            TypeParameter::type_check_type_parameters(ctx.by_ref(), type_parameters, true),
            return err(warnings, errors),
            warnings,
            errors
        );

        // Recursively make the interface surfaces and methods of the
        // supertraits available to this trait.
        check!(
            insert_supertraits_into_namespace(ctx.by_ref(), self_type, &supertraits),
            return err(warnings, errors),
            warnings,
            errors
        );

        // type check the interface surface
        let mut new_interface_surface = vec![];
        let mut dummy_interface_surface = vec![];
        for method in interface_surface.into_iter() {
            let method = check!(
                ty::TyTraitFn::type_check(ctx.by_ref(), method),
                return err(warnings, errors),
                warnings,
                errors
            );
            new_interface_surface.push(ty::TyMethodValue::new(
                method.name.clone(),
                decl_engine.insert(type_engine, method.clone()),
                TypeSubstList::new(), // TODO: change this to a value when trait fns can take type params
            ));
            dummy_interface_surface.push(ty::TyMethodValue::new(
                method.name.clone(),
                decl_engine.insert(type_engine, method.to_dummy_func(Mode::NonAbi)),
                TypeSubstList::new(), // TODO: change this to a value when trait fns can take type params
            ));
        }

        // insert placeholder functions representing the interface surface
        // to allow methods to use those functions
        check!(
            ctx.namespace.insert_trait_implementation(
                CallPath {
                    prefixes: vec![],
                    suffix: name.clone(),
                    is_absolute: false,
                },
                new_type_parameters.iter().map(|x| x.into()).collect(),
                self_type,
                dummy_interface_surface,
                &span,
                false,
                engines,
            ),
            return err(warnings, errors),
            warnings,
            errors
        );

        // type check the methods
        let mut new_methods = vec![];
        for method in methods.into_iter() {
            let (method, type_subst_list) = check!(
                ty::TyFunctionDeclaration::type_check(ctx.by_ref(), method.clone(), true, false),
                (
                    ty::TyFunctionDeclaration::error(method, engines),
                    TypeSubstList::new()
                ),
                warnings,
                errors
            );
            new_methods.push(ty::TyMethodValue::new(
                method.name.clone(),
                decl_engine.insert(type_engine, method),
                type_subst_list,
            ));
        }

        let typed_trait_decl = ty::TyTraitDeclaration {
            name,
            type_parameters: new_type_parameters,
            interface_surface: new_interface_surface,
            methods: new_methods,
            supertraits,
            visibility,
            attributes,
            span,
        };
        ok(typed_trait_decl, warnings, errors)
    }

    /// Retrieves the interface surface and implemented methods for this trait.
    pub(crate) fn retrieve_interface_surface_and_implemented_methods_for_type(
        self,
        ctx: TypeCheckContext,
        type_id: TypeId,
        call_path: &CallPath,
    ) -> (MethodMap, MethodMap) {
        let ty::TyTraitDeclaration {
            interface_surface, ..
        } = self;

        let engines = ctx.engines();

        // Retrieve the interface surface for this trait.
        let interface_surface_methods = interface_surface
            .into_iter()
            .map(|method_value| (method_value.name.clone(), method_value))
            .collect();

        // Retrieve the implemented methods for this type.
        let impld_methods = ctx
            .namespace
            .get_methods_for_type_and_trait_name(engines, type_id, call_path)
            .into_iter()
            .map(|method_value| (method_value.name.clone(), method_value))
            .collect();

        (interface_surface_methods, impld_methods)
    }

    /// Retrieves the interface surface, methods, and implemented methods for
    /// this trait.
    pub(crate) fn retrieve_interface_surface_and_methods_and_implemented_methods_for_type(
        &self,
        ctx: TypeCheckContext,
        type_id: TypeId,
        call_path: &CallPath,
        type_arguments: &[TypeArgument],
    ) -> CompileResult<(MethodMap, MethodMap, MethodMap)> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let mut interface_surface_methods: MethodMap = BTreeMap::new();
        let mut the_methods: MethodMap = BTreeMap::new();
        let mut impld_methods: MethodMap = BTreeMap::new();

        let ty::TyTraitDeclaration {
            interface_surface,
            methods,
            type_parameters,
            ..
        } = self;

        let type_engine = ctx.type_engine;
        let decl_engine = ctx.decl_engine;
        let engines = ctx.engines();

        // Retrieve the interface surface for this trait.
        for method_value in interface_surface.iter() {
            interface_surface_methods.insert(method_value.name.clone(), method_value.clone());
        }

        // Retrieve the trait methods for this trait.
        for method_value in methods.iter() {
            the_methods.insert(method_value.name.clone(), method_value.clone());
        }

        // Retrieve the implemented methods for this type.
        let type_mapping = TypeSubstMap::from_type_parameters_and_type_arguments(
            type_parameters
                .iter()
                .map(|type_param| type_param.type_id)
                .collect(),
            type_arguments
                .iter()
                .map(|type_arg| type_arg.type_id)
                .collect(),
        );
        for mut method_value in ctx
            .namespace
            .get_methods_for_type_and_trait_name(engines, type_id, call_path)
            .into_iter()
        {
            method_value.type_subst_list.subst(&type_mapping, engines);
            impld_methods.insert(method_value.name.clone(), method_value);
        }

        ok(
            (interface_surface_methods, the_methods, impld_methods),
            warnings,
            errors,
        )
    }

    pub(crate) fn insert_interface_surface_and_methods_into_namespace(
        self,
        ctx: TypeCheckContext,
        trait_name: &CallPath,
        type_arguments: &[TypeArgument],
        type_id: TypeId,
    ) -> CompileResult<()> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let type_engine = ctx.type_engine;
        let decl_engine = ctx.decl_engine;
        let engines = ctx.engines();

        let ty::TyTraitDeclaration {
            interface_surface,
            methods,
            ..
        } = self;

        let all_methods = vec![interface_surface, methods].concat();

        // Insert the methods of the trait into the namespace.
        // Specifically do not check for conflicting definitions because
        // this is just a temporary namespace for type checking and
        // these are not actual impl blocks.
        ctx.namespace.insert_trait_implementation(
            trait_name.clone(),
            type_arguments.to_vec(),
            type_id,
            all_methods,
            &trait_name.span(),
            false,
            engines,
        );

        if errors.is_empty() {
            ok((), warnings, errors)
        } else {
            err(warnings, errors)
        }
    }
}
