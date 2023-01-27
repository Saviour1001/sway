use sway_types::{Ident, Span};

use crate::{engine_threading::*, language::ty::*, transform, type_system::*};

/// A [TyAbiDeclaration] contains the type-checked version of the parse tree's `AbiDeclaration`.
#[derive(Clone, Debug)]
pub struct TyAbiDeclaration {
    /// The name of the abi trait (also known as a "contract trait")
    pub name: Ident,
    /// The methods a contract is required to implement in order opt in to this interface
    pub interface_surface: Vec<TyMethodValue>,
    pub methods: Vec<TyMethodValue>,
    pub span: Span,
    pub attributes: transform::AttributesMap,
}

impl EqWithEngines for TyAbiDeclaration {}
impl PartialEqWithEngines for TyAbiDeclaration {
    fn eq(&self, other: &Self, type_engine: &TypeEngine) -> bool {
        self.name == other.name
        && self.interface_surface.eq(&other.interface_surface, type_engine)
        && self.methods.eq(&other.methods, type_engine)
        // span ignored
        && self.attributes == other.attributes
    }
}

impl CreateTypeId for TyAbiDeclaration {
    fn create_type_id(&self, engines: Engines<'_>, subst_list: TypeSubstList) -> TypeId {
        let type_engine = engines.te();
        let decl_engine = engines.de();
        let ty = TypeInfo::ContractCaller {
            abi_name: AbiName::Known(self.name.clone().into()),
            address: None,
        };
        type_engine.insert(decl_engine, ty)
    }
}
