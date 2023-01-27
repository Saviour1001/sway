use std::{
    fmt,
    hash::{Hash, Hasher},
};

use sway_error::error::CompileError;
use sway_types::{Ident, Span, Spanned};

use crate::{
    decl_engine::*,
    engine_threading::*,
    error::*,
    language::{ty::*, Visibility},
    type_system::*,
};

#[derive(Clone, Debug)]
pub enum TyDeclaration {
    VariableDeclaration(Box<TyVariableDeclaration>),
    ConstantDeclaration(DeclId),
    FunctionDeclaration(DeclId),
    TraitDeclaration(DeclId),
    StructDeclaration(DeclId, TypeSubstList),
    EnumDeclaration(DeclId, TypeSubstList),
    ImplTrait(DeclId),
    AbiDeclaration(DeclId),
    // If type parameters are defined for a function, they are put in the namespace just for
    // the body of that function.
    GenericTypeForFunctionScope { name: Ident, type_id: TypeId },
    ErrorRecovery(Span),
    StorageDeclaration(DeclId),
}

impl EqWithEngines for TyDeclaration {}
impl PartialEqWithEngines for TyDeclaration {
    fn eq(&self, other: &Self, type_engine: &TypeEngine) -> bool {
        match (self, other) {
            (Self::VariableDeclaration(x), Self::VariableDeclaration(y)) => x.eq(y, type_engine),
            (Self::ConstantDeclaration(x), Self::ConstantDeclaration(y)) => x == y,
            (Self::FunctionDeclaration(x), Self::FunctionDeclaration(y)) => x == y,
            (Self::TraitDeclaration(x), Self::TraitDeclaration(y)) => x == y,
            (Self::StructDeclaration(x1, x2), Self::StructDeclaration(y1, y2)) => {
                x1 == y1 && x2.eq(y2, type_engine)
            }
            (Self::EnumDeclaration(x1, x2), Self::EnumDeclaration(y1, y2)) => {
                x1 == y1 && x2.eq(y2, type_engine)
            }
            (Self::ImplTrait(x), Self::ImplTrait(y)) => x == y,
            (Self::AbiDeclaration(x), Self::AbiDeclaration(y)) => x == y,
            (Self::StorageDeclaration(x), Self::StorageDeclaration(y)) => x == y,
            (
                Self::GenericTypeForFunctionScope {
                    name: xn,
                    type_id: xti,
                },
                Self::GenericTypeForFunctionScope {
                    name: yn,
                    type_id: yti,
                },
            ) => xn == yn && xti == yti,
            (Self::ErrorRecovery(x), Self::ErrorRecovery(y)) => x == y,
            _ => false,
        }
    }
}

impl HashWithEngines for TyDeclaration {
    fn hash<H: Hasher>(&self, state: &mut H, type_engine: &TypeEngine) {
        match self {
            TyDeclaration::VariableDeclaration(decl) => {
                state.write_u8(1);
                decl.hash(state, type_engine);
            }
            TyDeclaration::ConstantDeclaration(decl_id) => {
                state.write_u8(2);
                decl_id.hash(state);
            }
            TyDeclaration::FunctionDeclaration(decl_id) => {
                state.write_u8(3);
                decl_id.hash(state);
            }
            TyDeclaration::TraitDeclaration(decl_id) => {
                state.write_u8(4);
                decl_id.hash(state);
            }
            TyDeclaration::StructDeclaration(decl_id, subst_list) => {
                state.write_u8(5);
                decl_id.hash(state);
                subst_list.hash(state, type_engine);
            }
            TyDeclaration::EnumDeclaration(decl_id, subst_list) => {
                state.write_u8(6);
                decl_id.hash(state);
                subst_list.hash(state, type_engine);
            }
            TyDeclaration::ImplTrait(decl_id) => {
                state.write_u8(7);
                decl_id.hash(state);
            }
            TyDeclaration::AbiDeclaration(decl_id) => {
                state.write_u8(8);
                decl_id.hash(state);
            }
            TyDeclaration::GenericTypeForFunctionScope { name, type_id } => {
                state.write_u8(9);
                name.hash(state);
                type_engine.get(*type_id).hash(state, type_engine);
            }
            TyDeclaration::ErrorRecovery(_) => {
                state.write_u8(10);
            }
            TyDeclaration::StorageDeclaration(decl_id) => {
                state.write_u8(11);
                decl_id.hash(state);
            }
        }
    }
}

impl SubstTypes for TyDeclaration {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        todo!();
        // use TyDeclaration::*;
        // match self {
        //     VariableDeclaration(ref mut var_decl) => var_decl.subst(type_mapping, engines),
        //     FunctionDeclaration(ref mut decl_id) => decl_id.subst(type_mapping, engines),
        //     TraitDeclaration(ref mut decl_id) => decl_id.subst(type_mapping, engines),
        //     StructDeclaration(_, _) | EnumDeclaration(_, _) => todo!(),
        //     // StructDeclaration(ref mut decl_id) => decl_id.subst(type_mapping, engines),
        //     // EnumDeclaration(ref mut decl_id, _) => decl_id.subst(type_mapping, engines),
        //     ImplTrait(decl_id) => decl_id.subst(type_mapping, engines),
        //     // generics in an ABI is unsupported by design
        //     AbiDeclaration(..)
        //     | ConstantDeclaration(_)
        //     | StorageDeclaration(..)
        //     | GenericTypeForFunctionScope { .. }
        //     | ErrorRecovery(_) => (),
        // }
    }
}

impl ReplaceSelfType for TyDeclaration {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        todo!();
        // use TyDeclaration::*;
        // match self {
        //     VariableDeclaration(ref mut var_decl) => var_decl.replace_self_type(engines, self_type),
        //     FunctionDeclaration(ref mut decl_id) => decl_id.replace_self_type(engines, self_type),
        //     TraitDeclaration(ref mut decl_id) => decl_id.replace_self_type(engines, self_type),
        //     StructDeclaration(_, _) | EnumDeclaration(_, _) => todo!(),
        //     // StructDeclaration(ref mut decl_id) => decl_id.replace_self_type(engines, self_type),
        //     // EnumDeclaration(ref mut decl_id, _) => decl_id.replace_self_type(engines, self_type),
        //     ImplTrait(decl_id) => decl_id.replace_self_type(engines, self_type),
        //     // generics in an ABI is unsupported by design
        //     AbiDeclaration(..)
        //     | ConstantDeclaration(_)
        //     | StorageDeclaration(..)
        //     | GenericTypeForFunctionScope { .. }
        //     | ErrorRecovery(_) => (),
        // }
    }
}

impl Spanned for TyDeclaration {
    fn span(&self) -> Span {
        use TyDeclaration::*;
        match self {
            VariableDeclaration(decl) => decl.name.span(),
            ConstantDeclaration(decl_id) => decl_id.span(),
            FunctionDeclaration(decl_id) => decl_id.span(),
            TraitDeclaration(decl_id) => decl_id.span(),
            StructDeclaration(_, _) | EnumDeclaration(_, _) => todo!(),
            // StructDeclaration(decl_id) => decl_id.span(),
            // EnumDeclaration(decl_id, _) => decl_id.span(),
            AbiDeclaration(decl_id) => decl_id.span(),
            ImplTrait(decl_id) => decl_id.span(),
            StorageDeclaration(decl) => decl.span(),
            GenericTypeForFunctionScope { name, .. } => name.span(),
            ErrorRecovery(span) => span.clone(),
        }
    }
}

impl DisplayWithEngines for TyDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: Engines<'_>) -> std::fmt::Result {
        let type_engine = engines.te();
        let decl_engine = engines.de();
        write!(
            f,
            "{} declaration ({})",
            self.friendly_name(),
            match self {
                TyDeclaration::VariableDeclaration(decl) => {
                    let TyVariableDeclaration {
                        mutability,
                        name,
                        type_ascription,
                        body,
                        ..
                    } = &**decl;
                    let mut builder = String::new();
                    match mutability {
                        VariableMutability::Mutable => builder.push_str("mut"),
                        VariableMutability::RefMutable => builder.push_str("ref mut"),
                        VariableMutability::Immutable => {}
                    }
                    builder.push_str(name.as_str());
                    builder.push_str(": ");
                    builder.push_str(
                        &engines
                            .help_out(type_engine.get(*type_ascription))
                            .to_string(),
                    );
                    builder.push_str(" = ");
                    builder.push_str(&engines.help_out(body).to_string());
                    builder
                }
                TyDeclaration::FunctionDeclaration(decl_id) => {
                    match decl_engine.get_function(decl_id.clone(), &decl_id.span()) {
                        Ok(TyFunctionDeclaration { name, .. }) => name.as_str().into(),
                        Err(_) => "unknown function".into(),
                    }
                }
                TyDeclaration::TraitDeclaration(decl_id) => {
                    match decl_engine.get_trait(decl_id.clone(), &decl_id.span()) {
                        Ok(TyTraitDeclaration { name, .. }) => name.as_str().into(),
                        Err(_) => "unknown trait".into(),
                    }
                }
                TyDeclaration::StructDeclaration(_, _) | TyDeclaration::EnumDeclaration(_, _) =>
                    todo!(),
                // TyDeclaration::StructDeclaration(decl_id) => {
                //     match decl_engine.get_struct(decl_id.clone(), &decl_id.span()) {
                //         Ok(TyStructDeclaration { name, .. }) => name.as_str().into(),
                //         Err(_) => "unknown struct".into(),
                //     }
                // }
                // TyDeclaration::EnumDeclaration(decl_id, _) => {
                //     match decl_engine.get_enum(decl_id.clone(), &decl_id.span()) {
                //         Ok(TyEnumDeclaration { name, .. }) => name.as_str().into(),
                //         Err(_) => "unknown enum".into(),
                //     }
                // }
                _ => String::new(),
            }
        )
    }
}

impl CollectTypesMetadata for TyDeclaration {
    // this is only run on entry nodes, which must have all well-formed types
    fn collect_types_metadata(
        &self,
        ctx: &mut CollectTypesMetadataContext,
    ) -> CompileResult<Vec<TypeMetadata>> {
        use TyDeclaration::*;
        let mut warnings = vec![];
        let mut errors = vec![];
        let decl_engine = ctx.decl_engine;
        let metadata = match self {
            VariableDeclaration(decl) => {
                let mut body = check!(
                    decl.body.collect_types_metadata(ctx),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                body.append(&mut check!(
                    decl.type_ascription.collect_types_metadata(ctx),
                    return err(warnings, errors),
                    warnings,
                    errors
                ));
                body
            }
            FunctionDeclaration(decl_id) => {
                match decl_engine.get_function(decl_id.clone(), &decl_id.span()) {
                    Ok(decl) => {
                        check!(
                            decl.collect_types_metadata(ctx),
                            return err(warnings, errors),
                            warnings,
                            errors
                        )
                    }
                    Err(e) => {
                        errors.push(e);
                        return err(warnings, errors);
                    }
                }
            }
            ConstantDeclaration(decl_id) => {
                match decl_engine.get_constant(decl_id.clone(), &decl_id.span()) {
                    Ok(TyConstantDeclaration { value, .. }) => {
                        check!(
                            value.collect_types_metadata(ctx),
                            return err(warnings, errors),
                            warnings,
                            errors
                        )
                    }
                    Err(e) => {
                        errors.push(e);
                        return err(warnings, errors);
                    }
                }
            }
            ErrorRecovery(_)
            | StorageDeclaration(_)
            | TraitDeclaration(_)
            | StructDeclaration(_, _)
            | EnumDeclaration(_, _)
            | ImplTrait { .. }
            | AbiDeclaration(_)
            | GenericTypeForFunctionScope { .. } => vec![],
        };
        if errors.is_empty() {
            ok(metadata, warnings, errors)
        } else {
            err(warnings, errors)
        }
    }
}

impl GetDeclIdent for TyDeclaration {
    fn get_decl_ident(&self, decl_engine: &DeclEngine) -> Option<Ident> {
        match self {
            TyDeclaration::VariableDeclaration(decl) => Some(decl.name.clone()),
            TyDeclaration::ConstantDeclaration(decl) => Some(
                decl_engine
                    .get_constant(decl.clone(), &decl.span())
                    .unwrap()
                    .name,
            ),
            TyDeclaration::FunctionDeclaration(decl) => Some(
                decl_engine
                    .get_function(decl.clone(), &decl.span())
                    .unwrap()
                    .name,
            ),
            TyDeclaration::TraitDeclaration(decl) => Some(
                decl_engine
                    .get_trait(decl.clone(), &decl.span())
                    .unwrap()
                    .name,
            ),
            TyDeclaration::StructDeclaration(_, _) | TyDeclaration::EnumDeclaration(_, _) => {
                todo!()
            }
            // TyDeclaration::StructDeclaration(decl) => Some(
            //     decl_engine
            //         .get_struct(decl.clone(), &decl.span())
            //         .unwrap()
            //         .name,
            // ),
            // TyDeclaration::EnumDeclaration(decl, _) => Some(
            //     decl_engine
            //         .get_enum(decl.clone(), &decl.span())
            //         .unwrap()
            //         .name,
            // ),
            TyDeclaration::ImplTrait(decl) => Some(
                decl_engine
                    .get_impl_trait(decl.clone(), &decl.span())
                    .unwrap()
                    .trait_name
                    .suffix,
            ),
            TyDeclaration::AbiDeclaration(decl) => Some(
                decl_engine
                    .get_abi(decl.clone(), &decl.span())
                    .unwrap()
                    .name,
            ),
            TyDeclaration::GenericTypeForFunctionScope { name, .. } => Some(name.clone()),
            TyDeclaration::ErrorRecovery(_) => None,
            TyDeclaration::StorageDeclaration(_decl) => None,
        }
    }
}

impl TyDeclaration {
    /// Returns `Some(decl_id)` if `self` is an enum declaration, otherwise
    /// returns `None`.
    pub(crate) fn unwrap_enum(&self) -> Option<DeclId> {
        match self {
            TyDeclaration::EnumDeclaration(decl_id, _) => todo!(),
            _ => None,
        }
    }

    /// Retrieves the declaration as an enum declaration.
    ///
    /// Returns an error if `self` is not a [TyEnumDeclaration].
    pub(crate) fn expect_enum(
        self,
        decl_engine: &DeclEngine,
        access_span: &Span,
    ) -> CompileResult<(TyEnumDeclaration, DeclId, TypeSubstList)> {
        match self {
            TyDeclaration::EnumDeclaration(decl_id, type_subst_list) => {
                CompileResult::from(decl_engine.get_enum(decl_id.clone(), access_span))
                    .map(|decl| (decl, decl_id.clone(), type_subst_list))
            }
            TyDeclaration::ErrorRecovery(_) => err(vec![], vec![]),
            decl => err(
                vec![],
                vec![CompileError::DeclIsNotAnEnum {
                    actually: decl.friendly_name().to_string(),
                    span: decl.span(),
                }],
            ),
        }
    }

    /// Retrieves the declaration as a struct declaration.
    ///
    /// Returns an error if `self` is not a [TyStructDeclaration].
    pub(crate) fn expect_struct(
        &self,
        decl_engine: &DeclEngine,
        access_span: &Span,
    ) -> CompileResult<TyStructDeclaration> {
        todo!()
        // let mut warnings = vec![];
        // let mut errors = vec![];
        // match self {
        //     TyDeclaration::StructDeclaration(decl_id) => {
        //         let decl = check!(
        //             CompileResult::from(decl_engine.get_struct(decl_id.clone(), access_span)),
        //             return err(warnings, errors),
        //             warnings,
        //             errors
        //         );
        //         ok(decl, warnings, errors)
        //     }
        //     TyDeclaration::ErrorRecovery(_) => err(vec![], vec![]),
        //     decl => {
        //         errors.push(CompileError::DeclIsNotAStruct {
        //             actually: decl.friendly_name().to_string(),
        //             span: decl.span(),
        //         });
        //         err(warnings, errors)
        //     }
        // }
    }

    /// Retrieves the declaration as a function declaration.
    ///
    /// Returns an error if `self` is not a [TyFunctionDeclaration].
    pub(crate) fn expect_function(
        self,
        decl_engine: &DeclEngine,
        access_span: &Span,
    ) -> CompileResult<(TyFunctionDeclaration, DeclId, TypeSubstList)> {
        let mut warnings = vec![];
        let mut errors = vec![];
        match self {
            TyDeclaration::FunctionDeclaration(decl_id) => {
                let type_subst_list: TypeSubstList = todo!();
                CompileResult::from(decl_engine.get_function(decl_id.clone(), access_span))
                    .map(|decl| (decl, decl_id.clone(), type_subst_list))
            }
            TyDeclaration::ErrorRecovery(_) => err(vec![], vec![]),
            decl => {
                errors.push(CompileError::DeclIsNotAFunction {
                    actually: decl.friendly_name().to_string(),
                    span: decl.span(),
                });
                err(warnings, errors)
            }
        }
    }

    /// Retrieves the declaration as a variable declaration.
    ///
    /// Returns an error if `self` is not a [TyVariableDeclaration].
    pub(crate) fn expect_variable(&self) -> CompileResult<&TyVariableDeclaration> {
        let warnings = vec![];
        let mut errors = vec![];
        match self {
            TyDeclaration::VariableDeclaration(decl) => ok(decl, warnings, errors),
            TyDeclaration::ErrorRecovery(_) => err(vec![], vec![]),
            decl => {
                errors.push(CompileError::DeclIsNotAVariable {
                    actually: decl.friendly_name().to_string(),
                    span: decl.span(),
                });
                err(warnings, errors)
            }
        }
    }

    /// Retrieves the declaration as an Abi declaration.
    ///
    /// Returns an error if `self` is not a [TyAbiDeclaration].
    pub(crate) fn expect_abi(
        &self,
        decl_engine: &DeclEngine,
        access_span: &Span,
    ) -> CompileResult<TyAbiDeclaration> {
        match self {
            TyDeclaration::AbiDeclaration(decl_id) => {
                CompileResult::from(decl_engine.get_abi(decl_id.clone(), access_span))
            }
            TyDeclaration::ErrorRecovery(_) => err(vec![], vec![]),
            decl => err(
                vec![],
                vec![CompileError::DeclIsNotAnAbi {
                    actually: decl.friendly_name().to_string(),
                    span: decl.span(),
                }],
            ),
        }
    }

    /// Retrieves the declaration as an Constant declaration.
    ///
    /// Returns an error if `self` is not a [TyConstantDeclaration].
    pub(crate) fn expect_const(
        &self,
        decl_engine: &DeclEngine,
        access_span: &Span,
    ) -> CompileResult<TyConstantDeclaration> {
        match self {
            TyDeclaration::ConstantDeclaration(decl) => {
                CompileResult::from(decl_engine.get_constant(decl.clone(), access_span))
            }
            TyDeclaration::ErrorRecovery(_) => err(vec![], vec![]),
            decl => {
                let errors = vec![
                    (CompileError::DeclIsNotAConstant {
                        actually: decl.friendly_name().to_string(),
                        span: decl.span(),
                    }),
                ];
                err(vec![], errors)
            }
        }
    }

    /// friendly name string used for error reporting.
    pub fn friendly_name(&self) -> &'static str {
        use TyDeclaration::*;
        match self {
            VariableDeclaration(_) => "variable",
            ConstantDeclaration(_) => "constant",
            FunctionDeclaration(_) => "function",
            TraitDeclaration(_) => "trait",
            StructDeclaration(_, _) => "struct",
            EnumDeclaration(_, _) => "enum",
            ImplTrait { .. } => "impl trait",
            AbiDeclaration(..) => "abi",
            GenericTypeForFunctionScope { .. } => "generic type parameter",
            ErrorRecovery(_) => "error",
            StorageDeclaration(_) => "contract storage declaration",
        }
    }

    /// name string used in `forc doc` file path generation that mirrors `cargo doc`.
    pub fn doc_name(&self) -> &'static str {
        use TyDeclaration::*;
        match self {
            StructDeclaration(_, _) => "struct",
            EnumDeclaration(_, _) => "enum",
            TraitDeclaration(_) => "trait",
            AbiDeclaration(_) => "abi",
            StorageDeclaration(_) => "contract_storage",
            ImplTrait(_) => "impl_trait",
            FunctionDeclaration(_) => "fn",
            ConstantDeclaration(_) => "constant",
            _ => unreachable!("these items are non-documentable"),
        }
    }

    pub(crate) fn return_type(
        &self,
        engines: Engines<'_>,
        access_span: &Span,
    ) -> CompileResult<TypeId> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let type_engine = engines.te();
        let decl_engine = engines.de();
        let type_id = match self {
            TyDeclaration::VariableDeclaration(decl) => decl.body.return_type,
            TyDeclaration::FunctionDeclaration(decl_id) => {
                let decl = check!(
                    CompileResult::from(decl_engine.get_function(decl_id.clone(), &self.span())),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                decl.return_type
            }
            TyDeclaration::StructDeclaration(_, _) | TyDeclaration::EnumDeclaration(_, _) => {
                todo!()
            }
            // TyDeclaration::StructDeclaration(decl_id) => {
            //     let decl = check!(
            //         CompileResult::from(decl_engine.get_struct(decl_id.clone(), &self.span())),
            //         return err(warnings, errors),
            //         warnings,
            //         errors
            //     );
            //     todo!()
            //     // decl.create_type_id(engines)
            // }
            // TyDeclaration::EnumDeclaration(decl_id, type_subst_list) => {
            //     let decl = check!(
            //         CompileResult::from(decl_engine.get_enum(decl_id.clone(), access_span)),
            //         return err(warnings, errors),
            //         warnings,
            //         errors
            //     );
            //     type_engine.insert(
            //         decl_engine,
            //         TypeInfo::Enum {
            //             name: decl.name,
            //             decl_id: decl_id.clone(),
            //             subst_list: type_subst_list.clone(),
            //         },
            //     )
            // }
            TyDeclaration::StorageDeclaration(decl_id) => {
                let storage_decl = check!(
                    CompileResult::from(decl_engine.get_storage(decl_id.clone(), &self.span())),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                type_engine.insert(
                    decl_engine,
                    TypeInfo::Storage {
                        fields: storage_decl.fields_as_typed_struct_fields(),
                    },
                )
            }
            TyDeclaration::GenericTypeForFunctionScope { type_id, .. } => *type_id,
            decl => {
                errors.push(CompileError::NotAType {
                    span: decl.span(),
                    name: engines.help_out(decl).to_string(),
                    actually_is: decl.friendly_name(),
                });
                return err(warnings, errors);
            }
        };
        ok(type_id, warnings, errors)
    }

    pub(crate) fn visibility(&self, decl_engine: &DeclEngine) -> CompileResult<Visibility> {
        use TyDeclaration::*;
        let mut warnings = vec![];
        let mut errors = vec![];
        let visibility = match self {
            TraitDeclaration(decl_id) => {
                let TyTraitDeclaration { visibility, .. } = check!(
                    CompileResult::from(decl_engine.get_trait(decl_id.clone(), &decl_id.span())),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                visibility
            }
            ConstantDeclaration(decl_id) => {
                let TyConstantDeclaration { visibility, .. } = check!(
                    CompileResult::from(decl_engine.get_constant(decl_id.clone(), &decl_id.span())),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                visibility
            }
            TyDeclaration::StructDeclaration(_, _) | TyDeclaration::EnumDeclaration(_, _) => {
                todo!()
            }
            // StructDeclaration(decl_id) => {
            //     let TyStructDeclaration { visibility, .. } = check!(
            //         CompileResult::from(decl_engine.get_struct(decl_id.clone(), &decl_id.span())),
            //         return err(warnings, errors),
            //         warnings,
            //         errors
            //     );
            //     visibility
            // }
            // EnumDeclaration(decl_id, _) => {
            //     let TyEnumDeclaration { visibility, .. } = check!(
            //         CompileResult::from(decl_engine.get_enum(decl_id.clone(), &decl_id.span())),
            //         return err(warnings, errors),
            //         warnings,
            //         errors
            //     );
            //     visibility
            // }
            FunctionDeclaration(decl_id) => {
                let TyFunctionDeclaration { visibility, .. } = check!(
                    CompileResult::from(decl_engine.get_function(decl_id.clone(), &decl_id.span())),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                visibility
            }
            GenericTypeForFunctionScope { .. }
            | ImplTrait { .. }
            | StorageDeclaration { .. }
            | AbiDeclaration(..)
            | ErrorRecovery(_) => Visibility::Public,
            VariableDeclaration(decl) => decl.mutability.visibility(),
        };
        ok(visibility, warnings, errors)
    }
}
