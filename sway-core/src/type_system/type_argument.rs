use crate::{engine_threading::*, type_system::*};
use std::{fmt, hash::Hasher};
use sway_types::{Span, Spanned};

#[derive(Debug, Clone)]
pub struct TypeArgument {
    pub type_id: TypeId,
    pub initial_type_id: TypeId,
    pub span: Span,
}

impl Spanned for TypeArgument {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl HashWithEngines for TypeArgument {
    fn hash<H: Hasher>(&self, state: &mut H, type_engine: &TypeEngine) {
        type_engine.get(self.type_id).hash(state, type_engine);
    }
}

impl EqWithEngines for TypeArgument {}
impl PartialEqWithEngines for TypeArgument {
    fn eq(&self, other: &Self, type_engine: &TypeEngine) -> bool {
        type_engine
            .get(self.type_id)
            .eq(&type_engine.get(other.type_id), type_engine)
    }
}
impl OrdWithEngines for TypeArgument {
    fn cmp(&self, rhs: &Self, _: &TypeEngine) -> std::cmp::Ordering {
        self.type_id
            .cmp(&rhs.type_id)
            .then_with(|| self.initial_type_id.cmp(&rhs.initial_type_id))
    }
}

impl DisplayWithEngines for TypeArgument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: Engines<'_>) -> fmt::Result {
        write!(f, "{}", engines.help_out(engines.te().get(self.type_id)))
    }
}

impl From<&TypeParameter> for TypeArgument {
    fn from(type_param: &TypeParameter) -> Self {
        TypeArgument {
            type_id: type_param.type_id,
            initial_type_id: type_param.initial_type_id,
            span: type_param.name_ident.span(),
        }
    }
}

impl ReplaceSelfType for TypeArgument {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        self.type_id.replace_self_type(engines, self_type);
    }
}

impl SubstTypes for TypeArgument {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        self.type_id.subst(type_mapping, engines);
    }
}

impl FinalizeTypes for TypeArgument {
    fn finalize_inner(&mut self, engines: Engines<'_>, subst_list: &TypeSubstList) {
        self.type_id.finalize(engines, subst_list);
    }
}
