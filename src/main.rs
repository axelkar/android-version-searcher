use std::{
    fs::File,
    io::{Cursor, Read, Seek, SeekFrom},
};

use abootimg_oxide::BufReader;
use clap::Parser;
use flate2::read::GzDecoder;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    /// Path to the boot, recovery or vendor_boot image
    #[arg(value_name = "FILE")]
    boot_img: String,
}

fn get_magisk_bin(
    mut ramdisk_reader: impl Read + Seek,
) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
    loop {
        let cpio_reader = cpio::NewcReader::new(&mut ramdisk_reader).unwrap();
        if cpio_reader.entry().is_trailer() {
            return Ok(None);
        }
        if cpio_reader.entry().name() == "overlay.d/sbin/magisk.xz"
          || cpio_reader.entry().name() == "overlay.d/sbin/magisk32.xz" {
            let mut buf = Vec::new();
            lzma_rs::xz_decompress(&mut BufReader::new(cpio_reader), &mut buf).unwrap();
            return Ok(Some(buf));
        } else {
            cpio_reader.skip().unwrap();
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let mut r = BufReader::new(File::open(cli.boot_img).unwrap());
    let hdr = abootimg_oxide::Header::parse(&mut r).unwrap();

    let os_ver_patch = hdr.osversionpatch();
    println!("OS version: {}", os_ver_patch.version());
    println!("OS patch level: {}", os_ver_patch.patch());

    let cmdline = std::str::from_utf8(hdr.cmdline()).expect("Cmdline should be valid UTF-8");
    println!("Cmdline: {}", cmdline);

    let ramdisk = {
        r.seek(SeekFrom::Start(hdr.ramdisk_position() as u64))
            .unwrap();
        let mut buf = Vec::new();
        GzDecoder::new(r.take(hdr.ramdisk_size() as u64))
            .read_to_end(&mut buf)
            .unwrap();
        buf
    };

    if let Some(magisk32) = get_magisk_bin(&mut Cursor::new(ramdisk))? {
        let needle_idx = memchr::memmem::find_iter(&magisk32, b"Magisk ")
            .find(|i| magisk32[i + b"Magisk ".len()].is_ascii_digit())
            .expect("Should find version pattern in magisk32");

        let version = &magisk32[needle_idx + "Magisk ".len()..];
        let version = &version[..version
            .iter()
            .position(|&b| !matches!(b, b'0'..=b'9' | b'.' | b'(' | b')'))
            .unwrap_or(version.len())];
        let version = std::str::from_utf8(version).unwrap();

        println!("Magisk version: {version}");
    } else {
        println!("Magisk not found");
    }

    // TODO: Android dynamic partitions impl fully in userspace to read the other slot
    // TODO: also look at `/system/build.prop` for ro.build.version.release and ro.build.version.security_patch
    /*let path = std::env::args().nth(1).expect("Please provide a path to a system.img.\nMost likely it's /dev/block/mapper/system{_a,_b}");
    let file = File::open(path).unwrap();
    let fs = Ext4::load(Box::new(file));*/

    Ok(())
}
