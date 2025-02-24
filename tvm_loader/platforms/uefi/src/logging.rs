//! Logging interface implementation for UEFI.

use core::fmt::{self, Write};

use tvm_loader::{
    logging::{Log, Message},
    unsafe_global_logger,
};

use crate::system_table_ptr;

unsafe_global_logger!(Logger);

/// Implementation of [`Log`] using the UEFI System Table console out.
struct Logger;

impl Log for Logger {
    fn log(&self, message: &Message) {
        let _ = writeln!(
            ConOut,
            "[{}][{}]: {}",
            message.level(),
            message.target(),
            message.args()
        );
    }
}

/// Implementation of [`Write`] for the UEFI System Table console out.
struct ConOut;

impl Write for ConOut {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        const BUFFER_SIZE: usize = 128;

        let Some(system_table_ptr) = system_table_ptr() else {
            return Err(fmt::Error);
        };

        // SAFETY:
        //
        // According to the invariants of [`system_table_ptr()`], since `system_table_ptr` is
        // [`Some`], it is safe to dereference.
        let con_out = unsafe { (*system_table_ptr.as_ptr()).con_out };
        if con_out.is_null() {
            return Err(fmt::Error);
        }

        // SAFETY:
        //
        // According to the invariants of [`system_table_ptr()`], since `system_table_ptr` is
        // [`Some`], it is safe to dereference, and since `con_out` is non-NULL, it is safe to
        // dereference.
        let output_string_func = unsafe { (*con_out).output_string };

        let mut buffer = [0u16; BUFFER_SIZE + 1];
        let mut index = 0;

        let mut chars = s.chars();
        let mut next_char = chars.next();

        let mut newline_processed = false;
        while let Some(mut c) = next_char.take() {
            if c == '\n' && !newline_processed {
                newline_processed = true;

                next_char = Some(c);
                c = '\r';
            } else {
                newline_processed = false;
            }

            if c.len_utf8() == 4 {
                // Print replacement character
                c = '\u{FFFD}';
            }

            buffer[index] = c as u16;
            index += 1;

            if index == BUFFER_SIZE {
                let string = &mut buffer[..=index];
                string[index] = 0;

                // Ignore warning/error, can't fix it anyway.
                //
                // SAFETY:
                //
                // `output_string_func` was obtained from a valid UEFI SimpleTextOutputProtocol
                // pointer, which means it is safe to called.
                let _ = unsafe { output_string_func(con_out, string.as_mut_ptr()) };

                index = 0;
            }

            if next_char.is_none() {
                next_char = chars.next();
            }
        }

        if index != 0 {
            let string = &mut buffer[..=index];
            string[index] = 0;

            // Ignore warning/error, can't fix it anyway.
            //
            // SAFETY:
            //
            // `output_string_func` was obtained from a valid UEFI SimpleTextOutputProtocol
            // pointer, which means it is safe to called.
            let _ = unsafe { output_string_func(con_out, string.as_mut_ptr()) };
        }

        Ok(())
    }
}
