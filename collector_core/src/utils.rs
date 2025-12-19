use crate::error::{CollectorError, Result};
use std::path::{Path, PathBuf};

pub const FILE_BUFFER_SIZE: usize = 32 * 1024;
pub const NTFS_READ_BUFFER_SIZE: usize = 32 * 1024;
pub const HASH_BUFFER_SIZE: usize = 4 * 1024;

/// Path wrapper with convenient manipulation methods.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct FormatSource {
    path: PathBuf,
}

impl FormatSource {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    pub fn as_path(&self) -> &Path {
        &self.path
    }

    pub fn into_path_buf(self) -> PathBuf {
        self.path
    }

    pub fn to_path_buf(&self) -> PathBuf {
        self.path.clone()
    }

    pub fn to_string_lossy(&self) -> String {
        self.path.to_string_lossy().to_string()
    }

    pub fn push<P: AsRef<Path>>(&mut self, value: P) -> &mut Self {
        self.path.push(value);
        self
    }

    pub fn join<P: AsRef<Path>>(&self, value: P) -> Self {
        Self {
            path: self.path.join(value),
        }
    }

    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    pub fn is_dir(&self) -> bool {
        self.path.is_dir()
    }

    pub fn is_file(&self) -> bool {
        self.path.is_file()
    }
}

impl std::fmt::Display for FormatSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path.display())
    }
}

impl AsRef<Path> for FormatSource {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}

impl From<PathBuf> for FormatSource {
    fn from(path: PathBuf) -> Self {
        Self { path }
    }
}

impl From<&str> for FormatSource {
    fn from(s: &str) -> Self {
        Self {
            path: PathBuf::from(s),
        }
    }
}

impl From<String> for FormatSource {
    fn from(s: String) -> Self {
        Self {
            path: PathBuf::from(s),
        }
    }
}

impl From<FormatSource> for PathBuf {
    fn from(source: FormatSource) -> Self {
        source.path
    }
}

#[cfg(target_os = "windows")]
pub fn is_admin() -> bool {
    use std::mem;
    use std::ptr::null_mut;
    use winapi::shared::minwindef::{DWORD, LPVOID};
    use winapi::um::processthreadsapi::{GetCurrentProcess, OpenProcessToken};
    use winapi::um::securitybaseapi::GetTokenInformation;
    use winapi::um::winnt::{HANDLE, TOKEN_ELEVATION, TOKEN_QUERY, TokenElevation};

    unsafe {
        let mut token: HANDLE = null_mut();
        let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
        let mut size: DWORD = 0;

        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) != 0 {
            if GetTokenInformation(
                token,
                TokenElevation,
                &mut elevation as *mut _ as LPVOID,
                mem::size_of::<TOKEN_ELEVATION>() as u32,
                &mut size,
            ) != 0
            {
                return elevation.TokenIsElevated != 0;
            }
        }
        false
    }
}

// #[cfg(target_os = "windows")]
// pub fn is_acdmin() -> bool {
//     use std::mem;
//     use windows::Win32::Foundation::HANDLE;
//     use windows::Win32::Security::{
//         GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
//     };
//     use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

//     unsafe {
//         let mut token: HANDLE = HANDLE::default();
//         let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
//         let mut size: u32 = 0;

//         let process = GetCurrentProcess();

//         if OpenProcessToken(process, TOKEN_QUERY, &mut token).is_ok() {
//             let result = GetTokenInformation(
//                 token,
//                 TokenElevation,
//                 Some(&mut elevation as *mut _ as *mut _),
//                 mem::size_of::<TOKEN_ELEVATION>() as u32,
//                 &mut size,
//             );

//             // Fermer le handle du token
//             let _ = windows::Win32::Foundation::CloseHandle(token);

//             if result.is_ok() {
//                 return elevation.TokenIsElevated != 0;
//             }
//         }
//         false
//     }
// }


#[cfg(target_os = "linux")]
pub fn is_admin() -> bool {
    use nix::unistd::Uid;
    Uid::current().is_root()
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn is_admin() -> bool {
    false
}

pub fn require_admin() -> Result<()> {
    if !is_admin() {
        return Err(CollectorError::InsufficientPrivileges);
    }
    Ok(())
}

pub fn normalize_path(path: &str) -> String {
    path.replace(':', "")
        .trim_start_matches('\\')
        .trim_start_matches('/')
        .to_string()
}

pub fn truncate_text(text: &str, max_len: usize) -> String {
    if text.len() > max_len {
        format!("{}...", &text[..max_len])
    } else {
        text.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_source_new() {
        let source = FormatSource::new("/home/user");
        assert_eq!(source.to_path_buf(), PathBuf::from("/home/user"));
    }

    #[test]
    fn test_format_source_push() {
        let mut source = FormatSource::new("/home");
        source.push("user");
        assert_eq!(source.to_path_buf(), PathBuf::from("/home/user"));
    }

    #[test]
    fn test_format_source_join() {
        let source = FormatSource::new("/home");
        let new_source = source.join("user");

        // Original unchanged
        assert_eq!(source.to_path_buf(), PathBuf::from("/home"));
        // New one has joined path
        assert_eq!(new_source.to_path_buf(), PathBuf::from("/home/user"));
    }

    #[test]
    fn test_format_source_from_string() {
        let source: FormatSource = "/test/path".into();
        assert_eq!(source.to_string_lossy(), "/test/path");
    }

    #[test]
    fn test_format_source_display() {
        let source = FormatSource::new("/test/path");
        assert_eq!(format!("{}", source), "/test/path");
    }

    #[test]
    fn test_normalize_path_windows() {
        assert_eq!(normalize_path("\\Users\\test"), "Users\\test");
        assert_eq!(normalize_path("\\data"), "data");
    }

    #[test]
    fn test_normalize_path_unix() {
        assert_eq!(normalize_path("/home/user"), "home/user");
        assert_eq!(normalize_path("/var/log"), "var/log");
    }

    #[test]
    fn test_truncate_text() {
        assert_eq!(truncate_text("Hello World", 5), "Hello...");
        assert_eq!(truncate_text("Hi", 5), "Hi");
        assert_eq!(truncate_text("Exact", 5), "Exact");
    }

    #[test]
    fn test_format_source_as_ref() {
        let source = FormatSource::new("/test");
        let path: &Path = source.as_ref();
        assert_eq!(path, Path::new("/test"));
    }
}
