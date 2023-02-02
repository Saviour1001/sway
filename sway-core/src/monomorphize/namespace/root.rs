use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::Ident;

use crate::{
    language::{ty, CallPath},
    monomorphize::priv_prelude::*,
};

/// The root module, from which all other modules can be accessed.
#[derive(Clone, Debug)]
pub(crate) struct Root<'a> {
    pub(crate) module: Module<'a>,
}

impl<'a> Root<'a> {
    /// Resolve a symbol that is potentially prefixed with some path, e.g.
    /// `foo::bar::symbol`.
    pub(crate) fn resolve_call_path(
        &self,
        handler: &Handler,
        mod_path: &Path,
        call_path: &CallPath,
    ) -> Result<&ty::TyDeclaration, ErrorEmitted> {
        let symbol_path: Vec<_> = mod_path
            .iter()
            .chain(&call_path.prefixes)
            .cloned()
            .collect();
        self.resolve_symbol(handler, &symbol_path, &call_path.suffix)
    }

    /// Given a path to a module and the identifier of a symbol within that
    /// module, resolve its declaration.
    pub(crate) fn resolve_symbol(
        &self,
        handler: &Handler,
        mod_path: &Path,
        symbol: &Ident,
    ) -> Result<&ty::TyDeclaration, ErrorEmitted> {
        let module = self.check_submodule(handler, mod_path)?;
        let true_symbol = self[mod_path]
            .typed
            .use_aliases
            .get(symbol.as_str())
            .unwrap_or(symbol);
        match module.typed.use_synonyms.get(symbol) {
            Some((src_path, _, _)) if mod_path != src_path => {
                self.resolve_symbol(handler, src_path, true_symbol)
            }
            _ => module
                .typed
                .check_symbol(true_symbol)
                .map_err(|err| handler.emit_err(err)),
        }
    }
}

impl<'a> std::ops::Deref for Root<'a> {
    type Target = Module<'a>;
    fn deref(&self) -> &Self::Target {
        &self.module
    }
}

impl<'a> std::ops::DerefMut for Root<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.module
    }
}

impl<'a> From<Module<'a>> for Root<'a> {
    fn from(module: Module<'a>) -> Root<'a> {
        Root { module }
    }
}

impl<'a> From<Namespace<'a>> for Root<'a> {
    fn from(namespace: Namespace<'a>) -> Root<'a> {
        namespace.root
    }
}
