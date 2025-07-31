use std::mem;
use std::path::PathBuf;
use std::ptr::null_mut;

#[cfg(target_family = "windows")]
use winapi::{
	shared::minwindef::{DWORD, LPVOID},
	um::{
		processthreadsapi::{
			GetCurrentProcess,
			OpenProcessToken,
		},
		securitybaseapi::GetTokenInformation,
		winnt::{
			TokenElevation,
			HANDLE,
			TOKEN_ELEVATION,
			TOKEN_QUERY,
		},
	},
};

#[cfg(target_os = "linux")]
use nix::unistd::Uid;


pub struct FormatSource {
    source: String,
}

impl FormatSource {
    pub fn from<S: AsRef<str>>(source: S) -> Self {
        FormatSource {
            source: source.as_ref().to_string(),
        }
    }

    pub fn to_path(&mut self) -> PathBuf {
        let get_self_string: String = self.to_string();
        PathBuf::from(get_self_string)
    }

    pub fn to_string(&mut self) -> String {
        if !self.source.ends_with("/") || !self.source.ends_with("\\") {
            self.source.push('\\');
        }
        if self.source.starts_with("/") || self.source.starts_with("\\") {
            self.source.remove(0);
        }
        self.source.clone()
    }
}


#[cfg(target_os = "windows")]
pub fn is_admin() -> bool {
    let mut current_token_ptr: HANDLE = null_mut();
    let mut token_elevation: TOKEN_ELEVATION = TOKEN_ELEVATION {
        TokenIsElevated: 0,
    };
    let token_elevation_type_ptr: *mut TOKEN_ELEVATION = &mut token_elevation;
    let mut size: DWORD = 0;

    let result = unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut current_token_ptr) };

    if result != 0 {
        let result = unsafe {
            GetTokenInformation(
                current_token_ptr,
                TokenElevation,
                token_elevation_type_ptr as LPVOID,
                mem::size_of::<winapi::um::winnt::TOKEN_ELEVATION_TYPE>() as u32,
                &mut size,
            )
        };
        if result != 0 {
            return token_elevation.TokenIsElevated != 0;
        }
    }
    false
}

#[cfg(target_os = "linux")]
pub fn is_admin() -> bool {
    let user = Uid::current();
    user.is_root()
}