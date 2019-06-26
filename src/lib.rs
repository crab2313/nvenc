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


/// Encoder session object
pub struct Session {
    api: Api,
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
            Some(Self { api: api, encoder: encoder})
        } else { None }
    }

    pub fn support(&self, guid: GUID) -> Option<bool> {
        let mut count = 0;
        let status = unsafe { self.api.fptr.nvEncGetEncodeGUIDCount?(self.encoder, &mut count) };

        if status != _NVENCSTATUS::NV_ENC_SUCCESS { return None; }

        let mut guids = Vec::with_capacity(count as usize);
        let mut returned = 0;
        let status = unsafe { self.api.fptr.nvEncGetEncodeGUIDs?(self.encoder,
                guids.as_mut_ptr(), count, &mut returned) };

        if status != _NVENCSTATUS::NV_ENC_SUCCESS { return None; }

        unsafe { guids.set_len(returned as usize) };

        println!("{:?} {}", guids, returned
        );
        for g in guids.into_iter().take(returned as usize) {
            if guid == g { return Some(true) }
        }
        Some(false)
    }

    pub fn preset_config(&self, encode: GUID, preset: GUID) -> Option<PresetConfig> {
        let mut config: NV_ENC_PRESET_CONFIG = unsafe { uninitialized() };
        config.presetCfg.version = NV_ENC_CONFIG_VER;
        config.version = NV_ENC_PRESET_CONFIG_VER;

        let status = unsafe {
            self.api.fptr.nvEncGetEncodePresetConfig?(self.encoder, encode, preset, &mut config)
        };

        if status != _NVENCSTATUS::NV_ENC_SUCCESS { return None; }
        Some(PresetConfig { preset: config })
    }

    pub fn initialize(&self, init_params: &mut InitParams) -> Option<bool> {
        let mut params = init_params.init_params;
        let status = unsafe { self.api.fptr.nvEncInitializeEncoder?(self.encoder, &mut params) };

        if status != _NVENCSTATUS::NV_ENC_SUCCESS { return Some(false) }
        else { Some(true) }
    }
}

/// Preset configuration which provided by NVIDIA Video SDK
pub struct PresetConfig {
    preset: NV_ENC_PRESET_CONFIG,
}

/// Parameters used to initialize the encoder
pub struct InitParams {
    init_params: NV_ENC_INITIALIZE_PARAMS,
}

struct InitParamsBuilder(InitParams);

impl InitParamsBuilder {
    pub fn new(encode: GUID) -> Self {
        let mut init = InitParams{ init_params: unsafe { std::mem::zeroed() } };
        init.init_params.encodeGUID = encode;
        Self(init)
    }

    pub fn width(mut self, width: u32) -> Self {
        self.0.init_params.encodeWidth = width;
        self
    }

    pub fn height(mut self, height : u32) -> Self {
        self.0.init_params.encodeHeight = height;
        self
    }

    pub fn preset_guid(mut self, preset: GUID) -> Self {
        self.0.init_params.presetGUID = preset;
        self
    }

    pub fn preset_config(mut self, mut preset: PresetConfig) -> Self {
        let config = &mut preset.preset.presetCfg;
        self.0.init_params.encodeConfig = config;
        self
    }

    pub fn framerate(mut self, num: u32, den: u32) -> Self {
        self.0.init_params.frameRateNum = num;
        self.0.init_params.frameRateDen = den;
        self
    }

    pub fn ptd(mut self, enable: bool) -> Self {
        self.0.init_params.enablePTD = enable as u32;
        self
    }

    pub fn build(self) -> InitParams {
        self.0
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
    use ::cuda::ffi::cuda;

    fn init_cuda_context() -> cuda::CUcontext {
        unsafe {
            let ret = cuda::cuInit(0);
            assert_eq!(ret, cuda::CUDA_SUCCESS, "failed to init cuda");
            let mut count: i32 = 0;
            let ret = cuda::cuDeviceGetCount(&mut count as *mut i32);
            assert_eq!(ret, cuda::CUDA_SUCCESS);

            println!("found {} cuda capable devices", count);

            let mut device: cuda::CUdevice = uninitialized();

            let ret = cuda::cuDeviceGet(&mut device, 0);
            assert_eq!(ret, cuda::CUDA_SUCCESS);

            let v = Vec::<u8>::with_capacity(30);
            let cs = std::ffi::CString::from_vec_unchecked(v);

            let c_raw = cs.into_raw();
            let ret = cuda::cuDeviceGetName(c_raw, 30, device);
            let cs = std::ffi::CString::from_raw(c_raw);
            assert_eq!(ret, cuda::CUDA_SUCCESS);

            println!("device name: {:?}", cs);

            let mut context: cuda::CUcontext = uninitialized();

            let ret = cuda::cuCtxCreate_v2(&mut context, 0, device);
            assert_eq!(ret, cuda::CUDA_SUCCESS);
            context
        }
    }

    #[test]
    fn session_create() {
        let context = init_cuda_context();
        assert!(Session::new(DeviceType::Cuda, context as *mut c_void).is_some())
    }

    #[test]
    fn h264() {
        let h264_guid = GUID {
            Data1: 0x6bc82762,
            Data2: 0x4e63,
            Data3: 0x4ca4,
            Data4: [ 0xaa, 0x85, 0x1e, 0x50, 0xf3, 0x21, 0xf6, 0xbf]
        };
        let context = init_cuda_context();
        let session = Session::new(DeviceType::Cuda, context as *mut c_void).unwrap();
        let supported = session.support(h264_guid);
        assert!(supported.is_some());
        assert!(supported.unwrap())
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
