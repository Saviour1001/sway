use std::fmt;

use sway_types::Ident;

use crate::{decl_engine::*, monomorphize::priv_prelude::*, Engines, TypeEngine};

/// Contextual state tracked and accumulated throughout gathering the
/// monomorphization constraints.
pub(crate) struct Context<'a, 'b: 'a> {
    /// The namespace context accumulated throughout gathering the constraints.
    pub(crate) namespace: &'a mut Namespace<'b>,

    /// The type engine storing types.
    pub(crate) type_engine: &'a TypeEngine,

    /// The declaration engine holds declarations.
    pub(crate) decl_engine: &'a DeclEngine,

    pub(crate) constraints: im::Vector<Constraint>,
}

impl<'a, 'b> Context<'a, 'b> {
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
    pub(crate) fn by_ref(&mut self) -> Context<'_, 'b> {
        Context {
            namespace: self.namespace,
            type_engine: self.type_engine,
            decl_engine: self.decl_engine,
        }
    }

    /// Scope the [Context] with the given [Namespace].
    pub(crate) fn scoped(self, namespace: &'a mut Namespace<'b>) -> Context<'a, 'b> {
        Context {
            namespace,
            type_engine: self.type_engine,
            decl_engine: self.decl_engine,
        }
    }

    /// Enter the submodule with the given name and produce a type-check context ready for
    /// type-checking its content.
    ///
    /// Returns the result of the given `with_submod_ctx` function.
    pub(crate) fn enter_submodule<T>(
        self,
        dep_name: Ident,
        with_submod_ctx: impl FnOnce(Context) -> T,
    ) -> T {
        // We're checking a submodule, so no need to pass through anything other than the
        // namespace. However, we will likely want to pass through the type engine and declaration
        // engine here once they're added.
        let Self { namespace, .. } = self;
        let mut submod_ns = namespace.enter_submodule(dep_name);
        let submod_ctx = Context::from_module_namespace(
            &mut submod_ns,
            Engines::new(self.type_engine, self.decl_engine),
        );
        with_submod_ctx(submod_ctx)
    }
}

impl fmt::Debug for Context<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.namespace)
    }
}
