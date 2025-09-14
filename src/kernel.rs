//! Extract kernel information from a kernel image in Android's boot image
//!
//! <https://docs.kernel.org/arch/arm64/booting.html>
//!
//! See also <https://android.googlesource.com/platform/build/+/refs/heads/master/tools/extract_kernel.py>.

pub mod arm64_image_header {
    use bilge::prelude::*;

    #[binrw::binread]
    #[derive(Debug)]
    #[brw(little)]
    pub struct Arm64ImageHeader {
        /// Executable code responsible for branching to "stext"
        /// (see [Linux kernel docs](https://docs.kernel.org/arch/arm64/booting.html)).
        pub code: [u8; 8],
        /// Image load offset
        pub text_offset: u64,
        /// Effective Image size
        pub image_size: u64,
        /// Kernel flags
        pub flags: Flags,
        /// Magic number, "ARM\x64"
        #[brw(pad_before = 3 * 8)] // reserved fields
        #[br(temp, assert(magic == u32::from_le_bytes(*b"ARM\x64")))]
        #[bw(calc = u32::from_le_bytes(*b"ARM\x64"))]
        magic: u32,
        /// reserved (used for PE COFF offset)
        pub res5: [u8; 4],
    }

    #[bitsize(64)]
    #[derive(Clone, Copy, FromBits, DebugBits, binrw::BinRead, binrw::BinWrite)]
    #[br(map = |x: u64| Self::from(x))]
    #[bw(map = |&x| u64::from(x))]
    pub struct Flags {
        pub endianness: Endianness,
        pub page_size: PageSize,
        pub physical_placement: PhysicalPlacement,
        pub reserved: u60,
    }

    #[bitsize(1)]
    #[derive(FromBits, Debug)]
    pub enum Endianness {
        Little,
        Big,
    }

    #[bitsize(2)]
    #[derive(FromBits, Debug)]
    pub enum PageSize {
        Unspecified,
        Size4K,
        Size16K,
        Size64K,
    }

    #[bitsize(1)]
    #[derive(FromBits, Debug)]
    pub enum PhysicalPlacement {
        /// "2MB aligned base should be as close as possible to the base of DRAM, since memory below it is not accessible via the linear mapping"
        A,
        /// "2MB aligned base such that all image_size bytes counted from the start of the image are within the 48-bit addressable range of physical memory"
        B,
    }
}

pub mod kernel_banner {
    use std::sync::LazyLock;

    use regex::bytes::Regex;

    static PREFIX: &[u8] = b"Linux version ";

    pub fn find_kernel_banner(haystack: &[u8]) -> Option<ParsedKernelBanner> {
        memchr::memmem::find_iter(haystack, PREFIX)
            .find_map(|i| parse_kernel_banner(&haystack[i..]))
    }

    /// # Capture groups
    ///
    /// - `release`: Kernel version with any suffixes. For example: "w.x.y-flavor"
    /// - `version`: Kernel version. For example: "w.x.y"
    /// - `builder`: Builder machine's info. For example: "username@hostname"
    /// - `compiler`: Compiler information. For example: "gcc ..., GNU ld ..."
    /// - `extra`: Anything extra. For example: "#1 SMP PREEMPT Thu Jan 1 00:00:01 UTC 1970"
    static RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"Linux version (?<release>(?<version>[0-9]+\.[0-9]+\.[0-9]+).*) \((?<builder>.*@.*)\) \((?<compiler>.*)\) (?<extra>.*)\n").unwrap()
    });

    pub fn parse_kernel_banner(haystack: &[u8]) -> Option<ParsedKernelBanner> {
        let caps = RE.captures(haystack)?;

        Some(ParsedKernelBanner {
            banner: caps.get(0).unwrap().as_bytes(),
            release: caps.name("release").unwrap().as_bytes(),
            version: caps.name("version").unwrap().as_bytes(),
            builder: caps.name("builder").unwrap().as_bytes(),
            compiler: caps.name("compiler").unwrap().as_bytes(),
            extra: caps.name("extra").unwrap().as_bytes(),
        })
    }

    pub struct ParsedKernelBanner<'banner> {
        pub banner: &'banner [u8],
        pub release: &'banner [u8],
        pub version: &'banner [u8],
        pub builder: &'banner [u8],
        pub compiler: &'banner [u8],
        pub extra: &'banner [u8],
    }
}

/// Extract the .config from a kernel image
///
/// Requires the kernel to be compiled with `CONFIG_IKCONFIG`.
mod ikconfig {
    // TODO: ikconfig
    // IKCFG_ST
    // IKCFG_ED
}
