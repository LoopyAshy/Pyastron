use std::{env, ffi::OsStr, path::PathBuf};

const LIBS_PATH: &str = "libs/Astron/";

macro_rules! astron_path {
    ($path:literal) => {
        format!("{}{}", LIBS_PATH, $path)
    };
}

fn main() {
    compile_dclass();
    compile_astron();
}

fn compile_astron() {
    let mut builder = cc::Build::new();

    builder
        .cpp(true)
        .std("c++14")
        .files(&[
            astron_path!("src/core/global.cpp"),
            astron_path!("src/core/main.cpp"),
            astron_path!("src/core/Logger.cpp"),
            astron_path!("src/core/Role.cpp"),
            astron_path!("src/core/RoleFactory.cpp"),
            astron_path!("src/core/shutdown.cpp"),
        ])
        .files(&[
            astron_path!("src/config/ConfigGroup.cpp"),
            astron_path!("src/config/ConfigVariable.cpp"),
            astron_path!("src/config/constraints.cpp"),
        ])
        .files(&[
            astron_path!("src/messagedirector/ChannelMap.cpp"),
            astron_path!("src/messagedirector/MessageDirector.cpp"),
            astron_path!("src/messagedirector/MDNetworkParticipant.cpp"),
            astron_path!("src/messagedirector/MDNetworkUpstream.cpp"),
        ])
        .files(&[
            astron_path!("src/util/EventSender.cpp"),
            astron_path!("src/util/Timeout.cpp"),
            astron_path!("src/util/TaskQueue.cpp"),
        ])
        .files(&[
            astron_path!("src/net/address_utils.cpp"),
            astron_path!("src/net/HAProxyHandler.cpp"),
            astron_path!("src/net/NetworkAcceptor.cpp"),
            astron_path!("src/net/NetworkClient.cpp"),
            astron_path!("src/net/NetworkConnector.cpp"),
            astron_path!("src/net/TcpAcceptor.cpp"),
        ])
        // Database + YAML
        .files(&[
            astron_path!("src/database/DatabaseServer.cpp"),
            astron_path!("src/database/DatabaseBackend.cpp"),
            astron_path!("src/database/DBOperation.cpp"),
            astron_path!("src/database/DBOperationQueue.cpp"),
            astron_path!("src/database/OldDatabaseBackend.cpp"),
            astron_path!("src/database/DBBackendFactory.cpp"),
            astron_path!("src/database/YAMLDatabase.cpp"),
        ])
        .define("BUILD_DBSERVER", None)
        .define("_NOEXCEPT", "noexcept")
        .define("BUILD_DB_YAML", None)
        // State Server
        .files(&[
            astron_path!("src/stateserver/StateServer.cpp"),
            astron_path!("src/stateserver/DistributedObject.cpp"),
        ])
        .define("BUILD_STATESERVER", None)
        // State Server + Database
        .files(&[
            astron_path!("src/stateserver/DBStateServer.cpp"),
            astron_path!("src/stateserver/LoadingObject.cpp"),
        ])
        .define("BUILD_STATESERVER_DBSS", None)
        // Event Logger
        .file(astron_path!("src/eventlogger/EventLogger.cpp"))
        .define("BUILD_EVENTLOGGER", None)
        // Client Agent
        .files(&[
            astron_path!("src/clientagent/Client.cpp"),
            astron_path!("src/clientagent/ClientAgent.cpp"),
            astron_path!("src/clientagent/ClientFactory.cpp"),
            astron_path!("src/clientagent/AstronClient.cpp"),
        ])
        .define("BUILD_CLIENTAGENT", None)
        // GIT SHA1 - should actually handle this in future lol
        .define("GIT_SHA1", "c1436f90-dirty")
        .include("libs/Astron/src")
        .define("STATIC_LIB", None)
        .emit_rerun_if_env_changed(true)
        .link_lib_modifier("+whole-archive")
        .opt_level(3);

    #[cfg(target_os = "linux")]
    builder.static_flag(true);

    #[cfg(target_os = "windows")]
    {
        configure_windows_dependencies(&mut builder);
        builder
            .define("WIN32", None)
            .define("_WINDOWS", None)
            .define("NDEBUG", None)
            .define("_WIN32_WINDOWS", None)
            .define("WIN32_LEAN_AND_MEAN", None)
            .define("_WIN32_WINNT", "0x0600")
            .define("NOMINMAX", None)
            .define("_WINSOCK_DEPRECATED_NO_WARNINGS", None)
            .define("_CRT_SECURE_NO_WARNINGS", None);
    }

    builder.compile("astrond");

    println!("cargo:rustc-link-lib=dclass");

    #[cfg(target_os = "linux")]
    {
        println!("cargo:rustc-link-lib=static:+whole-archive=uv");
        println!("cargo:rustc-link-lib=static:+whole-archive=yaml-cpp");
        println!("cargo:rustc-link-lib=pthread");
        println!("cargo:rustc-link-lib=rt");
    }
}

#[cfg(target_os = "windows")]
fn configure_windows_dependencies(builder: &mut cc::Build) {
    let vcpkg_root = env::var_os("VCPKG_ROOT")
        .map(PathBuf::from)
        .expect("VCPKG_ROOT must point to a vcpkg checkout");
    let installed_root = env::var_os("VCPKG_INSTALLED_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| vcpkg_root.join("installed"));
    let triplet =
        env::var("VCPKGRS_TRIPLET").unwrap_or_else(|_| "x64-windows-static-md".to_owned());
    let prefix = installed_root.join(triplet);
    let include_path = prefix.join("include");
    let library_path = prefix.join("lib");

    if !include_path.is_dir() || !library_path.is_dir() {
        panic!(
            "vcpkg dependencies are missing for {} (expected {})",
            prefix.display(),
            library_path.display()
        );
    }

    builder.include(include_path);
    println!("cargo:rustc-link-search=native={}", library_path.display());
    println!("cargo:rustc-link-lib=static=uv_a");
    println!("cargo:rustc-link-lib=static=yaml-cpp");

    // Transitive Windows libraries required by a static libuv build.
    for library in [
        "advapi32", "iphlpapi", "psapi", "shell32", "user32", "userenv", "ws2_32",
    ] {
        println!("cargo:rustc-link-lib={}", library);
    }

    println!("cargo:rerun-if-env-changed=VCPKG_ROOT");
    println!("cargo:rerun-if-env-changed=VCPKG_INSTALLED_ROOT");
    println!("cargo:rerun-if-env-changed=VCPKGRS_TRIPLET");
}

fn compile_dclass() {
    let mut compiler = cc::Build::new();

    compiler
        .cpp(true)
        .std("c++14")
        .files(CppFilesFromPath::new(&astron_path!("src/dclass/dc")))
        .files(CppFilesFromPath::new(&astron_path!("src/dclass/file")))
        .files(CppFilesFromPath::new(&astron_path!("src/dclass/util")))
        .files(CppFilesFromPath::new(&astron_path!("src/dclass/value")))
        .include(astron_path!("src/dclass"))
        .opt_level(3)
        .emit_rerun_if_env_changed(true);

    #[cfg(target_os = "linux")]
    compiler.flag("-Wno-sign-compare").flag("-lfl");

    compiler.compile("dclass");
}

struct CppFilesFromPath {
    dict: std::fs::ReadDir,
}

impl Iterator for CppFilesFromPath {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(entry) = self.dict.next() {
            if let Ok(entry) = entry {
                let path = entry.path();
                let extension = path.extension();
                if extension == Some(&OsStr::new("cpp")) {
                    return Some(path);
                }
            }
        }
        None
    }
}

impl CppFilesFromPath {
    fn new(path: &str) -> Self {
        let entries = std::fs::read_dir(path).unwrap();
        CppFilesFromPath { dict: entries }
    }
}
