//! Definitions for the ELF file identifier.

use core::{error, fmt, mem};

/// Contains basic information about an ELF file that can be obtained in an architecture
/// independent manner.
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct ElfIdent<'slice> {
    /// The underlying bytes of the ELF file identifier.
    pub(crate) bytes: &'slice [u8; 16],
}

impl<'slice> ElfIdent<'slice> {
    /// Creates a new [`ElfIdent`] from the given `slice`, returning `None` if `slice` too
    /// small to contain an [`ElfIdent`].
    pub const fn new(slice: &'slice [u8]) -> Option<Self> {
        match slice.first_chunk() {
            Some(bytes) => Some(Self { bytes }),
            None => None,
        }
    }

    /// Validates that this [`ElfIdent`] matches the ELF specification and is supported by this
    /// crate.
    ///
    /// # Errors
    /// - [`ValidateElfIdentSpecError::InvalidMagicBytes`]: Returned when this [`ElfIdent`]'s magic
    ///     bytes are invalid.
    /// - [`ValidateElfIdentSpecError::UnsupportedElfHeaderVersion`]: Returned when this ELF header
    ///     version is not supported.
    /// - [`ValidateElfIdentSpecError::NonZeroPadding`]: Returned when the padding of this
    ///     [`ElfIdent`] is non-zero.
    pub fn validate_spec(&self) -> Result<(), ValidateElfIdentSpecError> {
        if self.magic() != Self::MAGIC_BYTES {
            return Err(ValidateElfIdentSpecError::InvalidMagicBytes(self.magic()));
        }

        if self.header_version() != Self::CURRENT_HEADER_VERSION {
            return Err(ValidateElfIdentSpecError::UnsupportedElfHeaderVersion(
                self.header_version(),
            ));
        }

        if self.padding().into_iter().any(|val| val != 0) {
            return Err(ValidateElfIdentSpecError::NonZeroPadding(self.padding()));
        }

        Ok(())
    }

    /// The magic bytes that identify the start of an ELf file.
    pub const MAGIC_BYTES: [u8; 4] = [0x7F, b'E', b'L', b'F'];

    /// The current version of the ELF file header.
    pub const CURRENT_HEADER_VERSION: u8 = 1;

    /// Returns the magic bytes that identify this file as an ELF file.
    pub const fn magic(&self) -> [u8; 4] {
        match self.bytes.first_chunk() {
            Some(bytes) => *bytes,
            None => unreachable!(),
        }
    }

    /// Returns the [`Class`] of this ELF file.
    pub const fn class(&self) -> Class {
        Class(self.bytes[mem::offset_of!(DefElfIdent, class)])
    }

    /// Returns the [`Encoding`] of this ELF file.
    pub const fn encoding(&self) -> Encoding {
        Encoding(self.bytes[mem::offset_of!(DefElfIdent, data)])
    }

    /// Returns the version of the ELF file identifier.
    pub const fn header_version(&self) -> u8 {
        self.bytes[mem::offset_of!(DefElfIdent, header_version)]
    }

    /// Returns the [`OsAbi`] of the ELF file.
    pub const fn os_abi(&self) -> OsAbi {
        OsAbi(self.bytes[mem::offset_of!(DefElfIdent, os_abi)])
    }

    /// Returns the version of the ABI to which the object is targeted.
    pub const fn abi_version(&self) -> u8 {
        self.bytes[mem::offset_of!(DefElfIdent, abi_version)]
    }

    /// Returns the padding bytes of this [`ElfIdent`].
    pub const fn padding(&self) -> [u8; 7] {
        match self.bytes.last_chunk() {
            Some(bytes) => *bytes,
            None => unreachable!(),
        }
    }
}

impl core::fmt::Debug for ElfIdent<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut debug_struct = f.debug_struct("ElfIdent");

        debug_struct.field("magic", &self.magic());
        debug_struct.field("class", &self.class());
        debug_struct.field("encoding", &self.encoding());
        debug_struct.field("header_version", &self.header_version());
        debug_struct.field("os_abi", &self.os_abi());
        debug_struct.field("abi_version", &self.abi_version());

        debug_struct.finish()
    }
}

/// Various errors that can occur when validating an [`ElfIdent`] follows the ELF specification and
/// is supported by this crate.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValidateElfIdentSpecError {
    /// The given slice has invalid magic bytes.
    InvalidMagicBytes([u8; 4]),
    /// The ELF header verison is not unsupported.
    UnsupportedElfHeaderVersion(u8),
    /// The padding of the [`ElfIdent`] is non-zero.
    NonZeroPadding([u8; 7]),
}

impl fmt::Display for ValidateElfIdentSpecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMagicBytes(bytes) => write!(f, "invalid magic bytes: {bytes:X?}",),
            Self::UnsupportedElfHeaderVersion(version) => {
                write!(f, "invalid ELF header version: {version}")
            }
            Self::NonZeroPadding(padding) => write!(f, "non-zero padding: {padding:X?}"),
        }
    }
}

impl error::Error for ValidateElfIdentSpecError {}

/// Specifier of the ELF file class, which determines the sizing
/// of various items in the ELF file format.
#[repr(transparent)]
#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Class(pub u8);

impl Class {
    /// Invalid [`Class`] specifier.
    pub const NONE: Self = Self(0);
    /// ELF file is formatted in its 32-bit format.
    pub const CLASS32: Self = Self(1);
    /// ELF file is formatted in its 64-bit format.
    pub const CLASS64: Self = Self(2);
}

impl fmt::Debug for Class {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::NONE => f.pad("Invalid"),
            Self::CLASS32 => f.pad("Class32"),
            Self::CLASS64 => f.pad("Class64"),
            class => f.debug_tuple("Class").field(&class.0).finish(),
        }
    }
}

/// Specifier of the ELF file data encoding, which determines the encoding
/// of both the data structures used by the ELF file format and data contained
/// in the object file sections.
#[repr(transparent)]
#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Encoding(pub u8);

impl Encoding {
    /// Invalid [`Encoding`] specifier.
    pub const NONE: Self = Self(0);
    /// The encoding of the ELF file format uses little endian
    /// two's complement integers.
    pub const LSB2: Self = Self(1);
    /// The encoding of the ELF file format uses big endian
    /// two's complement integers.
    pub const MSB2: Self = Self(2);
}

impl fmt::Debug for Encoding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::NONE => f.pad("NoEncoding"),
            Self::LSB2 => f.pad("LittleEndian"),
            Self::MSB2 => f.pad("BigEndian"),
            encoding => f.debug_tuple("Encoding").field(&encoding.0).finish(),
        }
    }
}

/// Specifier of the OS or ABI specific ELF extensions used by this file.
///
/// This field determines the interpretation of various OS or ABI specific values.
#[repr(transparent)]
#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct OsAbi(pub u8);

impl OsAbi {
    /// No extensions or unspecified extensions.
    pub const NONE: Self = Self(0);
    /// Hewlett-Packard HP_UX
    pub const HP_UX: Self = Self(1);
    /// NetBSD
    pub const NETBSD: Self = Self(2);
    /// Gnu (Historically also Linux)
    pub const GNU: Self = Self(3);
    /// Sun Solaris
    pub const SUN_SOLARIS: Self = Self(6);
    /// AIX
    pub const AIX: Self = Self(7);
    /// IRIX
    pub const IRIX: Self = Self(8);
    /// FreeBSD
    pub const FREE_BSD: Self = Self(9);
    /// Compaq TRU64 UNIX
    pub const COMPAQ_TRU64_UNIX: Self = Self(10);
    /// Novell Modesto
    pub const NOVELL_MODESTO: Self = Self(11);
    /// Open BSD
    pub const OPEN_BSD: Self = Self(12);
    /// Open VMS
    pub const OPEN_VMS: Self = Self(13);
    /// Hewlett-Packard Non-Stop Kernel
    pub const HP_NSK: Self = Self(14);
    /// Amiga Research OS
    pub const AMIGA_RESEARCH: Self = Self(15);
    /// The FenixOS highly scalable multi-core OS
    pub const FENIXOS: Self = Self(16);
    /// Nuxi CloudABI
    pub const CLOUD_ABI: Self = Self(17);
    /// Stratus Technologies OpenVOS
    pub const OPENVOS: Self = Self(18);

    /// Start of the architecture specific value range.
    pub const ARCHITECTURE_SPECIFIC_START: Self = Self(64);
    /// Inclusive end of the architecture specific value range.
    pub const ARCHITECTURE_SPECIFIC_END: Self = Self(255);
}

impl fmt::Debug for OsAbi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::NONE => f.pad("None"),
            Self::HP_UX => f.pad("Hewlett-Packard HP-UX"),
            Self::NETBSD => f.pad("NetBSD"),
            Self::GNU => f.pad("GNU"),
            Self::SUN_SOLARIS => f.pad("Sun Solaris"),
            Self::AIX => f.pad("AIX"),
            Self::IRIX => f.pad("IRIX"),
            Self::FREE_BSD => f.pad("IRIX"),
            Self::COMPAQ_TRU64_UNIX => f.pad("Compaq TRU64 UNIX"),
            Self::NOVELL_MODESTO => f.pad("Novell Modesto"),
            Self::OPEN_BSD => f.pad("Open BSD"),
            Self::OPEN_VMS => f.pad("Open VMS"),
            Self::HP_NSK => f.pad("Hewlett-Packard Non-Stop Kernel"),
            Self::AMIGA_RESEARCH => f.pad("Amiga Research OS"),
            Self::FENIXOS => f.pad("FenixOS"),
            Self::CLOUD_ABI => f.pad("Nuxi CloudABI"),
            Self::OPENVOS => f.pad("Stratus Technologies OpenVOS"),
            os_abi => f.debug_tuple("OsAbi").field(&os_abi.0).finish(),
        }
    }
}

/// Block of machine-independent data to mark the file as an ELF file
/// and provide enough information for the remainder of the ELF file to be
/// decoded.
#[repr(C)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct DefElfIdent {
    /// Holds magic numbers to identify the file as an ELF file.
    pub magic: [u8; 4],
    /// The file's class (native word size).
    pub class: Class,
    /// Encoding of data structures used by the object file container
    /// and data contained in object file sections.
    pub data: Encoding,
    /// The ELF header version number..
    pub header_version: u8,
    /// Identifies the OS or ABI specific extensions used by this file.
    pub os_abi: OsAbi,
    /// The version of the ABI to which the object file is targeted.
    ///
    /// This should be zero if the [`DefElfIdent::os_abi`] field has no
    /// definitions or no version values in the processor supplement.
    pub abi_version: u8,
    /// Unused bytes, should all be zero.
    pub _padding: [u8; 7],
}
