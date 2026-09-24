#[cfg(not(target_os = "windows"))]
const _: () = assert!(false, "TTD bindings only work on Windows");

#[cfg(target_arch = "aarch64")]
const _: () = assert!(false, "Windows/ARM64 bindings for TTD are not available yet");

#[cfg(target_arch = "x86_64")]
const ARCH: &str = "x64";

#[cfg(target_arch = "aarch64")]
const ARCH: &str = "arm64";

const TTD_SDK_PACKAGE_NAME: &str = "microsoft.timetraveldebugging.apis";
const TTD_SDK_PACKAGE_VERSION: &str = "0.9.5";
const NUGET_DOWNLOAD_LINK: &str = const_format::formatcp!(
    "https://globalcdn.nuget.org/packages/{TTD_SDK_PACKAGE_NAME}.{TTD_SDK_PACKAGE_VERSION}.nupkg?packageVersion={TTD_SDK_PACKAGE_VERSION}",
);
const WINGET_TTD_PACKAGE_NAME: &str = "Microsoft.TimeTravelDebugging";
const WINGET_TTD_PACKAGE_VERSION: &str = "1.11.611.0";
const WINGET_TTD_PACKAGE_ID: &str = "8wekyb3d8bbwe";
const WINGET_TTD_INSTALL_PATH: &str =
    const_format::formatcp!("C:\\Program Files\\WindowsApps\\{WINGET_TTD_PACKAGE_NAME}_{WINGET_TTD_PACKAGE_VERSION}_{ARCH}__{WINGET_TTD_PACKAGE_ID}");

#[cfg(debug_assertions)]
const BUILD_TYPE: &str = "Debug";
#[cfg(not(debug_assertions))]
const BUILD_TYPE: &str = "Release";

const TTD_FFI_BASE_DIR: &str = "./ttd_ffi";
const TTD_FFI_BUILD_DIR: &str = const_format::formatcp!("{TTD_FFI_BASE_DIR}/build");
const TTD_FFI_INSTALL_DIR: &str = const_format::formatcp!("{TTD_FFI_BASE_DIR}/install");
const TTD_FFI_INSTALL_INCLUDE_DIR: &str = const_format::formatcp!("{TTD_FFI_INSTALL_DIR}/ttd_ffi/Include");
const TTD_FFI_INSTALL_LIBRARY_DIR: &str = const_format::formatcp!("{TTD_FFI_INSTALL_DIR}/ttd_ffi/Library");

const TTD_DLLS: [&str; 7] = [
    "TTDLiveRecorder.dll",
    "TTDLoader.dll",
    "TTDRecord.dll",
    "TTDRecordCPU.dll",
    "TTDRecordUI.dll",
    "TTDReplay.dll",
    "TTDReplayCPU.dll",
];

const NB_CPU: usize = 2;

fn get_nb_cpu() -> String {
    std::env::var("NUMBER_OF_PROCESSORS").unwrap_or(NB_CPU.to_string())
}

fn get_nuget_pkg_path() -> std::path::PathBuf {
    let mut nuget_path = std::path::PathBuf::from(std::env::var("USERPROFILE").unwrap().as_str());
    nuget_path.push(".nuget/packages");
    if !nuget_path.exists() {
        std::fs::create_dir_all(&nuget_path).unwrap();
        assert!(nuget_path.exists(), "{} doesn't exist but should", &nuget_path.to_string_lossy());
    }
    nuget_path
}

fn get_ttd_sdk_path() -> std::path::PathBuf {
    let mut ttd_sdk_path = std::path::PathBuf::from(std::env::var("USERPROFILE").unwrap().as_str());
    ttd_sdk_path.push(format!(".nuget/packages/{}/{}", TTD_SDK_PACKAGE_NAME, TTD_SDK_PACKAGE_VERSION).as_str());
    ttd_sdk_path
}

fn download_nuget_package(package_name: &str, version: &str) -> std::path::PathBuf {
    let download_dir = get_nuget_pkg_path();
    std::fs::create_dir_all(&download_dir).unwrap();

    let package_path = download_dir.join(format!("{}.{}.nupkg", package_name, version));
    let extract_path = download_dir.join(format!("{}/{}", package_name, version));

    let mut response = reqwest::blocking::get(NUGET_DOWNLOAD_LINK).unwrap();
    let mut dest_file = std::fs::File::create(&package_path).unwrap();
    std::io::copy(&mut response, &mut dest_file).unwrap();

    let package_file = std::fs::File::open(&package_path).unwrap();
    let mut archive = zip::ZipArchive::new(package_file).unwrap();

    archive.extract(&extract_path).unwrap();
    extract_path
}

fn cmake_build_ffi() {
    // CMake configure
    {
        assert!(
            std::process::Command::new("cmake")
                .args(["-S", TTD_FFI_BASE_DIR])
                .args(["-B", TTD_FFI_BUILD_DIR])
                .spawn()
                .unwrap()
                .wait()
                .expect("failed to configure cmake")
                .success()
        );
    }

    // CMake build
    {
        assert!(
            std::process::Command::new("cmake")
                .args(["--build", TTD_FFI_BUILD_DIR])
                .args(["--parallel", &get_nb_cpu()])
                .args(["--config", BUILD_TYPE])
                // .args(["--", "-D_LIBTTD_VERBOSE_OUTPUT"]) // Uncomment for verbose output from ttd_ffi
                .spawn()
                .unwrap()
                .wait()
                .expect("failed to build with cmake")
                .success()
        );
    }

    // CMake install
    {
        assert!(
            std::process::Command::new("cmake")
                .args(["--install", TTD_FFI_BUILD_DIR])
                .args(["--config", BUILD_TYPE])
                .args(["--prefix", TTD_FFI_INSTALL_DIR])
                .spawn()
                .unwrap()
                .wait()
                .expect("failed to install with cmake")
                .success()
        );
    }

    {
        for hdr in ["ttd_ffi.cpp", "ttd_ffi.hpp", "constants.hpp.in"] {
            let header_path = format!("{}/{}", TTD_FFI_INSTALL_INCLUDE_DIR, hdr);
            println!("cargo:rerun-if-changed={}", header_path.as_str());
        }
    }
}

fn generate_ttd_bindings() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let ttd_sdk_path = get_ttd_sdk_path();

    if BUILD_TYPE == "Debug" {
        println!("cargo:rustc-link-lib=dylib=ucrtd");
    }

    // include libs
    {
        let lib_dir = format!("{}/{}", manifest_dir, TTD_FFI_INSTALL_LIBRARY_DIR);
        assert!(std::path::Path::new(&lib_dir).exists(), "{lib_dir} should exist but doesn't");
        println!("cargo:rustc-link-search={}", lib_dir);
        println!("cargo:rustc-link-lib=ttd_ffi");

        let mut lib_path = ttd_sdk_path.clone();
        lib_path.push(format!("sdk/lib/{ARCH}"));

        println!("cargo:rustc-link-search={}", lib_path.as_path().to_string_lossy());
        for libname in ["TTDLiveRecorder", "TTDReplay"] {
            println!("cargo:rustc-link-lib={}", libname);
        }
    }

    // Create the binding files
    {
        let src = std::path::PathBuf::from(format!("{}/{}", TTD_FFI_INSTALL_INCLUDE_DIR, "ttd_ffi.hpp"));
        let dst = std::path::PathBuf::from("./src/bindings.rs");
        let bindings = bindgen::Builder::default()
            .generate_comments(true)
            .header(src.as_path().to_string_lossy())
            .clang_args(["-x", "c++", "-std=c++23"])
            .clang_arg(format!("-I{}", TTD_FFI_INSTALL_INCLUDE_DIR))
            .opaque_type("std::.*")
            .use_core()
            .opaque_type("TTD::TBufferView.*")
            .opaque_type("TTD::Replay::UniqueReplayEngine")
            .opaque_type("TTD::Replay::UniqueCursor")
            .blocklist_function("TTD::.*GetEndAddress.*")
            .allowlist_function("TTD_FFI::.*")
            .allowlist_type("TTD_FFI::.*")
            .allowlist_item("TTD_FFI::.*")
            .allowlist_var("TTD_FFI::.*")
            .allowlist_type("TTD::SystemInfo")
            .allowlist_type("TTD::Replay::Position")
            .enable_cxx_namespaces()
            .generate_inline_functions(true)
            .derive_default(true)
            .derive_debug(true)
            .derive_copy(true)
            .derive_hash(true)
            .derive_partialeq(true)
            .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
            .generate()
            .expect("bindgen failed");

        std::fs::write(
            dst.as_path(),
            format!(
                "//! Auto-generated bindings
#![allow(unused)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unsafe_op_in_unsafe_fn)]
#![allow(clippy::missing_safety_doc)]

{}",
                bindings
            ),
        )
        .unwrap();
    }
}

fn install_winget_ttd() {
    std::process::Command::new("winget")
        .args([
            "install",
            "--source",
            "winget",
            "--ignore-warnings",
            "--accept-package-agreements",
            "--silent",
            "--no-upgrade",
        ])
        .args(["--exact", "--id", WINGET_TTD_PACKAGE_NAME])
        .args(["--version", WINGET_TTD_PACKAGE_VERSION])
        .spawn()
        .unwrap()
        .wait()
        .expect("failed to install ttd");

    assert!(
        std::fs::exists(WINGET_TTD_INSTALL_PATH).unwrap(),
        "{} doesn't exist but should",
        WINGET_TTD_INSTALL_PATH
    );
}

fn copy_ttd_dlls() {
    let ttd_dll_glob = format!("{WINGET_TTD_INSTALL_PATH}/*.dll");
    for entry in glob::glob(&ttd_dll_glob).unwrap() {
        let Ok(entry) = entry else {
            continue;
        };
        if !entry.is_file() {
            continue;
        }

        let tgt = format!("../{}", entry.file_name().unwrap().to_string_lossy());
        std::fs::copy(&entry, tgt).unwrap();
    }
}

fn generate_constant_file() {
    std::fs::write(
        "./src/constants.rs",
        format!(
            "//! Auto-generated constants
#![allow(unused)]

/// The `winget` package name for TTD
const TTD_PACKAGE_NAME: &str = \"{}\";

/// The `winget` package version for TTD
const TTD_PACKAGE_VERSION: &str = \"{}\";

",
            TTD_SDK_PACKAGE_NAME, TTD_SDK_PACKAGE_VERSION
        ),
    )
    .unwrap();
}

fn main() {
    if std::env::var("DOCS_RS").is_ok() {
        return;
    }

    if !std::fs::exists(get_ttd_sdk_path()).unwrap() {
        download_nuget_package(TTD_SDK_PACKAGE_NAME, TTD_SDK_PACKAGE_VERSION);
    }

    for dll in TTD_DLLS {
        let path = std::path::PathBuf::from(format!("../{dll}"));
        if !std::fs::exists(&path).unwrap() {
            install_winget_ttd();
            copy_ttd_dlls();
            assert!(std::fs::exists(&path).unwrap(), "{} doesn't exist but should", &path.to_string_lossy());
        }
    }

    cmake_build_ffi();

    generate_ttd_bindings();

    generate_constant_file();
}
