use crate::{
    engine_threading::*,
    language::{parsed::*, *},
    transform::{self, AttributeKind},
    type_system::*,
};
use sway_types::{ident::Ident, span::Span, SpanTree};

#[derive(Debug, Clone)]
pub struct FunctionDeclaration {
    pub purity: Purity,
    pub attributes: transform::AttributesMap,
    pub name: Ident,
    pub visibility: Visibility,
    pub body: CodeBlock,
    pub parameters: Vec<FunctionParameter>,
    pub span: Span,
    pub return_type: TypeArgument,
    pub type_parameters: Vec<TypeParameter>,
}

#[derive(Debug, Clone)]
pub struct FunctionParameter {
    pub name: Ident,
    pub is_reference: bool,
    pub is_mutable: bool,
    pub mutability_span: Span,
    pub type_info: TypeInfo,
    pub type_span: Span,
    pub type_name_spans: Option<SpanTree>,
}

impl EqWithEngines for FunctionParameter {}
impl PartialEqWithEngines for FunctionParameter {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        self.name == other.name
            && self.is_reference == other.is_reference
            && self.is_mutable == other.is_mutable
            && self.mutability_span == other.mutability_span
            && self.type_info.eq(&other.type_info, engines)
            && self.type_span == other.type_span
    }
}

impl FunctionDeclaration {
    /// Checks if this `FunctionDeclaration` is a test.
    pub(crate) fn is_test(&self) -> bool {
        self.attributes
            .keys()
            .any(|k| matches!(k, AttributeKind::Test))
    }
}
