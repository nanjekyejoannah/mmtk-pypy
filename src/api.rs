use crate::PyPy;
use crate::BUILDER;
use crate::SINGLETON;
use crate::UPCALLS;
use libc::c_char;
use libc::c_void;
use mmtk::memory_manager;
use mmtk::plan::BarrierSelector;
use mmtk::scheduler::GCController;
use mmtk::scheduler::GCWorker;
use mmtk::util::alloc::AllocatorSelector;
use mmtk::util::constants::LOG_BYTES_IN_ADDRESS;
use mmtk::util::opaque_pointer::*;
use mmtk::util::{Address, ObjectReference};
use mmtk::AllocationSemantics;
use mmtk::Mutator;
use mmtk::MutatorContext;
use once_cell::sync;
use std::cell::RefCell;
use std::ffi::{CStr, CString};
use std::sync::atomic::Ordering;

static NO_BARRIER: sync::Lazy<CString> = sync::Lazy::new(|| CString::new("NoBarrier").unwrap());
static OBJECT_BARRIER: sync::Lazy<CString> =
    sync::Lazy::new(|| CString::new("ObjectBarrier").unwrap());

#[no_mangle]
pub extern "C" fn get_mmtk_version() -> *const c_char {
    crate::build_info::MMTK_PYPY_FULL_VERSION.as_ptr() as _
}

#[no_mangle]
pub extern "C" fn mmtk_active_barrier() -> *const c_char {
    match SINGLETON.get_plan().constraints().barrier {
        BarrierSelector::NoBarrier => NO_BARRIER.as_ptr(),
        BarrierSelector::ObjectBarrier => OBJECT_BARRIER.as_ptr(),
        #[allow(unreachable_patterns)]
        _ => unimplemented!(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn release_buffer(ptr: *mut Address, length: usize, capacity: usize) {
    let _vec = Vec::<Address>::from_raw_parts(ptr, length, capacity);
}

#[no_mangle]
pub extern "C" fn pypy_gc_init() {
    // unsafe { UPCALLS = calls };
    crate::abi::validate_memory_layouts();

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
        } else if cfg!(feature = "immix") {
            Some(PlanSelector::Immix)
        } else {
            None
        };
        if let Some(plan) = force_plan {
            BUILDER.lock().unwrap().options.plan.set(plan);
        }
    }

    assert!(!crate::MMTK_INITIALIZED.load(Ordering::SeqCst));
    lazy_static::initialize(&SINGLETON);
}

#[no_mangle]
pub extern "C" fn pypy_is_gc_initialized() -> bool {
    crate::MMTK_INITIALIZED.load(std::sync::atomic::Ordering::SeqCst)
}

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
    let tls = VMMutatorThread(VMThread::UNINITIALIZED);
    Box::into_raw(memory_manager::bind_mutator(&SINGLETON, tls)) as *mut c_void
}

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn destroy_mutator(mutatorptr: *mut c_void) {
    let mutator = mutatorptr as *mut Mutator<PyPy>;
    memory_manager::destroy_mutator(unsafe { &mut *mutator })
}

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn flush_mutator(mutatorptr: *mut c_void) {
    let mutator = mutatorptr as *mut Mutator<PyPy>;
    memory_manager::flush_mutator(unsafe { &mut *mutator })
}

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn mmtk_alloc(
    mutator: *mut c_void,
    size: usize,
    align: usize,
    offset: isize,
) -> Address {
    let allocator = AllocationSemantics::Default;
    let mutt = mutator as  *mut Mutator<PyPy>;

    memory_manager::alloc::<PyPy>(unsafe { &mut *mutt }, size, align, offset, allocator)
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
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn post_alloc(
    mutt: *mut c_void,
    reff: *mut c_void,
    bytes: usize,
) {
    let mutator = mutt as *mut Mutator<PyPy>;
    let refer = mem::transmute(ObjectReference);
    let allocator = AllocationSemantics::Default;
    memory_manager::post_alloc::<PyPy>(unsafe { &mut *mutator }, refer, bytes, allocator)
}

#[no_mangle]
pub extern "C" fn will_never_move(object: *mut c_void) -> bool {
    !object.is_movable()
}

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn start_control_collector(
    tls: VMWorkerThread,
    gc_controller: *mut GCController<PyPy>,
) {
    let mut gc_controller = unsafe { Box::from_raw(gc_controller) };
    memory_manager::start_control_collector(&SINGLETON, tls, &mut gc_controller);
}

#[no_mangle]
// We trust the worker pointer is valid.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn start_worker(tls: VMWorkerThread, worker: *mut GCWorker<PyPy>) {
    let mut worker = unsafe { Box::from_raw(worker) };
    memory_manager::start_worker::<PyPy>(&SINGLETON, tls, &mut worker)
}

#[no_mangle]
pub extern "C" fn initialize_collection(tls: VMThread) {
    memory_manager::initialize_collection(&SINGLETON, tls)
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
pub extern "C" fn mmtk_is_live_object(object: *mut c_void) -> bool{
    memory_manager::is_live_object(mem::transmute(object))
}

#[cfg(feature = "is_mmtk_object")]
#[no_mangle]
pub extern "C" fn mmtk_is_mmtk_object(addr: *mut c_void) -> bool {
    memory_manager::is_mmtk_object(mem::transmute(addr))
}

#[no_mangle]
#[cfg(feature = "sanity")]
pub extern "C" fn scan_region() {
    memory_manager::scan_region(&SINGLETON)
}

#[no_mangle]
pub extern "C" fn handle_user_collection_request(tls: VMMutatorThread) {
    memory_manager::handle_user_collection_request::<PyPy>(&SINGLETON, tls);
}

#[no_mangle]
pub extern "C" fn is_in_mmtk_spaces(object: *mut c_void) -> bool {
    memory_manager::is_in_mmtk_spaces::<PyPy>(mem::transmute(object))
}

#[no_mangle]
pub extern "C" fn is_mapped_address(addr: Address) -> bool {
    memory_manager::is_mapped_address(addr)
}

#[no_mangle]
pub extern "C" fn modify_check(object: *mut c_void) {
    memory_manager::modify_check(&SINGLETON, mem::transmute(object))
}

#[no_mangle]
pub extern "C" fn mmtk_handle_user_collection_request(tls: *mut c_void) {
    memory_manager::handle_user_collection_request::<DummyVM>(&SINGLETON, mem::transmute(tls));
}

#[no_mangle]
pub extern "C" fn add_weak_candidate(objectref: *mut c_void) {
    let reff = objectref as ObjectReference;
    memory_manager::add_weak_candidate(&SINGLETON, reff)
}

#[no_mangle]
pub extern "C" fn harness_begin(_id: usize) {
    unsafe { ((*UPCALLS).harness_begin)() };
}

#[no_mangle]
pub extern "C" fn mmtk_harness_begin_impl() {
    memory_manager::harness_begin(&SINGLETON, VMMutatorThread(VMThread::UNINITIALIZED));
}

#[no_mangle]
pub extern "C" fn harness_end(_id: usize) {
    unsafe { ((*UPCALLS).harness_end)() };
}

#[no_mangle]
pub extern "C" fn mmtk_harness_end_impl() {
    memory_manager::harness_end(&SINGLETON);
}

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn process(name: *const c_char, value: *const c_char) -> bool {
    let name_str: &CStr = unsafe { CStr::from_ptr(name) };
    let value_str: &CStr = unsafe { CStr::from_ptr(value) };
    let mut builder = BUILDER.lock().unwrap();
    memory_manager::process(
        &mut builder,
        name_str.to_str().unwrap(),
        value_str.to_str().unwrap(),
    )
}

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn process_bulk(options: *const c_char) -> bool {
    let options_str: &CStr = unsafe { CStr::from_ptr(options) };
    let mut builder = BUILDER.lock().unwrap();
    memory_manager::process_bulk(&mut builder, options_str.to_str().unwrap())
}

#[no_mangle]
pub extern "C" fn starting_heap_address() -> *mut c_void {
    mem::transmute(memory_manager::starting_heap_address())
}

#[no_mangle]
pub extern "C" fn last_heap_address() -> *mut c_void {
    mem::transmute(memory_manager::last_heap_address())
}

#[no_mangle]
pub extern "C" fn pypy_max_capacity() -> usize {
    memory_manager::total_bytes(&SINGLETON)
}

#[no_mangle]
pub extern "C" fn executable() -> bool {
    true
}

#[no_mangle]
pub extern "C" fn mmtk_object_reference_write_pre(
    mutator: *mut c_void,
    src: *mut c_void,
    slot: *mut c_void,
    target: *mut c_void,
) {
    mutator
        .barrier()
        .object_reference_write_pre(mem::transmute(src), mem::transmute(slot), mem::transmute(target));
}

#[no_mangle]
pub extern "C" fn mmtk_object_reference_write_post(
    mutator: *mut c_void,
    src: *mut c_void,
    slot: *mut c_void,
    target: *mut c_void,
) {
    mutator
        .barrier()
        .object_reference_write_post(mem::transmute(src), mem::transmute(slot), mem::transmute(target));
}

#[no_mangle]
pub extern "C" fn mmtk_object_reference_write_slow(
    mutator: *mut c_void,
    src: *mut c_void,
    slot: *mut c_void,
    target: *mut c_void,
) {
    mutator
        .barrier()
        .object_reference_write_slow(mem::transmute(src), mem::transmute(slot), mem::transmute(target));
}

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

#[no_mangle]
pub extern "C" fn add_finalizer(objectref: *mut c_void) {
    let object = objectref as ObjectReference;
    memory_manager::add_finalizer(&SINGLETON, object);
}

#[no_mangle]
pub extern "C" fn get_finalized_object() -> ObjectReference {
    let res = match memory_manager::get_finalized_object(&SINGLETON) {
        Some(obj) => obj,
        None => ObjectReference::NULL,
    } as *mut c_void;

    return res;
} 
