use ipis::{core::signed::IsSigned, object::data::ObjectData, pin::PinnedInner};
use ipwis_kernel_common::data::{ExternData, ExternDataRef};

#[allow(clippy::missing_safety_doc)]
pub(crate) mod memory {
    use std::alloc;

    #[no_mangle]
    pub unsafe extern "C" fn __alloc(size: usize, align: usize) -> *mut u8 {
        alloc::alloc(alloc::Layout::from_size_align_unchecked(size, align))
    }

    #[no_mangle]
    pub unsafe extern "C" fn __alloc_zeroed(size: usize, align: usize) -> *mut u8 {
        alloc::alloc_zeroed(alloc::Layout::from_size_align_unchecked(size, align))
    }

    #[no_mangle]
    pub unsafe extern "C" fn __dealloc(ptr: *mut u8, size: usize, align: usize) {
        alloc::dealloc(ptr, alloc::Layout::from_size_align_unchecked(size, align))
    }

    #[no_mangle]
    pub unsafe extern "C" fn __realloc(
        ptr: *mut u8,
        size: usize,
        align: usize,
        new_size: usize,
    ) -> *mut u8 {
        alloc::realloc(
            ptr,
            alloc::Layout::from_size_align_unchecked(size, align),
            new_size,
        )
    }
}

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn __ipwis_syscall(
    _handler: ExternDataRef,
    inputs: ExternDataRef,
    outputs: ExternDataRef,
    errors: ExternDataRef,
) -> ExternDataRef {
    let inputs = inputs as *const ExternData;
    let outputs = outputs as *mut ExternData;
    let errors = errors as *mut ExternData;

    let inputs: ObjectData = PinnedInner::deserialize_owned((*inputs).into_slice()).unwrap();

    {
        println!("{:?}", inputs);
    }

    let outputs_buf = Box::leak(inputs.to_bytes().unwrap().into_boxed_slice());
    (*outputs).ptr = outputs_buf.as_ptr() as ExternDataRef;
    (*outputs).len = outputs_buf.len() as ExternDataRef;

    0
}
