use super::{AudioPlayer, ControllableAudioPlayer};
use bass_sys::{
    BASS_ChannelBytes2Seconds, BASS_ChannelGetPosition, BASS_ChannelPause, BASS_ChannelPlay,
    BASS_ChannelSeconds2Bytes, BASS_ChannelSetPosition, BASS_ChannelUpdate, BASS_ErrorGetCode,
    BASS_Init, BASS_SetConfig, BASS_Start, BASS_StreamCreateFile, BASS_StreamFree, BASS_Update,
    BassConfigOption, BassInitFlags, BassPosMode, BassStreamFlags, HSTREAM,
};
use std::ffi::CString;
use std::ptr;

#[allow(dead_code)]
pub struct BassPlayer {
    stream: HSTREAM,
}

#[allow(dead_code)]
impl BassPlayer {
    pub fn new(src: &str) -> Option<BassPlayer> {
        // todo: we only want to init once, need a guard here
        /*unsafe {
            BASS_SetConfig(BassConfigOption::UpdatePeriod, 0);
            BASS_SetConfig(BassConfigOption::UpdateThreads, 0);
        }*/
        unsafe {
            BASS_Init(
                -1,
                44100,
                BassInitFlags::empty(),
                ptr::null_mut(),
                ptr::null_mut(),
            );
        }

        let src_cstr = CString::new(src).unwrap();
        let stream = unsafe {
            BASS_StreamCreateFile(
                0,
                src_cstr.as_ptr() as *const _,
                0,
                0,
                BassStreamFlags::PRESCAN,
            )
        };

        assert_eq!(unsafe { BASS_Start() }, 1);

        if stream == 0 {
            println!("BASS error: {:?}", unsafe { BASS_ErrorGetCode() });
            None
        } else {
            Some(BassPlayer { stream })
        }
    }
}

impl AudioPlayer for BassPlayer {
    fn update(&self) {
        unsafe {
            BASS_ChannelUpdate(self.stream, 100);
        }
    }

    fn get_current_seconds(&self) -> f64 {
        unsafe {
            let pos = BASS_ChannelGetPosition(self.stream, BassPosMode::BYTE);
            BASS_ChannelBytes2Seconds(self.stream, pos)
        }
    }
}

impl ControllableAudioPlayer for BassPlayer {
    fn play(&mut self) {
        unsafe {
            unsafe { BASS_ChannelUpdate(self.stream, 100) };
            BASS_ChannelPlay(self.stream, 0);
        }
    }

    fn pause(&mut self) {
        unsafe {
            BASS_ChannelPause(self.stream);
        }
    }

    fn seek(&mut self, seconds: f64) {
        unsafe {
            let pos = BASS_ChannelSeconds2Bytes(self.stream, seconds);
            BASS_ChannelSetPosition(self.stream, pos, BassPosMode::BYTE);
        }
    }
}

impl Drop for BassPlayer {
    fn drop(&mut self) {
        unsafe {
            BASS_StreamFree(self.stream);
        }
    }
}
