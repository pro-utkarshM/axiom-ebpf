use alloc::vec::Vec;
use core::mem::{align_of, size_of};

use kernel_abi::{EFAULT, EINVAL, Errno};
use kernel_syscall::UserspacePtr;

/// Copy a struct from userspace to kernel.
/// Validates: non-null, canonical address, alignment, within userspace range.
pub fn copy_from_userspace<T: Copy>(ptr: usize) -> Result<T, Errno> {
    if ptr == 0 {
        return Err(EFAULT);
    }
    
    // Validate address is in userspace (lower half)
    let user_ptr = unsafe { UserspacePtr::<T>::try_from_usize(ptr)? };
    user_ptr.validate_range(size_of::<T>())?;
    
    // Validate alignment
    if ptr % align_of::<T>() != 0 {
        return Err(EINVAL);
    }
    
    // SAFETY: Address validated as userspace, aligned, and within bounds
    Ok(unsafe { *(ptr as *const T) })
}

/// Read a slice from userspace. Returns owned Vec.
pub fn read_userspace_slice(ptr: usize, len: usize) -> Result<Vec<u8>, Errno> {
    if ptr == 0 || len == 0 {
        return Err(EFAULT);
    }
    
    let user_ptr = unsafe { UserspacePtr::<u8>::try_from_usize(ptr)? };
    user_ptr.validate_range(len)?;
    
    // SAFETY: Address validated as userspace and within bounds
    let slice = unsafe { core::slice::from_raw_parts(ptr as *const u8, len) };
    Ok(slice.to_vec())
}

/// Copy data to userspace buffer.
pub fn copy_to_userspace(ptr: usize, data: &[u8]) -> Result<(), Errno> {
    if ptr == 0 {
        return Err(EFAULT);
    }
    
    let user_ptr = unsafe { UserspacePtr::<u8>::try_from_usize(ptr)? };
    user_ptr.validate_range(data.len())?;
    
    // SAFETY: Address validated as userspace and within bounds
    unsafe {
        core::ptr::copy_nonoverlapping(data.as_ptr(), ptr as *mut u8, data.len())
    }
    Ok(())
}
