#![allow(non_snake_case)]

use nvenc_sys::*;
use std::ffi::c_void;
use std::mem::zeroed;
use std::fmt;
use enum_primitive_derive::Primitive;
use num_traits::FromPrimitive;
use log::{error, debug};

pub use nvenc_sys::GUID;

#[derive(Primitive, Copy, Clone, Debug)]
#[repr(u32)]
pub enum Error {
    Uninitialized = 0,
    InvalidPointer = _NVENCSTATUS::NV_ENC_ERR_INVALID_PTR,
    InvalidEncodeDevice = _NVENCSTATUS::NV_ENC_ERR_INVALID_ENCODERDEVICE,
    DeviceNotExist = _NVENCSTATUS::NV_ENC_ERR_DEVICE_NOT_EXIST,
    UnsupportedParam = _NVENCSTATUS::NV_ENC_ERR_UNSUPPORTED_PARAM,
    OutOfMemory = _NVENCSTATUS::NV_ENC_ERR_OUT_OF_MEMORY,
    InvalidParam = _NVENCSTATUS::NV_ENC_ERR_INVALID_PARAM,
    InvalidVersion = _NVENCSTATUS::NV_ENC_ERR_INVALID_VERSION,
    Generic = _NVENCSTATUS::NV_ENC_ERR_GENERIC,
    Unknown = std::u32::MAX,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "An Error Occurred, Please Try Again! {}", self)
    }
}

type Result<T> = std::result::Result<T, Error>;

/// Device type used by NVDIA Video Codec SDK
#[repr(u32)]
pub enum DeviceType {
    /// Cuda
    Cuda = _NV_ENC_DEVICE_TYPE::NV_ENC_DEVICE_TYPE_CUDA,
    /// DirectX
    DirectX = _NV_ENC_DEVICE_TYPE::NV_ENC_DEVICE_TYPE_DIRECTX,
    /// OpenGL (Only usable on linux)
    OpenGL = _NV_ENC_DEVICE_TYPE::NV_ENC_DEVICE_TYPE_OPENGL,
}

/// Data format of input and output buffer
#[repr(u32)]
#[derive(Primitive, Copy, Clone, Debug)]
pub enum BufferFormat {
    Undefined = _NV_ENC_BUFFER_FORMAT::NV_ENC_BUFFER_FORMAT_UNDEFINED,
    NV12 = _NV_ENC_BUFFER_FORMAT::NV_ENC_BUFFER_FORMAT_NV12,
    YV12 = _NV_ENC_BUFFER_FORMAT::NV_ENC_BUFFER_FORMAT_YV12,
    IYUV = _NV_ENC_BUFFER_FORMAT::NV_ENC_BUFFER_FORMAT_IYUV,
    YUV444 = _NV_ENC_BUFFER_FORMAT::NV_ENC_BUFFER_FORMAT_YUV444,
    YUV444_10Bit = _NV_ENC_BUFFER_FORMAT::NV_ENC_BUFFER_FORMAT_YUV444_10BIT,
    YUV420_10Bit = _NV_ENC_BUFFER_FORMAT::NV_ENC_BUFFER_FORMAT_YUV420_10BIT,
    ARGB = _NV_ENC_BUFFER_FORMAT::NV_ENC_BUFFER_FORMAT_ARGB,
    ARGB10 = _NV_ENC_BUFFER_FORMAT::NV_ENC_BUFFER_FORMAT_ARGB10,
    ABGR = _NV_ENC_BUFFER_FORMAT::NV_ENC_BUFFER_FORMAT_ABGR,
    AYUV = _NV_ENC_BUFFER_FORMAT::NV_ENC_BUFFER_FORMAT_AYUV,
    ABGR10 = _NV_ENC_BUFFER_FORMAT::NV_ENC_BUFFER_FORMAT_ABGR10,
    U8 = _NV_ENC_BUFFER_FORMAT::NV_ENC_BUFFER_FORMAT_U8,
}

macro_rules! api_call {
    ($api:expr, $ret:expr, $($p:expr),+) => {
        if let Some(entry) = $api {
            let status = unsafe { entry($($p),+) };
            if status != _NVENCSTATUS::NV_ENC_SUCCESS {
                Err(Error::from_u32(status).unwrap_or(Error::Unknown))
            } else {
                Ok($ret)
            }
        } else { Err(Error::Uninitialized) }
    };
}

/// Encoder session object
pub struct Encoder {
    api: Api,
    encoder: *mut c_void,
}

impl Encoder {
    pub fn new(device_type: DeviceType, device: *mut c_void) -> Result<Self> {
        let api = Api::init()?;
        let mut params: NV_ENC_OPEN_ENCODE_SESSION_EX_PARAMS = unsafe{zeroed()};
        params.version = NV_ENC_OPEN_ENCODE_SESSION_EX_PARAMS_VER;
        params.apiVersion = NVENCAPI_VERSION;
        params.deviceType = device_type as u32;
        params.device = device;
        let mut encoder: *mut c_void = std::ptr::null_mut();
        api_call!(api.fptr.nvEncOpenEncodeSessionEx,
                Self { api: api, encoder: encoder },
                &mut params, &mut encoder)
    }

    pub fn support_codec(&self, guid: GUID) -> Result<bool> {
        let mut count = 0;
        api_call!(self.api.fptr.nvEncGetEncodeGUIDCount, () ,self.encoder, &mut count)?;

        let mut guids = Vec::with_capacity(count as usize);
        let mut returned = 0;
        api_call!(self.api.fptr.nvEncGetEncodeGUIDs, (), self.encoder,
                guids.as_mut_ptr(), count, &mut returned)?;

        unsafe { guids.set_len(returned as usize) };

        for g in guids.into_iter().take(returned as usize) {
            if guid == g { return Ok(true) }
        }
        Ok(false)
    }

    pub fn supported_presets(&self, encode: GUID) -> Result<Vec<GUID>> {
        let mut count = 0;
        api_call!(self.api.fptr.nvEncGetEncodePresetCount, (), self.encoder, encode, &mut count)?;

        let mut guids = Vec::with_capacity(count as usize);
        let mut returned = 0;
        api_call!(self.api.fptr.nvEncGetEncodePresetGUIDs, (), self.encoder,
                encode, guids.as_mut_ptr(), count, &mut returned)?;
        assert!(returned <= count);
        unsafe { guids.set_len(returned as usize) };
        Ok(guids)
    }

    pub fn supported_formats(&self, encode: GUID) -> Result<Vec<BufferFormat>> {
        let mut count = 0;
        api_call!(self.api.fptr.nvEncGetInputFormatCount, (), self.encoder, encode, &mut count)?;

        let mut formats = Vec::with_capacity(count as usize);
        let mut returned = 0;
        api_call!(self.api.fptr.nvEncGetInputFormats, (), self.encoder,
                encode, formats.as_mut_ptr(), count, &mut returned)?;
        assert!(returned <= count);
        unsafe { formats.set_len(returned as usize) };
        Ok(formats.into_iter().map(|f| BufferFormat::from_u32(f).unwrap_or(BufferFormat::Undefined)).collect())
    }

    pub fn preset_config(&self, encode: GUID, preset: GUID) -> Result<PresetConfig> {
        let mut config: NV_ENC_PRESET_CONFIG = unsafe { zeroed() };
        config.presetCfg.version = NV_ENC_CONFIG_VER;
        config.version = NV_ENC_PRESET_CONFIG_VER;

        api_call!(self.api.fptr.nvEncGetEncodePresetConfig,
                PresetConfig { preset: config},
                self.encoder, encode, preset, &mut config)
    }

    pub fn initialize(&self, init_params: &InitParams) -> Result<()> {
        // We can safely assume the params won't be changed by the API
        // according to the API documentation
        let params = init_params.init_params;
        let params = &params as *const NV_ENC_INITIALIZE_PARAMS;
        let params = params as *mut NV_ENC_INITIALIZE_PARAMS;

        api_call!(self.api.fptr.nvEncInitializeEncoder, (), self.encoder, params)
    }

    /// Allocate a new buffer managed by NVIDIA Video SDK
    pub fn alloc_input_buffer(&self,
        width: u32,
        height: u32,
        format: BufferFormat
    ) -> Result<InputBuffer> {
        let mut params: NV_ENC_CREATE_INPUT_BUFFER = unsafe { zeroed() };
        params.version = NV_ENC_CREATE_INPUT_BUFFER_VER;
        params.width = width;
        params.height = height;
        params.bufferFmt = format as u32;

        api_call!(self.api.fptr.nvEncCreateInputBuffer,
                InputBuffer{
                    ptr: params.inputBuffer,
                    format: format,
                    width: width,
                    height: height
                }, self.encoder, &mut params)
    }

    pub fn input_buffer_lock(&self,
        buffer: &InputBuffer
    ) -> Result<&mut [u32]> {
        let mut params: NV_ENC_LOCK_INPUT_BUFFER = unsafe { zeroed() };
        params.version = NV_ENC_LOCK_INPUT_BUFFER_VER;
        params.inputBuffer = buffer.ptr;

        api_call!(self.api.fptr.nvEncLockInputBuffer,
                unsafe {std::slice::from_raw_parts_mut(
                    params.bufferDataPtr as *mut u32,
                    (buffer.width * buffer.height) as usize) },
                self.encoder, &mut params)
    }

    pub fn input_buffer_unlock(&self, buffer: &InputBuffer) -> Result<()> {
        api_call!(self.api.fptr.nvEncUnlockInputBuffer, (), self.encoder, buffer.ptr)
    }

    pub fn alloc_output_buffer(&self) -> Result<OutputBuffer> {
        let mut params: NV_ENC_CREATE_BITSTREAM_BUFFER = unsafe { zeroed() };
        params.version = NV_ENC_CREATE_BITSTREAM_BUFFER_VER;
        api_call!(self.api.fptr.nvEncCreateBitstreamBuffer,
                OutputBuffer {
                    ptr: params.bitstreamBufferPtr
                }, self.encoder, &mut params)
    }

    pub fn output_buffer_lock(&self, buffer: &InputBuffer) -> Result<*mut c_void> {
        let mut params: NV_ENC_LOCK_BITSTREAM = unsafe { zeroed() };
        params.version = NV_ENC_LOCK_INPUT_BUFFER_VER;
        params.outputBitstream = buffer.ptr;

        api_call!(self.api.fptr.nvEncLockBitstream, params.bitstreamBufferPtr, self.encoder, &mut params)
    }

    pub fn output_buffer_unlock(&self, buffer: &InputBuffer) -> Result<()> {
        api_call!(self.api.fptr.nvEncUnlockBitstream, (), self.encoder, buffer.ptr)
    }

    /// Main entry to encode a video frame
    pub fn encode(&self, input: InputBuffer, output: OutputBuffer) -> Result<()> {
        let mut params: NV_ENC_PIC_PARAMS = unsafe { zeroed() };
        params.version = NV_ENC_PIC_PARAMS_VER;
        params.inputBuffer = input.ptr;
        params.bufferFmt = input.format as u32;
        params.inputWidth = input.width;
        params.inputHeight = input.height;
        params.inputPitch = input.width;
        params.outputBitstream = output.ptr;

        api_call!(self.api.fptr.nvEncEncodePicture, (), self.encoder, &mut params)
    }
}

impl Drop for Encoder {
    fn drop(&mut self) {
        match api_call!(self.api.fptr.nvEncDestroyEncoder, (), self.encoder) {
            Ok(()) => (),
            Err(err) => error!("failed to destroy the encoder: {}", err)
        }
    }
}

pub struct OutputBuffer {
    ptr: NV_ENC_OUTPUT_PTR,
}

/// A simple wrapper of a buffer
pub struct InputBuffer {
    ptr: NV_ENC_INPUT_PTR,
    format: BufferFormat,
    width: u32,
    height: u32,
}

/// Preset configuration which provided by NVIDIA Video SDK
pub struct PresetConfig {
    preset: NV_ENC_PRESET_CONFIG,
}

/// Parameters used to initialize the encoder
pub struct InitParams {
    init_params: NV_ENC_INITIALIZE_PARAMS,
}

pub struct InitParamsBuilder(InitParams);

impl InitParamsBuilder {
    pub fn new(encode: GUID) -> Self {
        let mut init = InitParams{ init_params: unsafe { std::mem::zeroed() } };
        init.init_params.version = NV_ENC_INITIALIZE_PARAMS_VER;
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

    pub fn dar_width(mut self ,width: u32) -> Self {
        self.0.init_params.darWidth = width;
        self
    }

    pub fn dar_height(mut self, height: u32) -> Self {
        self.0.init_params.darHeight = height;
        self
    }

    pub fn max_width(mut self, width: u32) -> Self {
        self.0.init_params.maxEncodeWidth = width;
        self
    }

    pub fn max_height(mut self, height: u32) -> Self {
        self.0.init_params.maxEncodeHeight = height;
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
    pub fn init() -> Result<Self> {
        let mut function_list: NV_ENCODE_API_FUNCTION_LIST = unsafe {zeroed()};
        function_list.version = NV_ENCODE_API_FUNCTION_LIST_VER;

        let status = unsafe { NvEncodeAPICreateInstance(&mut function_list) };
        if status == _NVENCSTATUS::NV_ENC_SUCCESS {
            Ok(Self {fptr: function_list})
        } else { Err(Error::from_u32(status).unwrap_or(Error::Unknown)) }
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
pub fn max_version_supported() -> Result<Version> {
    let mut value: u32 = 0;
    let status = unsafe { NvEncodeAPIGetMaxSupportedVersion(&mut value) };

    if status == _NVENCSTATUS::NV_ENC_SUCCESS {
        Ok(Version { major: value >> 4, minor: value & 0xf })
    } else { Err(Error::from_u32(status).unwrap_or(Error::Unknown)) }
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

            let mut device: cuda::CUdevice = zeroed();

            let ret = cuda::cuDeviceGet(&mut device, 0);
            assert_eq!(ret, cuda::CUDA_SUCCESS);

            let v = Vec::<u8>::with_capacity(30);
            let cs = std::ffi::CString::from_vec_unchecked(v);

            let c_raw = cs.into_raw();
            let ret = cuda::cuDeviceGetName(c_raw, 30, device);
            let cs = std::ffi::CString::from_raw(c_raw);
            assert_eq!(ret, cuda::CUDA_SUCCESS);

            println!("device name: {:?}", cs);

            let mut context: cuda::CUcontext = zeroed();

            let ret = cuda::cuCtxCreate_v2(&mut context, 0, device);
            assert_eq!(ret, cuda::CUDA_SUCCESS);
            context
        }
    }

    #[test]
    fn session_create() {
        let context = init_cuda_context();
        assert!(Encoder::new(DeviceType::Cuda, context as *mut c_void).is_ok())
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
        let session = Encoder::new(DeviceType::Cuda, context as *mut c_void).unwrap();
        let supported = session.support_codec(h264_guid);
        assert!(supported.is_ok());
        assert!(supported.unwrap())
    }

    #[test]
    fn version_check() {
        assert!(max_version_supported().is_ok())
    }

    #[test]
    fn api_create_instance() {
        assert!(Api::init().is_ok())
    }
}
