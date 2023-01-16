use crate::gc_work::*;
use crate::{EdgesClosure, PyPy};
use crate::{NewBuffer, PyPyEdge, SINGLETON, UPCALLS};
use mmtk::memory_manager;
use mmtk::scheduler::WorkBucketStage;
use mmtk::util::opaque_pointer::*;
use mmtk::util::{Address, ObjectReference};
use mmtk::vm::{EdgeVisitor, RootsWorkFactory, Scanning};
use mmtk::Mutator;
use mmtk::MutatorContext;

pub struct VMScanning {}

const WORK_PACKET_CAPACITY: usize = 4096;

extern "C" fn report_edges_and_renew_buffer<F: RootsWorkFactory<PyPyEdge>>(
    ptr: *mut Address,
    length: usize,
    capacity: usize,
    factory_ptr: *mut libc::c_void,
) -> NewBuffer {
    if !ptr.is_null() {
        let buf = unsafe { Vec::<Address>::from_raw_parts(ptr, length, capacity) };
        let factory: &mut F = unsafe { &mut *(factory_ptr as *mut F) };
        factory.create_process_edge_roots_work(buf);
    }
    let (ptr, _, capacity) = {
        use std::mem::ManuallyDrop;
        let new_vec = Vec::with_capacity(WORK_PACKET_CAPACITY);
        let mut me = ManuallyDrop::new(new_vec);
        (me.as_mut_ptr(), me.len(), me.capacity())
    };
    NewBuffer { ptr, capacity }
}

pub(crate) fn to_edges_closure<F: RootsWorkFactory<PyPyEdge>>(factory: &mut F) -> EdgesClosure {
    EdgesClosure {
        func: report_edges_and_renew_buffer::<F>,
        data: factory as *mut F as *mut libc::c_void,
    }
}

impl Scanning<PyPy> for VMScanning {
    const SCAN_MUTATORS_IN_SAFEPOINT: bool = false;
    const SINGLE_THREAD_MUTATOR_SCANNING: bool = false;

    fn scan_object<EV: EdgeVisitor<PyPyEdge>>(
        tls: VMWorkerThread,
        object: ObjectReference,
        edge_visitor: &mut EV,
    ) {
        crate::object_scanning::scan_object(object, edge_visitor, tls)
    }

    fn notify_initial_thread_scan_complete(_partial_scan: bool, _tls: VMWorkerThread) {
        // unimplemented!()
        // TODO
    }

    fn scan_thread_roots(_tls: VMWorkerThread, mut factory: impl RootsWorkFactory<PyPyEdge>) {
        unsafe {
            ((*UPCALLS).scan_all_thread_roots)(to_edges_closure(&mut factory));
        }
    }

    fn scan_thread_root(
        _tls: VMWorkerThread,
        mutator: &'static mut Mutator<PyPy>,
        mut factory: impl RootsWorkFactory<PyPyEdge>,
    ) {
        let tls = mutator.get_tls();
        unsafe {
            ((*UPCALLS).scan_thread_roots)(to_edges_closure(&mut factory), tls);
        }
    }

    fn scan_vm_specific_roots(_tls: VMWorkerThread, factory: impl RootsWorkFactory<PyPyEdge>) {
        memory_manager::add_work_packets(
            &SINGLETON,
            WorkBucketStage::Prepare,
            vec![
                Box::new(ScanUniverseRoots::new(factory.clone())) as _,
                Box::new(ScanJNIHandlesRoots::new(factory.clone())) as _,
                Box::new(ScanObjectSynchronizerRoots::new(factory.clone())) as _,
                Box::new(ScanManagementRoots::new(factory.clone())) as _,
                Box::new(ScanJvmtiExportRoots::new(factory.clone())) as _,
                Box::new(ScanAOTLoaderRoots::new(factory.clone())) as _,
                Box::new(ScanSystemDictionaryRoots::new(factory.clone())) as _,
                Box::new(ScanCodeCacheRoots::new(factory.clone())) as _,
                Box::new(ScanStringTableRoots::new(factory.clone())) as _,
                Box::new(ScanClassLoaderDataGraphRoots::new(factory.clone())) as _,
                Box::new(ScanWeakProcessorRoots::new(factory.clone())) as _,
            ],
        );
        if !(Self::SCAN_MUTATORS_IN_SAFEPOINT && Self::SINGLE_THREAD_MUTATOR_SCANNING) {
            memory_manager::add_work_packet(
                &SINGLETON,
                WorkBucketStage::Prepare,
                ScanVMThreadRoots::new(factory),
            );
        }
    }

    fn supports_return_barrier() -> bool {
        unimplemented!()
    }

    fn prepare_for_roots_re_scanning() {
        unsafe {
            ((*UPCALLS).prepare_for_roots_re_scanning)();
        }
    }
}
