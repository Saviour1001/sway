use crate::{decl_engine::*, monomorphize::priv_prelude::*, Engines, TypeEngine};

/// Contextual state tracked and accumulated throughout gathering the
/// monomorphization constraints.
pub(crate) struct InstructContext<'a, 'b: 'a> {
    /// The namespace context accumulated throughout gathering the constraints.
    pub(crate) namespace: &'a mut Namespace<'b>,

    /// The type engine storing types.
    pub(crate) type_engine: &'a TypeEngine,

    /// The declaration engine holds declarations.
    pub(crate) decl_engine: &'a DeclEngine,
}

impl<'a, 'b> InstructContext<'a, 'b> {
    /// Initialize a context at the top-level of a module with its namespace.
    pub(crate) fn from_root(root_namespace: &'a mut Namespace<'b>, engines: Engines<'a>) -> Self {
        Self::from_module_namespace(root_namespace, engines)
    }

    fn from_module_namespace(namespace: &'a mut Namespace<'b>, engines: Engines<'a>) -> Self {
        let (type_engine, decl_engine) = engines.unwrap();
        Self {
            namespace,
            type_engine,
            decl_engine,
        }
    }

    /// Create a new context that mutably borrows the inner [Namespace] with a
    /// lifetime bound by `self`.
    pub(crate) fn by_ref(&mut self) -> InstructContext<'_, 'b> {
        InstructContext {
            namespace: self.namespace,
            type_engine: self.type_engine,
            decl_engine: self.decl_engine,
        }
    }

    /// Scope the [InstructContext] with the given [Namespace].
    pub(crate) fn scoped(self, namespace: &'a mut Namespace<'b>) -> InstructContext<'a, 'b> {
        InstructContext {
            namespace,
            type_engine: self.type_engine,
            decl_engine: self.decl_engine,
        }
    }
}
