#![allow(dead_code)]
use std::sync::atomic::Ordering;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate libc;

pub mod common;
pub mod objectmodel;
pub mod heap;

use common::ObjectReference;
use std::sync::Arc;
use std::sync::RwLock;
use heap::immix::ImmixSpace;
use heap::immix::ImmixMutatorLocal;
use heap::freelist;
use heap::freelist::FreeListSpace;
use std::boxed::Box;

#[repr(C)]
pub struct GC {
    immix_space: Arc<ImmixSpace>,
    lo_space   : Arc<RwLock<FreeListSpace>>
}

lazy_static! {
    pub static ref MY_GC : RwLock<Option<GC>> = RwLock::new(None);
}

#[no_mangle]
pub extern fn init(immix_size: usize, lo_size: usize, n_gcthreads: usize) {
    // init space size
    heap::IMMIX_SPACE_SIZE.store(immix_size, Ordering::SeqCst);
    heap::LO_SPACE_SIZE.store(lo_size, Ordering::SeqCst);
    
    let (immix_space, lo_space) = {
        let immix_space = Arc::new(ImmixSpace::new(immix_size));
        let lo_space    = Arc::new(RwLock::new(FreeListSpace::new(lo_size)));

        heap::gc::init(immix_space.clone(), lo_space.clone());        
        
        (immix_space, lo_space)
    };
    
    *MY_GC.write().unwrap() = Some(GC {immix_space: immix_space, lo_space: lo_space});
    println!("heap is {} bytes (immix: {} bytes, lo: {} bytes) . ", immix_size + lo_size, immix_size, lo_size);
    
    // gc threads
    heap::gc::GC_THREADS.store(n_gcthreads, Ordering::SeqCst);
    println!("{} gc threads", n_gcthreads);
    
    // init object model
    objectmodel::init();
}

#[no_mangle]
pub extern fn new_mutator() -> Box<ImmixMutatorLocal> {
    Box::new(ImmixMutatorLocal::new(MY_GC.read().unwrap().as_ref().unwrap().immix_space.clone()))
}

#[no_mangle]
pub extern fn alloc(mut mutator: Box<ImmixMutatorLocal>, size: usize, align: usize) -> ObjectReference {
    let ret = mutator.alloc(size, align);
    unsafe {ret.to_object_reference()}
}

#[no_mangle]
pub extern fn alloc_large(mut mutator: Box<ImmixMutatorLocal>, size: usize) -> ObjectReference {
    let ret = freelist::alloc_large(size, 8, &mut mutator, MY_GC.read().unwrap().as_ref().unwrap().lo_space.clone());
    unsafe {ret.to_object_reference()}
}