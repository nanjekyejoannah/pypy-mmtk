extern crate libc;
extern crate mmtk;
#[macro_use]
extern crate lazy_static;
use std::ops::Range;
use mmtk::vm::VMBinding;
use mmtk::util::Address;
use std::sync::atomic::AtomicUsize;
use std::collections::HashMap;
use mmtk::MMTKBuilder;
use mmtk::MMTK;

pub mod active_plan;
pub mod api;
pub mod collection;
pub mod object_model;
pub mod reference_glue;
pub mod scanning;

mod edges;
#[cfg(test)]
mod tests;

#[derive(Default)]
pub struct PyPy;

pub type PyPyEdge = Address;

impl VMBinding for PyPy {
    type VMObjectModel = object_model::VMObjectModel;
    type VMScanning = scanning::VMScanning;
    type VMCollection = collection::VMCollection;
    type VMActivePlan = active_plan::VMActivePlan;
    type VMReferenceGlue = reference_glue::VMReferenceGlue;
    // type VMEdge = edges::PyPyEdge;
    type VMEdge = PyPyEdge;
    type VMMemorySlice = Range<Address>;
    // type VMMemorySlice = edges::PyPyMemorySlice;

    const MIN_ALIGNMENT: usize = 8;
    /// Allowed maximum alignment in bytes.
    const MAX_ALIGNMENT: usize = 1 << 6;
}

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

/// This is used to ensure we initialize MMTk at a specified timing.
pub static MMTK_INITIALIZED: AtomicBool = AtomicBool::new(false);

lazy_static! {
    pub static ref BUILDER: Mutex<MMTKBuilder> = Mutex::new(MMTKBuilder::new());
    pub static ref SINGLETON: MMTK<PyPy> = {
        let builder = BUILDER.lock().unwrap();
        debug_assert!(!MMTK_INITIALIZED.load(Ordering::SeqCst));
        let ret = mmtk::memory_manager::mmtk_init(&builder);
        MMTK_INITIALIZED.store(true, std::sync::atomic::Ordering::Relaxed);
        *ret
    };
}

lazy_static! {
    static ref CODE_CACHE_ROOTS: Mutex<HashMap<Address, Vec<Address>>> = Mutex::new(HashMap::new());
}

static CODE_CACHE_ROOTS_SIZE: AtomicUsize = AtomicUsize::new(0);
