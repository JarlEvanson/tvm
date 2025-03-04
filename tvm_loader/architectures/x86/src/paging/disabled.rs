//! Implementation of disabled paging.

use tvm_loader::memory::virt::{
    MapError, NoMapping, NotMapped, ProtectionFlags, arch::AddressSpace,
};
use tvm_loader_x86_common::paging::X86CommonAddressSpace;
use x86_common::PagingMode;

/// An [`AddressSpace`] implementation for `x86`'s paging disabled mode.
pub struct DisabledPaging;

impl AddressSpace for DisabledPaging {
    fn map(
        &mut self,
        virtual_address: u64,
        physical_address: u64,
        count: u64,
        _protection: ProtectionFlags,
    ) -> Result<(), MapError> {
        let byte_count = count
            .checked_mul(self.page_size())
            .ok_or(MapError::InvalidSize)?;
        let virtual_end_address = virtual_address.checked_add(byte_count);
        if virtual_end_address
            .is_none_or(|virtual_end_address| virtual_end_address > u64::from(u32::MAX))
        {
            return Err(MapError::InvalidAddress);
        }

        assert_eq!(physical_address, virtual_address);

        Ok(())
    }

    unsafe fn unmap(&mut self, virtual_address: u64, count: u64) -> Result<(), NotMapped> {
        debug_assert_eq!(
            virtual_address & self.page_size(),
            0,
            "virtual address not properly aligned"
        );

        let byte_count = count
            .checked_mul(self.page_size())
            .expect("mapping too large");
        let virtual_end_address = virtual_address
            .checked_add(byte_count)
            .expect("virtual region too large");

        assert!(
            virtual_end_address <= u64::from(u32::MAX),
            "impossible mapping"
        );

        Ok(())
    }

    fn translate_virt(&self, virtual_address: u64) -> Result<u64, NoMapping> {
        if virtual_address > u64::from(u32::MAX) {
            return Err(NoMapping);
        }

        Ok(virtual_address)
    }

    fn max_address(&self) -> u64 {
        u64::from(u32::MAX)
    }

    fn page_size(&self) -> u64 {
        4096
    }
}

impl X86CommonAddressSpace for DisabledPaging {
    fn physical_address(&self) -> u64 {
        0
    }

    fn paging_mode(&self) -> PagingMode {
        PagingMode::Disabled
    }
}
