use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::{Ident, Span, Spanned};

use crate::{monomorphize::priv_prelude::*, namespace};

/// A single [Module] within a Sway project.
#[derive(Clone, Debug)]
pub(crate) struct Module<'a> {
    /// Name of the module, package name for root module, library name for other
    /// modules. Library name used is the same as declared in `library name;`.
    pub(crate) name: Option<Ident>,

    /// Submodules of the current module represented as an ordered map from each
    /// submodule's name to the associated [Module].
    pub(crate) submodules: im::OrdMap<ModuleName, Module<'a>>,

    pub(crate) typed: &'a namespace::Module,

    pub(crate) constraints: im::Vector<Constraint>,
}

impl<'a> Module<'a> {
    /// Lookup the submodule at the given path.
    pub(crate) fn submodule(&self, path: &Path) -> Option<&Module<'a>> {
        let mut module = self;
        for ident in path.iter() {
            match module.submodules.get(ident.as_str()) {
                Some(ns) => module = ns,
                None => return None,
            }
        }
        Some(module)
    }

    /// Unique access to the submodule at the given path.
    pub(crate) fn submodule_mut(&mut self, path: &Path) -> Option<&mut Module<'a>> {
        let mut module = self;
        for ident in path.iter() {
            match module.submodules.get_mut(ident.as_str()) {
                Some(ns) => module = ns,
                None => return None,
            }
        }
        Some(module)
    }

    /// Lookup the submodule at the given path.
    pub(crate) fn check_submodule(
        &self,
        handler: &Handler,
        path: &[Ident],
    ) -> Result<&Module<'a>, ErrorEmitted> {
        match self.submodule(path) {
            Some(module) => Ok(module),
            None => Err(handler.emit_err(module_not_found(path))),
        }
    }
}

impl<'a> std::ops::Index<&Path> for Module<'a> {
    type Output = Module<'a>;
    fn index(&self, path: &Path) -> &Self::Output {
        self.submodule(path)
            .unwrap_or_else(|| panic!("no module for the given path {path:?}"))
    }
}

impl<'a> std::ops::IndexMut<&Path> for Module<'a> {
    fn index_mut(&mut self, path: &Path) -> &mut Self::Output {
        self.submodule_mut(path)
            .unwrap_or_else(|| panic!("no module for the given path {path:?}"))
    }
}

impl<'a> From<Root<'a>> for Module<'a> {
    fn from(root: Root<'a>) -> Self {
        root.module
    }
}

impl<'a, 'b: 'a> From<&'b namespace::Module> for Module<'a> {
    fn from(value: &'b namespace::Module) -> Self {
        Module {
            name: value.name.clone(),
            submodules: value
                .submodules
                .iter()
                .map(|(name, submod)| (name, Module::from(submod)))
                .collect(),
            typed: value,
            constraints: im::Vector::new(),
        }
    }
}

fn module_not_found(path: &[Ident]) -> CompileError {
    CompileError::ModuleNotFound {
        span: path.iter().fold(path[0].span(), |acc, this_one| {
            if acc.path() == this_one.span().path() {
                Span::join(acc, this_one.span())
            } else {
                acc
            }
        }),
        name: path
            .iter()
            .map(|x| x.as_str())
            .collect::<Vec<_>>()
            .join("::"),
    }
}
