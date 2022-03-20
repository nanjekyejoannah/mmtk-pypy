use mmtk::memory_manager;
use mmtk::util::opaque_pointer::*;
use mmtk::util::Address;
use mmtk::AllocationSemantics;
use mmtk::Mutator;
use mmtk::MMTK;
use std::ptr::null_mut;
use api::memory_manager::bind_mutator;
use libc::c_void;

// use PyPy_Upcalls;
use UPCALLS;
use PyPy;

#[no_mangle]
pub extern "C" fn pypy_gc_init(heap_size: isize ){
    unsafe {
        UPCALLS =  null_mut();
    };
    let mmtk: Box<MMTK<PyPy>> = Box::new(MMTK::new());
    let mmtk: *mut MMTK<PyPy> = Box::into_raw(mmtk);
    memory_manager::gc_init(unsafe { &mut *mmtk }, heap_size as usize);
}


#[no_mangle]
pub extern "C" fn pypy_bind_mutator() -> *mut c_void {
    let mttk: Box<MMTK<PyPy>> = Box::new(MMTK::new());
    let mttk: *mut MMTK<PyPy> = Box::into_raw(mttk);
    let handle = bind_mutator(unsafe { &mut *mttk }, VMMutatorThread(VMThread::UNINITIALIZED));
    let handle_ptr: *mut c_void = Box::into_raw(handle) as *mut _ as *mut c_void;
    return handle_ptr
}

#[no_mangle]
pub extern "C" fn pypy_mmtk_alloc(handle: *mut c_void, heap_size: isize) {
    let handle: &mut Mutator<PyPy> = unsafe { &mut *(handle as *mut Mutator<PyPy>) };
    alloc(unsafe { &mut *handle }, heap_size as usize, 8, 0, AllocationSemantics::Default);
}

#[no_mangle]
// It is fine we turn the pointer back to box, as we turned a boxed value to the raw pointer in bind_mutator()
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn destroy_mutator(mutator: *mut Mutator<PyPy>) {
    memory_manager::destroy_mutator(unsafe { Box::from_raw(mutator) })
}

#[no_mangle]
pub extern "C" fn alloc(
    mutator: &mut Mutator<PyPy>,
    size: usize,
    align: usize,
    offset: isize,
    semantics: AllocationSemantics,
) -> Address {
    memory_manager::alloc::<PyPy>(mutator, size, align, offset, semantics)
}

