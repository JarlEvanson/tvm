//! Table acquisition implementation for UEFI.

use core::slice;

use uefi::table::configuration::{ACPI, ACPI_2, DEVICE_TREE, SMBIOS, SMBIOS_3};

use crate::system_table_ptr;

/// Locates and returns the physical address of various tables.
///
/// For ACPI, it attempts to acquire the address of XSDP and falls back to returning the address of
/// RSDP if it exists.
pub fn acquire_table_ptrs() -> TableLocations {
    let mut table_locations = TableLocations {
        acpi_root_table: 0,
        device_tree: 0,
        smbios_entry_32: 0,
        smbios_entry_64: 0,
        uefi_system_table: 0,
    };

    let Some(system_table_ptr) = system_table_ptr() else {
        return table_locations;
    };

    // SAFETY:
    //
    // `system_table_ptr` is not NULL and so according to the UEFI specification, the configuration
    // tables should be present.
    let config_table_count = unsafe { (*system_table_ptr.as_ptr()).number_of_table_entries };
    // SAFETY:
    //
    // `system_table_ptr` is not NULL and so according to the UEFI specification, the configuration
    // tables should be present.
    let config_tables_ptr = unsafe { (*system_table_ptr.as_ptr()).configuration_table };

    // SAFETY:
    //
    // `system_table_ptr` is not NULL and so according to the UEFI specification, the configuration
    // tables should be present.
    let config_tables = unsafe { slice::from_raw_parts(config_tables_ptr, config_table_count) };

    for table in config_tables {
        if table.vendor_guid == ACPI {
            if table_locations.acpi_root_table == 0 {
                // If XSDP has not been found, then save RSDP.
                table_locations.acpi_root_table = table.vendor_table as u64;
            }
        } else if table.vendor_guid == ACPI_2 {
            table_locations.acpi_root_table = table.vendor_table as u64;
        } else if table.vendor_guid == DEVICE_TREE {
            table_locations.device_tree = table.vendor_table as u64;
        } else if table.vendor_guid == SMBIOS {
            table_locations.smbios_entry_32 = table.vendor_table as u64;
        } else if table.vendor_guid == SMBIOS_3 {
            table_locations.smbios_entry_64 = table.vendor_table as u64;
        }
    }

    table_locations.uefi_system_table = system_table_ptr.as_ptr() as u64;

    table_locations
}

/// Physical addresses of various important tables.
///
/// All fields may be zero to indicate that the table was not found.
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct TableLocations {
    /// Physical address of the XSDP table, or RSDP if XSDP was not found.
    pub acpi_root_table: u64,
    /// Physical address of the device tree table.
    pub device_tree: u64,
    /// Physical address of the SMBIOS 32-bit entry point structure.
    pub smbios_entry_32: u64,
    /// Physical address of the SMBIOS 64-bit entry point structure.
    pub smbios_entry_64: u64,
    /// Physical address of the UEFI system table.
    pub uefi_system_table: u64,
}
