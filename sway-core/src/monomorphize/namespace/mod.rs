use sway_types::Ident;

pub(crate) mod context;
pub(crate) mod module;
#[allow(clippy::module_inception)]
pub(crate) mod namespace;
pub(crate) mod root;
pub(crate) mod submodule_namespace;

pub(crate) type ModuleName = String;
pub(crate) type Path = [Ident];
pub(crate) type PathBuf = Vec<Ident>;
