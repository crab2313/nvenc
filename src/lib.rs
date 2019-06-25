#![allow(non_snake_case)]

use nvenc_sys::*;
use std::ffi::c_void;
use std::mem::uninitialized;

/// Device type used by NVDIA Video Codec SDK
pub enum DeviceType {
    /// Cuda
    Cuda = _NV_ENC_DEVICE_TYPE::NV_ENC_DEVICE_TYPE_CUDA as isize,
    /// DirectX
    DirectX = _NV_ENC_DEVICE_TYPE::NV_ENC_DEVICE_TYPE_DIRECTX as isize,
    /// OpenGL (Only usable on linux)
    OpenGL = _NV_ENC_DEVICE_TYPE::NV_ENC_DEVICE_TYPE_OPENGL as isize,
}

pub struct Session {
    api: Api,
    session: NV_ENC_OPEN_ENCODE_SESSION_EX_PARAMS,
    encoder: *mut c_void,
}

impl Session {
    pub fn new(device_type: DeviceType, device: *mut c_void) -> Option<Self> {
        let api = Api::init()?;
        let mut session = unsafe { NV_ENC_OPEN_ENCODE_SESSION_EX_PARAMS {
                apiVersion: NVENCAPI_VERSION,
                version: NV_ENC_OPEN_ENCODE_SESSION_EX_PARAMS_VER,
                deviceType: device_type as u32,
                device: device,
                reserved1: uninitialized(),
                reserved2: uninitialized(),
                reserved: uninitialized(),
        }};
        let mut encoder: *mut c_void = std::ptr::null_mut();
        let status = unsafe { api.fptr.nvEncOpenEncodeSessionEx?(&mut session, &mut encoder) };

        if status == _NVENCSTATUS::NV_ENC_SUCCESS {
            Some(Self { session: session, api: api, encoder: encoder})
        } else { None }
    }

    pub fn support(&self, guid: GUID) -> Option<bool> {
        let mut count = 0;
        let status = unsafe { self.api.fptr.nvEncGetEncodeGUIDCount?(self.encoder, &mut count) };

        if status != _NVENCSTATUS::NV_ENC_SUCCESS { return None; }

        let guids = Vec::with_capacity(count as usize);
        let mut returned = 0;
        let status = unsafe { self.api.fptr.nvEncGetEncodeGUIDs?(self.encoder,
                guids.as_mut_ptr(), count, &mut returned) };

        for g in guids.into_iter().take(returned as usize) {
            if guid == g { return Some(true) }
        }
        Some(false)
    }
}

/// API calling entry for NVIDIA Video Codec
pub struct Api {
    fptr: NV_ENCODE_API_FUNCTION_LIST,
}

impl Api {
    /// Create a new instance of API
    pub fn init() -> Option<Self> {
        let mut function_list: NV_ENCODE_API_FUNCTION_LIST = unsafe {uninitialized()};
        function_list.version = NV_ENCODE_API_FUNCTION_LIST_VER;

        let status = unsafe { NvEncodeAPICreateInstance(&mut function_list) };
        if status == _NVENCSTATUS::NV_ENC_SUCCESS {
            Some(Self {fptr: function_list})
        } else { None }
    }
}

/// API version of NVIDIA Codec SDK
pub struct Version {
    /// Major version
    pub major: u32,
    /// Monior version
    pub minor: u32,
}

/// Returns the maximum version of the runtime driver
pub fn max_version_supported() -> Option<Version> {
    let mut value: u32 = 0;
    let ret = unsafe { NvEncodeAPIGetMaxSupportedVersion(&mut value) };

    if ret == _NVENCSTATUS::NV_ENC_SUCCESS {
        Some(Version { major: value & 0xf, minor: value >> 4})
    } else { None }
}

#[cfg(test)]
mod tests {
    use crate::*;
    #[test]
    fn session_create() {
        assert!(Session::new(DeviceType::Cuda, std::ptr::null_mut()).is_some())
    }

    #[test]
    fn version_check() {
        assert!(max_version_supported().is_some())
    }

    #[test]
    fn api_create_instance() {
        assert!(Api::init().is_some())
    }
}
