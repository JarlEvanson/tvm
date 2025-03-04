//! Implementation of the cross address space switching and function calls.

use crate::paging::X86CommonAddressSpace;

/// Sets up and executes the switching code, thereby executing the appplication starting at
/// `entry_point`.
///
/// # Errors
///
/// TODO:
pub fn switch(
    switch_space: &mut dyn X86CommonAddressSpace,
    application_space: &mut dyn X86CommonAddressSpace,
    entry_point: u64,
) -> Result<bool, SwitchError> {
    todo!()
}

/// Various errors that can occur while executing [`switch()`].
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum SwitchError {}
