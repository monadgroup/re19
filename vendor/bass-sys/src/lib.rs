use bitflags::bitflags;
use std::os::raw::{c_double, c_int, c_void};

type BOOL = c_int;
type DWORD = u32;
type QWORD = u64;

pub type HSTREAM = DWORD;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(i32)]
pub enum BassErrorCode {
    Ok = 0,
    ErrorMem = 1,
    ErrorFileOpen = 2,
    ErrorDriver = 3,
    ErrorBufLost = 4,
    ErrorHandle = 5,
    ErrorFormat = 6,
    ErrorPosition = 7,
    ErrorInit = 8,
    ErrorStart = 9,
    ErrorSsl = 10,
    ErrorAlready = 14,
    ErrorNoChan = 18,
    ErrorIllType = 19,
    ErrorIllParam = 20,
    ErrorNo3D = 21,
    ErrorNoEAX = 22,
    ErrorDevice = 23,
    ErrorNoPlay = 24,
    ErrorFreq = 25,
    ErrorNotFile = 27,
    ErrorNoHW = 29,
    ErrorEmpty = 31,
    ErrorNoNet = 32,
    ErrorCreate = 33,
    ErrorNoFX = 34,
    ErrorNotAvail = 37,
    ErrorDecode = 38,
    ErrorDX = 39,
    ErrorTimeout = 40,
    ErrorFileForm = 41,
    ErrorSpeaker = 42,
    ErrorVersion = 43,
    ErrorCodec = 44,
    ErrorEnded = 45,
    ErrorBusy = 46,
    ErrorUnknown = -1,
}

bitflags!(
    #[repr(C)]
    pub struct BassInitFlags: DWORD {
        const DEVICE_8BITS =      1;
        const DEVICE_MONO =       2;
        const DEVICE_3D =         4;
        const DEVICE_16BITS =     8;
        const DEVICE_LATENCY =    0x100;
        const DEVICE_CPSPEAKERS = 0x400;
        const DEVICE_SPEAKERS =   0x800;
        const DEVICE_NOSPEAKER =  0x1000;
        const DEVICE_DMIX =       0x2000;
        const DEVICE_FREQ =       0x4000;
        const DEVICE_STEREO =     0x8000;
        const DEVICE_HOG =        0x10000;
        const DEVICE_AUDIOTRACK = 0x20000;
        const DEVICE_DSOUND =     0x40000;
    }
);

bitflags!(
    #[repr(C)]
    pub struct BassStreamFlags: DWORD {
        const PRESCAN  = 0x20000;
        const AUTOFREE = 0x40000;
        const RESTRATE = 0x80000;
        const BLOCK    = 0x100000;
        const DECODE   = 0x200000;
        const STATUS   = 0x800000;
    }
);

bitflags!(
    #[repr(C)]
    pub struct BassPosMode: DWORD {
        const BYTE =        0;
        const MUSIC_ORDER = 1;
        const OGG =         3;
        const RESET =       0x2000000;
        const RELATIVE =    0x4000000;
        const INEXACT =     0x8000000;
        const DECODE =      0x10000000;
        const DECODETO =    0x20000000;
        const SCAN =        0x40000000;
    }
);

#[repr(u32)]
pub enum BassActiveState {
    Stopped = 0,
    Playing = 1,
    Stalled = 2,
    Paused = 3,
    PausedDevice = 4,
}

#[repr(u32)]
pub enum BassConfigOption {
    Buffer = 0,
    UpdatePeriod = 1,
    GVolSample = 4,
    GVolStream = 5,
    GVolMusic = 6,
    CurveVol = 7,
    CurvePan = 8,
    FloatDSP = 9,
    Algorithm3D = 10,
    NetTimeout = 11,
    NetBuffer = 12,
    PauseNoPlay = 13,
    NetPrebuf = 14,
    NetPassive = 18,
    RecBuffer = 19,
    NetPlaylist = 21,
    MusicVirtual = 22,
    Verify = 23,
    UpdateThreads = 24,
    DevBuffer = 27,
    VistaTruePos = 30,
    IOSMixAudio = 34,
    DevDefault = 36,
    NetReadTimeout = 37,
    VistaSpeakers = 38,
    IOsSpeakers = 39,
    MFDisable = 40,
    Handles = 41,
    Unicode = 42,
    Src = 43,
    SrcSample = 44,
    AsyncFileBuffer = 45,
    OggPrescan = 47,
    MfVideo = 48,
    Airplay = 49,
    DevNonStop = 50,
    IOsNoCategory = 51,
    VerifyNet = 52,
    DevPeriod = 53,
    Float = 54,
    NetSeek = 56,
}

extern "system" {
    pub fn BASS_Init(
        device: c_int,
        freq: DWORD,
        flags: BassInitFlags,
        win: *mut c_void,
        ds_guid: *mut c_void,
    ) -> BOOL;
    pub fn BASS_Start() -> BOOL;
    pub fn BASS_Update(length: DWORD) -> BOOL;
    pub fn BASS_StreamCreateFile(
        mem: BOOL,
        file: *const c_void,
        offset: QWORD,
        length: QWORD,
        flags: BassStreamFlags,
    ) -> HSTREAM;
    pub fn BASS_StreamFree(handle: HSTREAM) -> BOOL;
    pub fn BASS_ChannelUpdate(handle: DWORD, length: DWORD) -> BOOL;
    pub fn BASS_ChannelGetPosition(handle: DWORD, mode: BassPosMode) -> QWORD;
    pub fn BASS_ChannelBytes2Seconds(handle: DWORD, pos: QWORD) -> c_double;
    pub fn BASS_ChannelPlay(handle: DWORD, restart: BOOL) -> BOOL;
    pub fn BASS_ChannelPause(handle: DWORD) -> BOOL;
    pub fn BASS_ChannelSeconds2Bytes(handle: DWORD, pos: c_double) -> QWORD;
    pub fn BASS_ChannelSetPosition(handle: DWORD, pos: QWORD, mode: BassPosMode) -> BOOL;
    pub fn BASS_ChannelIsActive(handle: DWORD) -> BassActiveState;
    pub fn BASS_ErrorGetCode() -> BassErrorCode;
    pub fn BASS_SetConfig(option: BassConfigOption, value: DWORD) -> BOOL;
}
