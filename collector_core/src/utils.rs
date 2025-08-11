#[cfg(target_os = "windows")]
use std::{
    mem,
    ptr::null_mut
};

use std::path::PathBuf;

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

#[derive(Clone, Debug)]
pub struct FormatSource {
    source: PathBuf,
}

impl FormatSource {
    pub fn from<S: AsRef<str>>(source: S) -> Self {
        FormatSource {
            source: PathBuf::from(source.as_ref()),
        }
    }

    pub fn to_path(&mut self) -> PathBuf {
        self.source.clone()
    }

    pub fn to_string(&self) -> String {
        self.source.to_str().unwrap().to_string().clone()
    }

    pub fn push(&mut self,value: &str) -> Self {
        let _ = &self.source.push(value);
        self.clone()
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