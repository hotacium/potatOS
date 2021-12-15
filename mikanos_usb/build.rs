use flate2::read::GzDecoder;
use std::{env, fs, path::PathBuf};
use std::os::unix;
use std::time::SystemTime;
use tar::Archive as TarArchive;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

async fn build_lib() -> Result<()> {
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);

    let unpacked_dir = out_dir.join("x86_64-elf");

    if unpacked_dir.exists() {
        fs::remove_dir_all(&unpacked_dir)?;
    }

    let now = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => format!("+{}", n.as_secs()),
        Err(_) => String::from("SystemTime before UNIX EPOCH"),
    };
    let resp = reqwest::get(
        "https://github.com/uchan-nos/mikanos-build/releases/download/v2.0/x86_64-elf.tar.gz",
    )
    .await.expect(&format!("[{}]: Failed to fetch response :(", now))
    .bytes()
    .await?;

    let tar = GzDecoder::new(&*resp);
    let mut archive = TarArchive::new(tar);
    archive.unpack(&out_dir)?;

    env::set_var("CC", "clang");
    env::set_var("CXX", "clang++");

    let files = glob::glob("./usb_driver/**/*.cpp")?.collect::<std::result::Result<Vec<_>, _>>()?;
    println!("{:?}", files);

    cc::Build::new()
        .cpp(true)
        .include(unpacked_dir.join("include"))
        .include(unpacked_dir.join("include/c++/v1"))
        .include("./usb_driver/")
        .files(files)
        .define("__ELF__", None)
        .define("_LDBL_EQ_DBL", None)
        .define("_GNU_SOURCE", None)
        .define("_POSIX_TIMERS", None)
        .flag("-nostdlibinc")
        .flag("-ffreestanding")
        .flag("-mno-red-zone")
        .flag("-fno-exceptions")
        .flag("-fno-rtti")
        .flag("-std=c++17")
        .flag("-ggdb")
        .extra_warnings(false)
        .cpp_link_stdlib(None)
        .target("x86_64-elf")
        .compile("mikanos_usb");

    for lib in &["c", "c++", "c++abi"] {
        let filename = format!("lib{}.a", lib);
        let dest = out_dir.join(&filename);
        let src = unpacked_dir.join(format!("lib/{}", filename));
        if dest.exists() {
            fs::remove_file(&dest)?;
        }
        unix::fs::symlink(&src, &dest)?;
        println!("cargo:rustc-link-lib=static={}", lib);
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(e) = build_lib().await {
        println!("Build Error: {}", e);
    } else {
        println!("Build Success!");
    }
}
