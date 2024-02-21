#![allow(dead_code)]
use std::env;
use std::sync::atomic::Ordering;
use immix_rust::gc_init;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate immix_rust;

mod common;
mod objectmodel;
mod heap;

mod exhaust;
mod mark;
mod trace;
mod mt_trace;
mod gcbench;
mod mt_gcbench;
mod obj_init;
mod gc_ref;

fn init() {
    objectmodel::init();
}

fn main() {
    use heap;
    init();

    let mut immix_space_size = 0;
    let mut lo_space_size = 0;
    let mut gc_threads = 8;

    match env::var("HEAP_SIZE") {
        Ok(val) => {
            if val.ends_with("M") {
                let (num, _) = val.split_at(val.len() - 1);
                let heap_size = num.parse::<usize>().unwrap() << 20;
                
                immix_space_size = (heap_size as f64 * heap::IMMIX_SPACE_RATIO) as usize;
                heap::IMMIX_SPACE_SIZE.store(immix_space_size, Ordering::SeqCst);
                
                lo_space_size = (heap_size as f64 * heap::LO_SPACE_RATIO) as usize;
                heap::LO_SPACE_SIZE.store(lo_space_size, Ordering::SeqCst);

                println!("heap is {} bytes (immix: {} bytes, lo: {} bytes) . ", heap_size, immix_space_size, lo_space_size);
            } else {
                println!("unknow heap size variable: {}, ignore", val);
                println!("using default heap size: {} bytes. ", heap::IMMIX_SPACE_SIZE.load(Ordering::SeqCst));
            }
        },
        Err(_) => {
            println!("using default heap size: {} bytes. ", heap::IMMIX_SPACE_SIZE.load(Ordering::SeqCst));
        }
    }
    
    match env::var("N_GCTHREADS") {
        Ok(val) => {
            gc_threads = val.parse::<usize>().unwrap();
            heap::gc::GC_THREADS.store(gc_threads, Ordering::SeqCst);
        },
        Err(_) => {
            heap::gc::GC_THREADS.store(gc_threads, Ordering::SeqCst);
        }
    }

    gc_init(immix_space_size, lo_space_size, gc_threads);
    
    if cfg!(feature = "exhaust") {
        exhaust::exhaust_alloc();
    } else if cfg!(feature = "initobj") {
        obj_init::alloc_init();
    } else if cfg!(feature = "gcbench") {
        gcbench::start();
    } else if cfg!(feature = "mt-gcbench") {
        mt_gcbench::start();
    } else if cfg!(feature = "mark") {
        mark::alloc_mark();
    } else if cfg!(feature = "trace") {
        trace::alloc_trace();
    } else if cfg!(feature = "mt-trace") {
        mt_trace::alloc_trace();
    }
    else {
        println!("unknown features: build with 'cargo build --release --features \"exhaust\"");
    }
}
