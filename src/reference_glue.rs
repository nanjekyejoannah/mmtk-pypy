use mmtk::util::opaque_pointer::*;
use mmtk::util::ObjectReference;
use mmtk::vm::ReferenceGlue;
use mmtk::TraceLocal;
use PyPy;

pub struct VMReferenceGlue {}

impl ReferenceGlue<PyPy> for VMReferenceGlue {
    fn set_referent(_reff: ObjectReference, _referent: ObjectReference) {
        unimplemented!()
    }
    fn get_referent(_object: ObjectReference) -> ObjectReference {
        unimplemented!()
    }
    fn process_reference<T: TraceLocal>(
        _trace: &mut T,
        _reference: ObjectReference,
        _tls: VMWorkerThread,
    ) -> ObjectReference {
        unimplemented!()
    }
}
