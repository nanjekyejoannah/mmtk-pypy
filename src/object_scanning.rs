use crate::SINGLETON;

use crate::abi::*;
use crate::{PyPyEdge, UPCALLS};
use mmtk::util::constants::*;
use mmtk::util::opaque_pointer::*;
use mmtk::util::{Address, ObjectReference};
use mmtk::vm::EdgeVisitor;
use std::{mem, slice};

trait OopIterate: Sized {
    fn oop_iterate(&self, oop: Oop, closure: &mut impl EdgeVisitor<PyPyEdge>);
}

impl OopIterate for OopMapBlock {
    #[inline]
    fn oop_iterate(&self, oop: Oop, closure: &mut impl EdgeVisitor<PyPyEdge>) {
        let start = oop.get_field_address(self.offset);
        for i in 0..self.count as usize {
            let edge = start + (i << LOG_BYTES_IN_ADDRESS);
            closure.visit_edge(edge);
        }
    }
}

impl OopIterate for InstanceKlass {
    #[inline]
    fn oop_iterate(&self, oop: Oop, closure: &mut impl EdgeVisitor<PyPyEdge>) {
        let oop_maps = self.nonstatic_oop_maps();
        for map in oop_maps {
            map.oop_iterate(oop, closure)
        }
    }
}

impl OopIterate for InstanceMirrorKlass {
    #[inline]
    fn oop_iterate(&self, oop: Oop, closure: &mut impl EdgeVisitor<PyPyEdge>) {
        self.instance_klass.oop_iterate(oop, closure);

        // static fields
        let start: *const Oop = Self::start_of_static_fields(oop).to_ptr::<Oop>();
        let len = Self::static_oop_field_count(oop);
        let slice = unsafe { slice::from_raw_parts(start, len as _) };
        for oop in slice {
            closure.visit_edge(Address::from_ref(oop as &Oop));
        }
    }
}

impl OopIterate for InstanceClassLoaderKlass {
    #[inline]
    fn oop_iterate(&self, oop: Oop, closure: &mut impl EdgeVisitor<PyPyEdge>) {
        self.instance_klass.oop_iterate(oop, closure);
    }
}

impl OopIterate for ObjArrayKlass {
    #[inline]
    fn oop_iterate(&self, oop: Oop, closure: &mut impl EdgeVisitor<PyPyEdge>) {
        let array = unsafe { oop.as_array_oop() };
        for oop in unsafe { array.data::<Oop>(BasicType::T_OBJECT) } {
            closure.visit_edge(Address::from_ref(oop as &Oop));
        }
    }
}

impl OopIterate for TypeArrayKlass {
    #[inline]
    fn oop_iterate(&self, _oop: Oop, _closure: &mut impl EdgeVisitor<PyPyEdge>) {
    }
}

impl OopIterate for InstanceRefKlass {
    #[inline]
    fn oop_iterate(&self, oop: Oop, closure: &mut impl EdgeVisitor<PyPyEdge>) {
        use crate::abi::*;
        use crate::api::{add_phantom_candidate, add_soft_candidate, add_weak_candidate};
        self.instance_klass.oop_iterate(oop, closure);

        if Self::should_scan_weak_refs() {
            let reference = ObjectReference::from(oop);
            match self.instance_klass.reference_type {
                ReferenceType::None => {
                    panic!("oop_iterate on InstanceRefKlass with reference_type as None")
                }
                ReferenceType::Weak => add_weak_candidate(reference),
                ReferenceType::Soft => add_soft_candidate(reference),
                ReferenceType::Phantom => add_phantom_candidate(reference),
                // Process these two types normally (as if they are strong refs)
                // We will handle final reference later
                ReferenceType::Final | ReferenceType::Other => {
                    Self::process_ref_as_strong(oop, closure)
                }
            }
        } else {
            Self::process_ref_as_strong(oop, closure);
        }
    }
}

impl InstanceRefKlass {
    #[inline]
    fn should_scan_weak_refs() -> bool {
        !*SINGLETON.get_options().no_reference_types
    }
    #[inline]
    fn process_ref_as_strong(oop: Oop, closure: &mut impl EdgeVisitor<PyPyEdge>) {
        let referent_addr = Self::referent_address(oop);
        closure.visit_edge(referent_addr);
        let discovered_addr = Self::discovered_address(oop);
        closure.visit_edge(discovered_addr);
    }
}

#[allow(unused)]
fn oop_iterate_slow(oop: Oop, closure: &mut impl EdgeVisitor<PyPyEdge>, tls: OpaquePointer) {
    unsafe {
        ((*UPCALLS).scan_object)(closure as *mut _ as _, mem::transmute(oop), tls);
    }
}

#[inline]
fn oop_iterate(oop: Oop, closure: &mut impl EdgeVisitor<PyPyEdge>) {
    let klass_id = oop.klass.id;
    debug_assert!(
        klass_id as i32 >= 0 && (klass_id as i32) < 6,
        "Invalid klass-id: {:x} for oop: {:x}",
        klass_id as i32,
        unsafe { mem::transmute::<Oop, ObjectReference>(oop) }
    );
    match klass_id {
        KlassID::Instance => {
            let instance_klass = unsafe { oop.klass.cast::<InstanceKlass>() };
            instance_klass.oop_iterate(oop, closure);
        }
        KlassID::InstanceClassLoader => {
            let instance_klass = unsafe { oop.klass.cast::<InstanceClassLoaderKlass>() };
            instance_klass.oop_iterate(oop, closure);
        }
        KlassID::InstanceMirror => {
            let instance_klass = unsafe { oop.klass.cast::<InstanceMirrorKlass>() };
            instance_klass.oop_iterate(oop, closure);
        }
        KlassID::ObjArray => {
            let array_klass = unsafe { oop.klass.cast::<ObjArrayKlass>() };
            array_klass.oop_iterate(oop, closure);
        }
        KlassID::TypeArray => {
            let array_klass = unsafe { oop.klass.cast::<TypeArrayKlass>() };
            array_klass.oop_iterate(oop, closure);
        }
        KlassID::InstanceRef => {
            let instance_klass = unsafe { oop.klass.cast::<InstanceRefKlass>() };
            instance_klass.oop_iterate(oop, closure);
        }
    }
}

#[inline]
pub fn scan_object(
    object: ObjectReference,
    closure: &mut impl EdgeVisitor<PyPyEdge>,
    _tls: VMWorkerThread,
) {
    unsafe { oop_iterate(mem::transmute(object), closure) }
}
