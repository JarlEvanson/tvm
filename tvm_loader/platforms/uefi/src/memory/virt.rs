//! Implementation of [`LoaderAddressSpace`] for UEFI systems.

use tvm_loader::{
    memory::virt::{MapError, NoMapping, NotMapped, ProtectionFlags, loader::LoaderAddressSpace},
    unsafe_loader_address_space,
};

unsafe_loader_address_space!(UefiAddressSpace);

/// The [`LoaderAddressSpace`] implementation for UEFI systems.
struct UefiAddressSpace;

// SAFETY:
//
// The semantics of this method are implemented to be safe when properly using the APIs offered by
// `tvm_loader`.
unsafe impl LoaderAddressSpace for UefiAddressSpace {
    fn map(
        &self,
        physical_address: u64,
        _count: usize,
        _protection: ProtectionFlags,
    ) -> Result<usize, MapError> {
        // TODO: Check if the physical memory region is in the UEFI memory map.

        physical_address
            .try_into()
            .map_err(|_| MapError::InvalidAddress)
    }

    unsafe fn unmap(&self, _virtual_address: usize, _count: usize) -> Result<(), NotMapped> {
        // TODO: Check if the virtual memory region is in the UEFI memory map.

        Ok(())
    }

    fn translate_virt(&self, virtual_address: usize) -> Result<u64, NoMapping> {
        // TODO: Check if `virtual_address` is in the UEFI memory map.

        virtual_address.try_into().map_err(|_| NoMapping)
    }

    fn translate_phys(&self, physical_address: u64) -> Result<usize, NoMapping> {
        // TODO: Check if `physical_address` is in the UEFI memory map.

        physical_address.try_into().map_err(|_| NoMapping)
    }

    fn page_size(&self) -> usize {
        // The UEFI specification requires 4KiB pages for all supported architectures thus far.

        4096
    }
}
