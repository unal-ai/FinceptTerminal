// Utility module for Python execution with PyO3
use std::path::PathBuf;
use tauri::Manager;

/// NumPy 1.x compatible libraries (use venv-numpy1)
const NUMPY1_LIBRARIES: &[&str] = &[
    "vectorbt",
    "backtesting",
    "gluonts",
    "functime",
    "PyPortfolioOpt",
    "pyqlib",
    "rdagent",
    "gs-quant",
];

/// Determine which venv to use based on library name
fn get_venv_for_library(library_name: Option<&str>) -> &'static str {
    if let Some(lib) = library_name {
        // Check if library requires NumPy 1.x
        if NUMPY1_LIBRARIES.iter().any(|&numpy1_lib| lib.contains(numpy1_lib)) {
            return "venv-numpy1";
        }
    }
    // Default to NumPy 2.x venv
    "venv-numpy2"
}

/// Get the Python executable path from app installation directory
/// Supports dual-venv setup for NumPy 1.x and 2.x compatibility
/// In production: Uses installed Python from setup.rs
/// In development: Uses %LOCALAPPDATA%/fincept-dev (matches setup.rs)
pub fn get_python_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    get_python_path_for_library(app, None)
}

/// Get Python path for a specific library (switches between numpy1 and numpy2 venvs)
pub fn get_python_path_for_library(app: &tauri::AppHandle, library_name: Option<&str>) -> Result<PathBuf, String> {
    // Get install directory - MUST match setup.rs get_install_dir()
    let install_dir = if cfg!(debug_assertions) {
        // Dev mode: use LOCALAPPDATA/fincept-dev
        let base_dir = if cfg!(target_os = "windows") {
            std::env::var("LOCALAPPDATA")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("C:\\Users\\Default\\AppData\\Local"))
        } else if cfg!(target_os = "macos") {
            std::env::var("HOME")
                .map(|h| PathBuf::from(h).join("Library/Application Support"))
                .unwrap_or_else(|_| PathBuf::from("/tmp"))
        } else {
            std::env::var("HOME")
                .map(|h| PathBuf::from(h).join(".local/share"))
                .unwrap_or_else(|_| PathBuf::from("/tmp"))
        };
        base_dir.join("fincept-dev")
    } else {
        // Production: use app data directory
        app.path().app_data_dir()
            .map_err(|e| format!("Failed to get app data dir: {}", e))?
    };

    // Determine which venv to use based on library
    let venv_name = get_venv_for_library(library_name);

    // Platform-specific Python executable location in venv
    let python_exe = if cfg!(target_os = "windows") {
        install_dir.join(format!("{}/Scripts/python.exe", venv_name))
    } else {
        install_dir.join(format!("{}/bin/python3", venv_name))
    };


    // Check if venv Python exists
    if python_exe.exists() {
        // Strip the \\?\ prefix that can cause issues on Windows
        let path_str = python_exe.to_string_lossy().to_string();
        let clean_path = if path_str.starts_with(r"\\?\") {
            PathBuf::from(&path_str[4..])
        } else {
            python_exe.clone()
        };
        return Ok(clean_path);
    }

    // Fallback to system Python in dev mode only
    #[cfg(debug_assertions)]
    {
        use std::process::Command;
        let system_python = if cfg!(target_os = "windows") {
            "python"
        } else {
            "python3"
        };

        if let Ok(output) = Command::new(system_python).arg("--version").output() {
            if output.status.success() {
                return Ok(PathBuf::from(system_python));
            }
        }
    }

    // If we get here, Python is not available
    Err(format!(
        "Python interpreter not found at: {}\n\n\
        The Python runtime is required to run analytics and data processing features.\n\n\
        Troubleshooting steps:\n\
        1. Run the setup wizard from the application to install Python\n\
        2. Ensure both venv-numpy1 and venv-numpy2 are created\n\
        3. If setup fails, ensure Microsoft Visual C++ Redistributable is installed:\n\
           Download from: https://aka.ms/vs/17/release/vc_redist.x64.exe\n\
        4. Restart the application after installation",
        python_exe.display()
    ))
}

/// Get the Bun executable path from app installation directory
pub fn get_bundled_bun_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    get_bundled_bun_path_for_runtime(Some(app))
}

/// Resolve the Bun executable path in a runtime-agnostic way.
///
/// This function is similar to [`get_bundled_bun_path`] but is designed to work
/// in contexts where a Tauri [`AppHandle`] may not be available, such as
/// background tasks, CLIs, or web server runtimes.
///
/// # Parameters
///
/// * `app` - Optional Tauri application handle used to determine the
///   installation directory when running inside a Tauri application. Pass
///   `None` when calling from a non-Tauri or server context where no
///   `AppHandle` is available.
///
/// # Returns
///
/// On success, returns the resolved [`PathBuf`] to the Bun executable,
/// preferring a bundled Bun distribution and, in debug builds, potentially
/// falling back to a system-installed `bun`. On failure, returns a
/// human-readable error message describing the missing locations.
///
/// # Differences from [`get_bundled_bun_path`]
///
/// * `get_bundled_bun_path` requires a non-optional [`AppHandle`] and is
///   intended for use from within the main Tauri application.
/// * `get_bundled_bun_path_for_runtime` accepts an optional [`AppHandle`] and
///   can be safely used from generic runtime or server code where Tauri
///   context may not exist.
pub fn get_bundled_bun_path_for_runtime(app: Option<&tauri::AppHandle>) -> Result<PathBuf, String> {
    let install_dir = get_install_dir_for_runtime(app)?;

    // Platform-specific Bun executable location
    let bun_candidates = if cfg!(target_os = "windows") {
        vec![
            install_dir.join("bun").join("bun.exe"),
            install_dir.join("bun").join("bin").join("bun.exe"),
        ]
    } else {
        vec![
            install_dir.join("bun").join("bin").join("bun"),
            install_dir.join("bun").join("bun"),
        ]
    };

    // DEVELOPMENT MODE: Prefer system Bun
    #[cfg(debug_assertions)]
    {
        use std::process::Command;

        let system_bun = if cfg!(target_os = "windows") {
            "bun.exe"
        } else {
            "bun"
        };

        // Try system Bun first in dev mode
        if let Ok(output) = Command::new(system_bun).arg("--version").output() {
            if output.status.success() {
                return Ok(PathBuf::from(system_bun));
            }
        }
    }

    // PRODUCTION MODE or dev fallback: Check bundled Bun
    for bun_exe in &bun_candidates {
        if bun_exe.exists() {
            return Ok(bun_exe.clone());
        }
    }

    let candidate_list = bun_candidates
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>()
        .join(", ");

    // If we get here, Bun is not available
    Err(format!(
        "Bun not found at any of: {}\n\nPlease run the setup process to install Bun.",
        candidate_list
    ))
}

/// Determine the installation directory used to locate bundled runtimes and tools.
///
/// When running in debug mode, this returns a development-specific directory
/// (e.g. `LOCALAPPDATA/fincept-dev` on Windows) to avoid interfering with the
/// production installation.
///
/// In release mode, if a Tauri [`AppHandle`] is provided, this uses
/// `app.path().app_data_dir()` as the base. If no handle is available, it first
/// checks the `FINCEPT_APP_DATA_DIR` environment variable, and if unset falls
/// back to platform-specific default app data locations (e.g. `APPDATA` on
/// Windows, `~/Library/Application Support` on macOS, or `XDG_DATA_HOME` /
/// `~/.local/share` on Linux).
///
/// The logic in this function MUST remain in sync with `setup.rs::get_install_dir()`
/// so that the installer and the runtime both agree on where the application data
/// and bundled binaries are stored.
fn get_install_dir_for_runtime(app: Option<&tauri::AppHandle>) -> Result<PathBuf, String> {
    // Get install directory - MUST match setup.rs get_install_dir()
    if cfg!(debug_assertions) {
        // Dev mode: use LOCALAPPDATA/fincept-dev
        let base_dir = if cfg!(target_os = "windows") {
            std::env::var("LOCALAPPDATA")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("C:\\Users\\Default\\AppData\\Local"))
        } else if cfg!(target_os = "macos") {
            std::env::var("HOME")
                .map(|h| PathBuf::from(h).join("Library/Application Support"))
                .unwrap_or_else(|_| PathBuf::from("/tmp"))
        } else {
            std::env::var("HOME")
                .map(|h| PathBuf::from(h).join(".local/share"))
                .unwrap_or_else(|_| PathBuf::from("/tmp"))
        };
        return Ok(base_dir.join("fincept-dev"));
    }

    if let Some(app_handle) = app {
        return app_handle
            .path()
            .app_data_dir()
            .map_err(|e| format!("Failed to get app data dir: {}", e));
    }

    if let Ok(custom_dir) = std::env::var("FINCEPT_APP_DATA_DIR") {
        return Ok(PathBuf::from(custom_dir));
    }

    let base_dir = if cfg!(target_os = "windows") {
        std::env::var("APPDATA")
            .or_else(|_| std::env::var("PROGRAMDATA"))
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("C:\\ProgramData"))
    } else if cfg!(target_os = "macos") {
        std::env::var("HOME")
            .map(|h| PathBuf::from(h).join("Library/Application Support"))
            .unwrap_or_else(|_| PathBuf::from("/tmp"))
    } else {
        std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|_| {
                std::env::var("HOME").map(|h| PathBuf::from(h).join(".local/share"))
            })
            .unwrap_or_else(|_| PathBuf::from("/tmp"))
    };

    Ok(base_dir.join("com.fincept.terminal"))
}

/// Get a Python script path at runtime
/// Works in dev mode, production builds, and CI/CD pipelines
pub fn get_script_path(app: &tauri::AppHandle, script_name: &str) -> Result<PathBuf, String> {
    get_script_path_for_runtime(Some(app), script_name)
}

/// Resolve a Python script path in both Tauri and non-Tauri runtimes.
pub fn get_script_path_for_runtime(
    app: Option<&tauri::AppHandle>,
    script_name: &str,
) -> Result<PathBuf, String> {
    // SECURITY: Validate script_name to prevent path traversal attacks
    if script_name.contains("..") || script_name.contains("/") || script_name.contains("\\") {
        return Err(format!(
            "Invalid script name '{}': path traversal not allowed",
            script_name
        ));
    }

    // Strategy: Try multiple paths in order until we find the script
    let mut candidate_paths = Vec::new();

    // 0. Optional override for server/runtime deployments
    // SECURITY WARNING: FINCEPT_SCRIPTS_PATH should only be set in trusted environments.
    // An attacker who can control this environment variable could execute arbitrary Python code.
    // Only use this in development/testing, not in production deployments.
    if let Ok(custom_dir) = std::env::var("FINCEPT_SCRIPTS_PATH") {
        let custom_path = PathBuf::from(&custom_dir);
        
        // Validate that the custom path is absolute and exists
        if !custom_path.is_absolute() {
            return Err(format!(
                "FINCEPT_SCRIPTS_PATH must be an absolute path, got: {}",
                custom_dir
            ));
        }
        
        if !custom_path.exists() {
            return Err(format!(
                "FINCEPT_SCRIPTS_PATH directory does not exist: {}",
                custom_dir
            ));
        }
        
        candidate_paths.push(custom_path.join(script_name));
    }

    // 1. Try Tauri's resource_dir (works in production and should work in dev)
    if let Some(app) = app {
        if let Ok(resource_dir) = app.path().resource_dir() {
            candidate_paths.push(resource_dir.join("scripts").join(script_name));
        }
    }

    // 2. Try relative to current executable (production fallback)
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            candidate_paths.push(exe_dir.join("resources").join("scripts").join(script_name));
            candidate_paths.push(exe_dir.join("scripts").join(script_name));
        }
    }

    // 3. Try relative to current working directory (dev mode)
    if let Ok(cwd) = std::env::current_dir() {
        // If CWD is src-tauri
        candidate_paths.push(cwd.join("resources").join("scripts").join(script_name));
        // If CWD is project root
        candidate_paths.push(cwd.join("src-tauri").join("resources").join("scripts").join(script_name));
    }

    // Try each candidate path
    for path in candidate_paths.iter() {
        if path.exists() {
            // Strip the \\?\ prefix that Python can't handle on Windows
            let path_str = path.to_string_lossy().to_string();
            let clean_path = if path_str.starts_with(r"\\?\") {
                PathBuf::from(&path_str[4..])
            } else {
                path.clone()
            };

            return Ok(clean_path);
        }
    }

    Err(format!(
        "Script '{}' not found in any of {} candidate locations",
        script_name,
        candidate_paths.len()
    ))
}

/// Execute Python script with PyO3 embedded runtime
/// This is the primary execution method - fast, embedded, no subprocess
pub fn execute_python_script_simple(
    app: &tauri::AppHandle,
    script_relative_path: &str,
    args: &[&str],
) -> Result<String, String> {
    let script_path = get_script_path(app, script_relative_path)?;
    let args_vec: Vec<String> = args.iter().map(|s| s.to_string()).collect();

    // Execute with PyO3
    crate::python_runtime::execute_python_script(&script_path, args_vec)
}
