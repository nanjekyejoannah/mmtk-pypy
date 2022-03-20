use mmtk::scheduler::GCWorker;
use mmtk::scheduler::ProcessEdgesWork;
use mmtk::util::opaque_pointer::*;
use mmtk::util::ObjectReference;
use mmtk::vm::Scanning;
use mmtk::{Mutator, TransitiveClosure};
use PyPy;

pub struct VMScanning {}

impl Scanning<PyPy> for VMScanning {
    const SCAN_MUTATORS_IN_SAFEPOINT: bool = false;
    const SINGLE_THREAD_MUTATOR_SCANNING: bool = false;

    fn scan_object<T: TransitiveClosure>(
        _trace: &mut T,
        _object: ObjectReference,
        _tls: VMWorkerThread,
    ) {
        unimplemented!()
    }

    fn notify_initial_thread_scan_complete(_partial_scan: bool, _tls: VMWorkerThread) {
        unimplemented!()
    }

    fn scan_objects<W: ProcessEdgesWork<VM = PyPy>>(
        _objects: &[ObjectReference],
        _worker: &mut GCWorker<PyPy>,
    ) {
        unimplemented!()
    }

    fn scan_thread_roots<W: ProcessEdgesWork<VM = PyPy>>() {
        unimplemented!()
    }

    fn scan_thread_root<W: ProcessEdgesWork<VM = PyPy>>(
        _mutator: &'static mut Mutator<PyPy>,
        _tls: VMWorkerThread,
    ) {
        unimplemented!()
    }

    fn scan_vm_specific_roots<W: ProcessEdgesWork<VM = PyPy>>() {
        unimplemented!()
    }

    fn supports_return_barrier() -> bool {
        unimplemented!()
    }

    fn prepare_for_roots_re_scanning() {
        unimplemented!()
    }
}
