use mmtk::vm::*;

#[cfg(target_pointer_width = "64")]
pub(crate) const FORWARDING_BITS_OFFSET: isize = 56;
#[cfg(target_pointer_width = "32")]
pub(crate) const FORWARDING_BITS_OFFSET: isize = unimplemenged!();

pub(crate) const FORWARDING_POINTER_OFFSET: isize = 0;


pub(crate) const LOGGING_SIDE_METADATA_SPEC: VMGlobalLogBitSpec = VMGlobalLogBitSpec::side_first();


pub(crate) const FORWARDING_POINTER_METADATA_SPEC: VMLocalForwardingPointerSpec =
    VMLocalForwardingPointerSpec::in_header(FORWARDING_POINTER_OFFSET);


pub(crate) const FORWARDING_BITS_METADATA_SPEC: VMLocalForwardingBitsSpec =
    VMLocalForwardingBitsSpec::in_header(FORWARDING_BITS_OFFSET);

#[cfg(feature = "mark_bit_in_header")]
pub(crate) const MARKING_METADATA_SPEC: VMLocalMarkBitSpec =
    VMLocalMarkBitSpec::in_header(FORWARDING_BITS_OFFSET);

#[cfg(not(feature = "mark_bit_in_header"))]
pub(crate) const MARKING_METADATA_SPEC: VMLocalMarkBitSpec =
    VMLocalMarkBitSpec::side_after(LOS_METADATA_SPEC.as_spec());

pub(crate) const LOS_METADATA_SPEC: VMLocalLOSMarkNurserySpec =
    VMLocalLOSMarkNurserySpec::side_first();

