use crate::{type_system::*, Engines};

pub(crate) trait CreateTypeId {
    fn create_type_id(&self, engines: Engines<'_>, subst_list: TypeSubstList) -> TypeId;
}
