use crate::{
    decl_engine::*,
    engine_threading::*,
    error::*,
    language::{ty, CallPath},
    semantic_analysis::*,
    type_system::*,
};

use sway_error::error::CompileError;
use sway_types::{ident::Ident, span::Span, Spanned};

use fuel_abi_types::program_abi;

use std::{
    collections::BTreeMap,
    fmt,
    hash::{Hash, Hasher},
};

#[derive(Clone)]
pub struct TypeParameter {
    pub type_id: TypeId,
    pub(crate) initial_type_id: TypeId,
    pub name_ident: Ident,
    pub(crate) trait_constraints: Vec<TraitConstraint>,
    pub(crate) trait_constraints_span: Span,
}

impl HashWithEngines for TypeParameter {
    fn hash<H: Hasher>(&self, state: &mut H, type_engine: &TypeEngine) {
        type_engine.get(self.type_id).hash(state, type_engine);
        self.name_ident.hash(state);
        self.trait_constraints.hash(state, type_engine);
    }
}

impl EqWithEngines for TypeParameter {}
impl PartialEqWithEngines for TypeParameter {
    fn eq(&self, other: &Self, type_engine: &TypeEngine) -> bool {
        type_engine
            .get(self.type_id)
            .eq(&type_engine.get(other.type_id), type_engine)
            && self.name_ident == other.name_ident
            && self
                .trait_constraints
                .eq(&other.trait_constraints, type_engine)
    }
}

impl SubstTypes for TypeParameter {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        self.type_id.subst(type_mapping, engines);
        self.trait_constraints
            .iter_mut()
            .for_each(|x| x.subst(type_mapping, engines));
    }
}

impl FinalizeTypes for TypeParameter {
    fn finalize_inner(&mut self, engines: Engines<'_>, subst_list: &TypeSubstList) {
        self.type_id.finalize(engines, subst_list);
        self.trait_constraints
            .iter_mut()
            .for_each(|x| x.finalize(engines, subst_list));
    }
}

impl ReplaceSelfType for TypeParameter {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        self.type_id.replace_self_type(engines, self_type);
        self.trait_constraints
            .iter_mut()
            .for_each(|x| x.replace_self_type(engines, self_type));
    }
}

impl Spanned for TypeParameter {
    fn span(&self) -> Span {
        self.name_ident.span()
    }
}

impl DisplayWithEngines for TypeParameter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: Engines<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name_ident, engines.help_out(self.type_id))
    }
}

impl fmt::Debug for TypeParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {:?}", self.name_ident, self.type_id)
    }
}

impl TypeParameter {
    /// Given a list of [TypeParameter], type check the type parameters and
    /// insert them into the namespace.
    pub(crate) fn type_check_type_parameters(
        mut ctx: TypeCheckContext,
        type_parameters: Vec<TypeParameter>,
        where_clauses_allowed: bool,
    ) -> CompileResult<(Vec<TypeParameter>, TypeSubstList)> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let mut subst_list_refs = vec![];
        let mut type_subst_stack = ctx
            .namespace
            .get_type_subst_stack()
            .last()
            .cloned()
            .unwrap_or_default();

        for type_param in type_parameters.into_iter() {
            if !where_clauses_allowed && !type_param.trait_constraints.is_empty() {
                errors.push(CompileError::WhereClauseNotYetSupported {
                    span: type_param.trait_constraints_span,
                });
                return err(warnings, errors);
            }
            let (type_subst, subst_list_ref) = check!(
                TypeParameter::type_check(ctx.by_ref(), type_param, type_subst_stack.len()),
                continue,
                warnings,
                errors
            );
            type_subst_stack.push(type_subst);
            subst_list_refs.push(subst_list_ref);
        }

        if errors.is_empty() {
            ok((subst_list_refs, type_subst_stack), warnings, errors)
        } else {
            err(warnings, errors)
        }
    }

    /// Type checks a single [TypeParameter], including its [TraitConstraint]s.
    fn type_check(
        mut ctx: TypeCheckContext,
        type_parameter: TypeParameter,
        n: usize,
    ) -> CompileResult<(TypeParameter, TypeParameter)> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let type_engine = ctx.type_engine;
        let decl_engine = ctx.decl_engine;

        let TypeParameter {
            initial_type_id,
            name_ident,
            mut trait_constraints,
            trait_constraints_span,
            ..
        } = type_parameter;

        // Type check the trait constraints.
        for trait_constraint in trait_constraints.iter_mut() {
            check!(
                trait_constraint.type_check(ctx.by_ref()),
                return err(warnings, errors),
                warnings,
                errors
            );
        }

        // TODO: add check here to see if the type parameter has a valid name and does not have type parameters

        // Create the [TypeParameter] to go in the type subst list in the type
        // engine.
        let type_subst = TypeParameter {
            name_ident: name_ident.clone(),
            type_id: type_engine.insert(
                decl_engine,
                TypeInfo::UnknownGeneric {
                    name: name_ident.clone(),
                    trait_constraints: VecSet(trait_constraints.clone()),
                },
            ),
            initial_type_id,
            trait_constraints: trait_constraints.clone(),
            trait_constraints_span: trait_constraints_span.clone(),
        };

        // Create the [TypeParameter] that refers to the type subst list to go
        // in the AST.
        let subst_list_ref_id = type_engine.insert(decl_engine, TypeInfo::TypeParam(n));
        let subst_list_ref = TypeParameter {
            name_ident: name_ident.clone(),
            type_id: subst_list_ref_id,
            initial_type_id,
            trait_constraints: trait_constraints.clone(),
            trait_constraints_span,
        };
        let subst_list_ref_decl = ty::TyDeclaration::GenericTypeForFunctionScope {
            name: name_ident.clone(),
            type_id: subst_list_ref_id,
        };

        // Insert the trait constraints into the namespace.
        for trait_constraint in trait_constraints.iter() {
            check!(
                TraitConstraint::insert_into_namespace(
                    ctx.by_ref(),
                    subst_list_ref_id,
                    trait_constraint
                ),
                return err(warnings, errors),
                warnings,
                errors
            );
        }

        // Insert a reference to the type parameter into the namespace as a
        // dummy type declaration.
        ctx.namespace
            .insert_symbol(name_ident, subst_list_ref_decl)
            .ok(&mut warnings, &mut errors);

        ok((type_subst, subst_list_ref), warnings, errors)
    }

    /// Returns the initial type ID of a TypeParameter. Also updates the provided list of types to
    /// append the current TypeParameter as a `program_abi::TypeDeclaration`.
    pub(crate) fn get_json_type_parameter(
        &self,
        type_engine: &TypeEngine,
        types: &mut Vec<program_abi::TypeDeclaration>,
    ) -> usize {
        let type_parameter = program_abi::TypeDeclaration {
            type_id: self.initial_type_id.index(),
            type_field: self
                .initial_type_id
                .get_json_type_str(type_engine, self.type_id),
            components: self.initial_type_id.get_json_type_components(
                type_engine,
                types,
                self.type_id,
            ),
            type_parameters: None,
        };
        types.push(type_parameter);
        self.initial_type_id.index()
    }

    /// Creates a [DeclMapping] from a list of [TypeParameter]s.
    pub(crate) fn gather_decl_mapping_from_trait_constraints(
        mut ctx: TypeCheckContext,
        type_parameters: &[TypeParameter],
        access_span: &Span,
    ) -> CompileResult<DeclMapping> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let mut original_method_ids: BTreeMap<Ident, DeclId> = BTreeMap::new();
        let mut impld_method_ids: BTreeMap<Ident, DeclId> = BTreeMap::new();

        for type_param in type_parameters.iter() {
            let TypeParameter {
                type_id,
                trait_constraints,
                ..
            } = type_param;

            // Check to see if the trait constraints are satisfied.
            check!(
                ctx.namespace
                    .implemented_traits
                    .check_if_trait_constraints_are_satisfied_for_type(
                        *type_id,
                        trait_constraints,
                        access_span,
                        ctx.engines()
                    ),
                continue,
                warnings,
                errors
            );

            for trait_constraint in trait_constraints.iter() {
                let TraitConstraint {
                    trait_name,
                    type_arguments: trait_type_arguments,
                } = trait_constraint;

                let (trait_original_method_ids, trait_impld_method_ids) = check!(
                    handle_trait(ctx.by_ref(), *type_id, trait_name, trait_type_arguments),
                    continue,
                    warnings,
                    errors
                );
                original_method_ids.extend(trait_original_method_ids);
                impld_method_ids.extend(trait_impld_method_ids);
            }
        }

        if errors.is_empty() {
            let decl_mapping =
                DeclMapping::from_stub_and_impld_decl_ids(original_method_ids, impld_method_ids);
            ok(decl_mapping, warnings, errors)
        } else {
            err(warnings, errors)
        }
    }
}

fn handle_trait(
    mut ctx: TypeCheckContext,
    type_id: TypeId,
    trait_name: &CallPath,
    type_arguments: &[TypeArgument],
) -> CompileResult<(BTreeMap<Ident, DeclId>, BTreeMap<Ident, DeclId>)> {
    let mut warnings = vec![];
    let mut errors = vec![];

    let decl_engine = ctx.decl_engine;

    let mut original_method_ids: BTreeMap<Ident, DeclId> = BTreeMap::new();
    let mut impld_method_ids: BTreeMap<Ident, DeclId> = BTreeMap::new();

    match ctx
        .namespace
        .resolve_call_path(trait_name)
        .ok(&mut warnings, &mut errors)
        .cloned()
    {
        Some(ty::TyDeclaration::TraitDeclaration(decl_id)) => {
            let trait_decl = check!(
                CompileResult::from(decl_engine.get_trait(decl_id, &trait_name.suffix.span())),
                return err(warnings, errors),
                warnings,
                errors
            );

            let (trait_original_method_ids, trait_method_ids, trait_impld_method_ids) = check!(
                trait_decl.retrieve_interface_surface_and_methods_and_implemented_methods_for_type(
                    ctx.by_ref(),
                    type_id,
                    trait_name,
                    type_arguments
                ),
                return err(warnings, errors),
                warnings,
                errors
            );
            original_method_ids.extend(trait_original_method_ids);
            original_method_ids.extend(trait_method_ids);
            impld_method_ids.extend(trait_impld_method_ids);

            for supertrait in trait_decl.supertraits.iter() {
                let (supertrait_original_method_ids, supertrait_impld_method_ids) = check!(
                    handle_trait(ctx.by_ref(), type_id, &supertrait.name, &[]),
                    continue,
                    warnings,
                    errors
                );
                original_method_ids.extend(supertrait_original_method_ids);
                impld_method_ids.extend(supertrait_impld_method_ids);
            }
        }
        _ => errors.push(CompileError::TraitNotFound {
            name: trait_name.to_string(),
            span: trait_name.span(),
        }),
    }

    if errors.is_empty() {
        ok((original_method_ids, impld_method_ids), warnings, errors)
    } else {
        err(warnings, errors)
    }
}
