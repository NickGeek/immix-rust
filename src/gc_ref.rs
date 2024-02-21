use std::sync::{Arc, RwLock};
use heap::freelist::FreeListSpace;
use heap::immix::ImmixSpace;

#[repr(C)]
pub struct GC {
    pub(crate) immix_space: Arc<ImmixSpace>,
    pub(crate) lo_space   : Arc<RwLock<FreeListSpace>>
}

lazy_static! {
    pub static ref MY_GC : RwLock<Option<GC>> = RwLock::new(None);
}
