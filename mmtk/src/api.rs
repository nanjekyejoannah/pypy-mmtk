// All functions here are extern function. There is no point for marking them as unsafe.
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use libc::c_char;
use libc::c_void;
use std::mem;
use std::sync::atomic::Ordering;
use std::ffi::{CStr, CString};
use mmtk::memory_manager;
use mmtk::plan::BarrierSelector;
use mmtk::util::alloc::AllocatorSelector;
use mmtk::util::constants::LOG_BYTES_IN_ADDRESS;
use mmtk::AllocationSemantics;
use mmtk::util::{Address, ObjectReference};
use mmtk::util::opaque_pointer::*;
use mmtk::scheduler::{GCController, GCWorker};
use mmtk::Mutator;
use crate::mmtk::MutatorContext;
use crate::PyPy;
use crate::SINGLETON;
use crate::BUILDER;
use once_cell::sync;
use std::cell::RefCell;


// Supported barriers:
static NO_BARRIER: sync::Lazy<CString> = sync::Lazy::new(|| CString::new("NoBarrier").unwrap());
static OBJECT_BARRIER: sync::Lazy<CString> =
    sync::Lazy::new(|| CString::new("ObjectBarrier").unwrap());

#[no_mangle]
pub extern "C" fn mmtk_active_barrier() -> *const c_char {
    match SINGLETON.get_plan().constraints().barrier {
        BarrierSelector::NoBarrier => NO_BARRIER.as_ptr(),
        BarrierSelector::ObjectBarrier => OBJECT_BARRIER.as_ptr(),
        // In case we have more barriers in mmtk-core.
        #[allow(unreachable_patterns)]
        _ => unimplemented!(),
    }
}

/// # Safety
/// Caller needs to make sure the ptr is a valid vector pointer.
#[no_mangle]
pub unsafe extern "C" fn release_buffer(ptr: *mut Address, length: usize, capacity: usize) {
    let _vec = Vec::<Address>::from_raw_parts(ptr, length, capacity);
}

#[no_mangle]
pub extern "C" fn pypy_gc_init() {
    println!("version!! ");
    {
        use mmtk::util::options::PlanSelector;
        let force_plan = if cfg!(feature = "nogc") {
            Some(PlanSelector::NoGC)
        } else if cfg!(feature = "semispace") {
            Some(PlanSelector::SemiSpace)
        } else if cfg!(feature = "gencopy") {
            Some(PlanSelector::GenCopy)
        } else if cfg!(feature = "marksweep") {
            Some(PlanSelector::MarkSweep)
        } else if cfg!(feature = "markcompact") {
            Some(PlanSelector::MarkCompact)
        } else {
            None
        };
        if let Some(plan) = force_plan {
            BUILDER.lock().unwrap().options.plan.set(plan);
        }
    }

    // Make sure MMTk has not yet been initialized
    assert!(!crate::MMTK_INITIALIZED.load(Ordering::SeqCst));
    // Initialize MMTk here
    lazy_static::initialize(&SINGLETON);
}

#[no_mangle]
pub extern "C" fn pypy_is_gc_initialized() -> bool {
    crate::MMTK_INITIALIZED.load(std::sync::atomic::Ordering::SeqCst)
}

// #[no_mangle]
// pub extern "C" fn mmtk_bind_mutator(tls: VMMutatorThread) -> *mut Mutator<PyPy> {
//     Box::into_raw(memory_manager::bind_mutator(&SINGLETON, tls))
// }

#[no_mangle]
pub extern "C" fn mmtk_set_heap_size(min: usize, max: usize) -> bool {
    use mmtk::util::options::GCTriggerSelector;
    let mut builder = BUILDER.lock().unwrap();
    let policy = if min == max {
        GCTriggerSelector::FixedHeapSize(min)
    } else {
        GCTriggerSelector::DynamicHeapSize(min, max)
    };
    builder.options.gc_trigger.set(policy)
}


#[no_mangle]
pub extern "C" fn bind_mutator() -> *mut c_void {
    println!("biinding2");
    let tls = VMMutatorThread(VMThread::UNINITIALIZED);
    Box::into_raw(memory_manager::bind_mutator(&SINGLETON, tls)) as *mut c_void
}


#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn destroy_mutator(mutatorptr: *mut c_void) {
    let mutator = mutatorptr as *mut Mutator<PyPy>;
    memory_manager::destroy_mutator(unsafe { &mut *mutator });
    let _ = unsafe { Box::from_raw(mutator) };
}

#[no_mangle]
// We trust the mutator pointer is valid.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn flush_mutator(mutatorptr: *mut c_void) {
    let mutator = mutatorptr as *mut Mutator<PyPy>;
    memory_manager::flush_mutator(unsafe { &mut *mutator })
}

#[no_mangle]
pub extern "C" fn alloc(
                    mutator: *mut c_void,
                    size: usize,
                    align: usize, 
                    offset: isize
                ) -> Address {
    // let semantics = AllocationSemantics::Default;
    let semantics = AllocationSemantics::Los;
    // if size >= SINGLETON.get_plan().constraints().max_non_los_default_alloc_bytes {
    //     semantics = AllocationSemantics::Los;
    // }
    let mutr = mutator as  *mut Mutator<PyPy>;
    memory_manager::alloc::<PyPy>(unsafe { &mut *mutr }, size, align, offset, semantics)
}

#[no_mangle]
pub extern "C" fn get_allocator_mapping(allocator: AllocationSemantics) -> AllocatorSelector {
    memory_manager::get_allocator_mapping(&SINGLETON, allocator)
}

#[no_mangle]
pub extern "C" fn get_max_non_los_default_alloc_bytes() -> usize {
    SINGLETON
        .get_plan()
        .constraints()
        .max_non_los_default_alloc_bytes
}

#[no_mangle]
pub extern "C" fn mmtk_post_alloc(mutt: *mut c_void, 
                                    refr: *mut c_void,
                                    bytes: usize, 
                                    ) {
    let semantics = AllocationSemantics::Los;
    println!("post_alloc ");
    // if bytes >= SINGLETON.get_plan().constraints().max_non_los_default_alloc_bytes {
    //     semantics = AllocationSemantics::Los;
    // }
    // let refer =  refr as *ObjectReference;
    let refer =  unsafe {mem::transmute(refr)};
    let mutator = mutt as *mut Mutator<PyPy>;
    memory_manager::post_alloc::<PyPy>(unsafe { &mut *mutator }, refer, bytes, semantics)
}

#[no_mangle]
pub extern "C" fn mmtk_will_never_move(object: ObjectReference) -> bool {
    !object.is_movable()
}

#[no_mangle]
pub extern "C" fn mmtk_start_control_collector(tls: VMWorkerThread, controller: &'static mut GCController<PyPy>) {
    memory_manager::start_control_collector(&SINGLETON, tls, controller);
}

#[no_mangle]
pub extern "C" fn mmtk_start_worker(tls: VMWorkerThread, worker: &'static mut GCWorker<PyPy>) {
    memory_manager::start_worker::<PyPy>(&SINGLETON, tls, worker)
}

#[no_mangle]
pub extern "C" fn mmtk_initialize_collection(tls: VMThread) {
    memory_manager::initialize_collection(&SINGLETON, tls)
}

#[no_mangle]
pub extern "C" fn mmtk_disable_collection() {
    memory_manager::disable_collection(&SINGLETON)
}

#[no_mangle]
pub extern "C" fn mmtk_enable_collection() {
    memory_manager::enable_collection(&SINGLETON)
}

#[no_mangle]
pub extern "C" fn mmtk_used_bytes() -> usize {
    memory_manager::used_bytes(&SINGLETON)
}

#[no_mangle]
pub extern "C" fn mmtk_free_bytes() -> usize {
    memory_manager::free_bytes(&SINGLETON)
}

#[no_mangle]
pub extern "C" fn mmtk_total_bytes() -> usize {
    memory_manager::total_bytes(&SINGLETON)
}

#[no_mangle]
#[cfg(feature = "sanity")]
pub extern "C" fn scan_region() {
    memory_manager::scan_region(&SINGLETON)
}

#[no_mangle]
pub extern "C" fn mmtk_is_live_object(object: ObjectReference) -> bool{
    memory_manager::is_live_object(object)
}

#[cfg(feature = "is_mmtk_object")]
#[no_mangle]
pub extern "C" fn mmtk_is_mmtk_object(addr: Address) -> bool {
    memory_manager::is_mmtk_object(addr)
}

#[no_mangle]
pub extern "C" fn mmtk_is_in_mmtk_spaces(object: ObjectReference) -> bool {
    memory_manager::is_in_mmtk_spaces::<PyPy>(object)
}

#[no_mangle]
pub extern "C" fn mmtk_is_mapped_address(address: Address) -> bool {
    memory_manager::is_mapped_address(address)
}

#[no_mangle]
pub extern "C" fn mmtk_modify_check(object: ObjectReference) {
    memory_manager::modify_check(&SINGLETON, object)
}

#[no_mangle]
pub extern "C" fn mmtk_handle_user_collection_request(tls: VMMutatorThread) {
    memory_manager::handle_user_collection_request::<PyPy>(&SINGLETON, tls);
}

#[no_mangle]
pub extern "C" fn mmtk_add_weak_candidate(reff: ObjectReference) {
    memory_manager::add_weak_candidate(&SINGLETON, reff)
}

#[no_mangle]
pub extern "C" fn mmtk_add_soft_candidate(reff: ObjectReference) {
    memory_manager::add_soft_candidate(&SINGLETON, reff)
}

#[no_mangle]
pub extern "C" fn mmtk_add_phantom_candidate(reff: ObjectReference) {
    memory_manager::add_phantom_candidate(&SINGLETON, reff)
}

#[no_mangle]
pub extern "C" fn mmtk_harness_begin(tls: VMMutatorThread) {
    memory_manager::harness_begin(&SINGLETON, tls)
}

#[no_mangle]
pub extern "C" fn mmtk_harness_end() {
    memory_manager::harness_end(&SINGLETON)
}

#[no_mangle]
pub extern "C" fn mmtk_process(name: *const c_char, value: *const c_char) -> bool {
    let name_str: &CStr = unsafe { CStr::from_ptr(name) };
    let value_str: &CStr = unsafe { CStr::from_ptr(value) };
    let mut builder = BUILDER.lock().unwrap();
    memory_manager::process(&mut builder, name_str.to_str().unwrap(), value_str.to_str().unwrap())
}

#[no_mangle]
// We trust the name/value pointer is valid.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn process_bulk(options: *const c_char) -> bool {
    let options_str: &CStr = unsafe { CStr::from_ptr(options) };
    let mut builder = BUILDER.lock().unwrap();
    memory_manager::process_bulk(&mut builder, options_str.to_str().unwrap())
}

#[no_mangle]
pub extern "C" fn mmtk_starting_heap_address() -> Address {
    memory_manager::starting_heap_address()
}

#[no_mangle]
pub extern "C" fn mmtk_last_heap_address() -> Address {
    memory_manager::last_heap_address()
}

#[no_mangle]
pub extern "C" fn pypy_max_capacity() -> usize {
    memory_manager::total_bytes(&SINGLETON)
}

#[no_mangle]
pub extern "C" fn executable() -> bool {
    true
}

/// Full pre barrier
#[no_mangle]
pub extern "C" fn mmtk_object_reference_write_pre(
    mutr: *mut c_void,
    src: ObjectReference,
    slot: Address,
    target: ObjectReference,
) {
    // mutator: &'static mut Mutator<PyPy>,
    let mutor = mutr as  *mut Mutator<PyPy>;
    let mutator =unsafe { &mut *mutor };
    mutator
        .barrier()
        .object_reference_write_pre(src, slot, target);
}

/// Full post barrier
#[no_mangle]
pub extern "C" fn mmtk_object_reference_write_post(
    mutator: &'static mut Mutator<PyPy>,
    src: ObjectReference,
    slot: Address,
    target: ObjectReference,
) {
    mutator
        .barrier()
        .object_reference_write_post(src, slot, target);
}

/// Barrier slow-path call
#[no_mangle]
pub extern "C" fn mmtk_object_reference_write_slow(
    mutator: &'static mut Mutator<PyPy>,
    src: ObjectReference,
    slot: Address,
    target: ObjectReference,
) {
    mutator
        .barrier()
        .object_reference_write_slow(src, slot, target);
}

/// Array-copy pre-barrier
#[no_mangle]
pub extern "C" fn mmtk_array_copy_pre(
    mutator: &'static mut Mutator<PyPy>,
    src: Address,
    dst: Address,
    count: usize,
) {
    let bytes = count << LOG_BYTES_IN_ADDRESS;
    mutator
        .barrier()
        .memory_region_copy_pre(src..src + bytes, dst..dst + bytes);
}

/// Array-copy post-barrier
#[no_mangle]
pub extern "C" fn mmtk_array_copy_post(
    mutator: &'static mut Mutator<PyPy>,
    src: Address,
    dst: Address,
    count: usize,
) {
    let bytes = count << LOG_BYTES_IN_ADDRESS;
    mutator
        .barrier()
        .memory_region_copy_post(src..src + bytes, dst..dst + bytes);
}

/// C2 Slowpath allocation barrier
#[no_mangle]
pub extern "C" fn mmtk_object_probable_write(
    mutator: &'static mut Mutator<PyPy>,
    obj: ObjectReference,
) {
    mutator.barrier().object_probable_write(obj);
}

// finalization
#[no_mangle]
pub extern "C" fn add_finalizer(obj: *mut c_void) {
    println!("add_finalizer ");
    let object =  unsafe {mem::transmute(obj)};
    memory_manager::add_finalizer(&SINGLETON, object);
}

#[no_mangle]
pub extern "C" fn get_finalized_object() -> ObjectReference {
    match memory_manager::get_finalized_object(&SINGLETON) {
        Some(obj) => obj,
        None => ObjectReference::NULL,
    }
}

thread_local! {
    /// Cache all the pointers reported by the current thread.
    static NMETHOD_SLOTS: RefCell<Vec<Address>> = RefCell::new(vec![]);
}

/// Report a list of pointers in nmethod to mmtk.
#[no_mangle]
pub extern "C" fn mmtk_add_nmethod_oop(addr: Address) {
    NMETHOD_SLOTS.with(|x| x.borrow_mut().push(addr))
}

/// Register a nmethod.
/// The c++ part of the binding should scan the nmethod and report all the pointers to mmtk first, before calling this function.
/// This function will transfer all the locally cached pointers of this nmethod to the global storage.
#[no_mangle]
pub extern "C" fn mmtk_register_nmethod(nm: Address) {
    let slots = NMETHOD_SLOTS.with(|x| {
        if x.borrow().len() == 0 {
            return None;
        }
        Some(x.replace(vec![]))
    });
    let slots = match slots {
        Some(slots) => slots,
        _ => return,
    };
    let mut roots = crate::CODE_CACHE_ROOTS.lock().unwrap();
    // Relaxed add instead of `fetch_add`, since we've already acquired the lock.
    crate::CODE_CACHE_ROOTS_SIZE.store(
        crate::CODE_CACHE_ROOTS_SIZE.load(Ordering::Relaxed) + slots.len(),
        Ordering::Relaxed,
    );
    roots.insert(nm, slots);
}

/// Unregister a nmethod.
#[no_mangle]
pub extern "C" fn mmtk_unregister_nmethod(nm: Address) {
    let mut roots = crate::CODE_CACHE_ROOTS.lock().unwrap();
    if let Some(slots) = roots.remove(&nm) {
        // Relaxed sub instead of `fetch_sub`, since we've already acquired the lock.
        crate::CODE_CACHE_ROOTS_SIZE.store(
            crate::CODE_CACHE_ROOTS_SIZE.load(Ordering::Relaxed) - slots.len(),
            Ordering::Relaxed,
        );
    }
}

#[no_mangle]
#[cfg(feature = "malloc_counted_size")]
pub extern "C" fn mmtk_counted_malloc(size: usize) -> Address {
    memory_manager::counted_malloc::<PyPy>(&SINGLETON, size)
}
#[no_mangle]
pub extern "C" fn mmtk_malloc(size: usize) -> Address {
    memory_manager::malloc(size)
}

#[no_mangle]
#[cfg(feature = "malloc_counted_size")]
pub extern "C" fn mmtk_counted_calloc(num: usize, size: usize) -> Address {
    memory_manager::counted_calloc::<PyPy>(&SINGLETON, num, size)
}
#[no_mangle]
pub extern "C" fn mmtk_calloc(num: usize, size: usize) -> Address {
    memory_manager::calloc(num, size)
}

#[no_mangle]
#[cfg(feature = "malloc_counted_size")]
pub extern "C" fn mmtk_realloc_with_old_size(addr: Address, size: usize, old_size: usize) -> Address {
    memory_manager::realloc_with_old_size::<PyPy>(&SINGLETON, addr, size, old_size)
}
#[no_mangle]
pub extern "C" fn mmtk_realloc(addr: Address, size: usize) -> Address {
    memory_manager::realloc(addr, size)
}

#[no_mangle]
#[cfg(feature = "malloc_counted_size")]
pub extern "C" fn mmtk_free_with_size(addr: Address, old_size: usize) {
    memory_manager::free_with_size::<PyPy>(&SINGLETON, addr, old_size)
}
#[no_mangle]
pub extern "C" fn mmtk_free(addr: Address) {
    memory_manager::free(addr)
}
