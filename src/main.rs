use std::{
    fs::File,
    io::{Cursor, Read, Seek, SeekFrom},
    sync::LazyLock,
};

use abootimg_oxide::BufReader;
use binrw::BinRead;
use clap::Parser;
use color_eyre::{
    eyre::{Context, OptionExt},
    Result,
};
use flate2::read::GzDecoder;
use owo_colors::{colors::xterm::Gray, OwoColorize};
use regex::bytes::Regex;

use crate::kernel::{arm64_image_header::Arm64ImageHeader, kernel_banner::find_kernel_banner};

pub mod kernel;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    /// Path to the boot, recovery or vendor_boot image
    #[arg(value_name = "FILE")]
    boot_img: String,
}

fn get_magisk_bin(mut ramdisk_reader: impl Read + Seek) -> Result<Option<Vec<u8>>> {
    loop {
        let cpio_reader =
            cpio::NewcReader::new(&mut ramdisk_reader).wrap_err("Failed to read CPIO archive")?;
        if cpio_reader.entry().is_trailer() {
            return Ok(None);
        }
        if cpio_reader.entry().name() == "overlay.d/sbin/magisk.xz"
            || cpio_reader.entry().name() == "overlay.d/sbin/magisk32.xz"
        {
            let mut buf = Vec::new();
            lzma_rs::xz_decompress(&mut BufReader::new(cpio_reader), &mut buf)
                .wrap_err("Failed to decompress XZ")?;
            return Ok(Some(buf));
        } else {
            cpio_reader.skip()?;
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    color_eyre::install()?;

    let mut r = BufReader::new(File::open(cli.boot_img).wrap_err("Failed to open boot image")?);
    let hdr =
        abootimg_oxide::Header::parse(&mut r).wrap_err("Failed to parse boot image header")?;

    {
        println!("{} {}", "###".fg::<Gray>(), "Boot image header info:".cyan());
        let os_ver_patch = hdr.osversionpatch();
        println!("{} {}", "OS version:".blue(), os_ver_patch.version());
        println!("{} {}", "OS patch level:".blue(), os_ver_patch.patch());

        let cmdline =
            std::str::from_utf8(hdr.cmdline()).wrap_err("Cmdline should be valid UTF-8")?;
        println!("{} {cmdline}", "Cmdline:".blue());
        println!("{} {}", "Kernel size:".blue(), hdr.kernel_size());
        println!();
    }

    {
        println!("{} {}", "###".fg::<Gray>(), "Kernel image info:".cyan());

        r.seek(SeekFrom::Start(hdr.kernel_position() as u64))?;

        let kernel = {
            let mut buf = vec![0; hdr.kernel_size() as usize];
            r.read_exact(&mut buf)?;
            buf
        };

        let header = Arm64ImageHeader::read(&mut Cursor::new(&kernel))
            .wrap_err("Failed to parse ARM64 Linux kernel image header")?;

        println!("{} {}", "Text offset:".blue(), header.text_offset);
        println!("{} {}", "Effective Image size:".blue(), header.image_size);
        println!("{} {:#?}", "Flags:".blue(), header.flags);

        let banner = find_kernel_banner(&kernel).ok_or_eyre("Couldn't find kernel banner")?;
        let banner = std::str::from_utf8(banner.banner)?.trim_end();
        println!("{} {}", "Banner:".blue(), banner);
        println!();
    }

    let ramdisk = {
        r.seek(SeekFrom::Start(hdr.ramdisk_position() as u64))?;
        let mut buf = Vec::new();
        GzDecoder::new(r.take(hdr.ramdisk_size() as u64))
            .read_to_end(&mut buf)
            .wrap_err("Failed to decompress GZIP")?;
        buf
    };

    if let Some(magisk) = get_magisk_bin(&mut Cursor::new(ramdisk))? {
        static MAGISK_VERSION_RE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"[0-9]+\.[0-9]+\([0-9]{5}\)").unwrap());

        let version = MAGISK_VERSION_RE
            .find(&magisk)
            .ok_or_eyre("Couldn't find version pattern in magisk binary")?
            .as_bytes();

        let version = std::str::from_utf8(version)?;

        println!("{} {version}", "Magisk version:".blue());
    } else {
        println!("{}", "Magisk not found".yellow());
    }

    // TODO: Android dynamic partitions impl fully in userspace to read the other slot
    // ^ my old android_metadata_rs project
    // TODO: also look at `/system/build.prop` for ro.build.version.{release,security_patch,sdk}

    // sys-mount crate
    /*let path = std::env::args().nth(1).expect("Please provide a path to a system.img.\nMost likely it's either of /dev/block/mapper/system{_a,_b}");
    let file = File::open(path).unwrap();
    let fs = Ext4::load(Box::new(file));*/

    Ok(())
}
