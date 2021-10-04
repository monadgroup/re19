use alloc::vec::Vec;
use core::{mem, ptr};
use engine::cstr;
use winapi::ctypes::{c_char, c_int};
use winapi::shared::basetsd::INT_PTR;
use winapi::shared::minwindef::{LPARAM, UINT, WPARAM};
use winapi::shared::windef::HWND;
use winapi::um::libloaderapi::GetModuleHandleA;
use winapi::um::wingdi::DEVMODEA;
use winapi::um::winuser::{
    DialogBoxParamA, EndDialog, EnumDisplaySettingsA, GetDlgItem, IsDlgButtonChecked, LoadIconA,
    PostMessageA, SendDlgItemMessageA, SendMessageA, SetWindowTextA, BM_SETCHECK, BST_CHECKED,
    CB_ADDSTRING, CB_GETCURSEL, CB_SETCURSEL, HTCAPTION, ICON_BIG, MAKEINTRESOURCEA, WM_CLOSE,
    WM_COMMAND, WM_INITDIALOG, WM_LBUTTONDOWN, WM_NCLBUTTONDOWN, WM_SETICON,
};

const IDD_INIT: u32 = 101;
const IDI_ICON1: u32 = 103;
const IDB_EXIT: u32 = 40003;
const ID_FS: u32 = 40004;
const ID_PR: u32 = 40000;
const ID_RESOLUTION: u32 = 40005;
const IDB_START: u32 = 40006;

extern "C" {
    fn sprintf_s(s: *mut c_char, size_of_buffer: usize, format: *const c_char, ...) -> c_int;
}

#[derive(Clone, Copy)]
pub struct Config {
    pub display_width: u32,
    pub display_height: u32,
    pub is_fullscreen: bool,
    pub is_prerender: bool,
}

static mut AVAILABLE_RESOLUTIONS: Vec<(u32, u32)> = Vec::new();
static mut CONFIG: Config = Config {
    display_width: 0,
    display_height: 0,
    is_fullscreen: false,
    is_prerender: false,
};

unsafe extern "system" fn dlg_proc(
    h_dlg: HWND,
    u_msg: UINT,
    w_param: WPARAM,
    _l_param: LPARAM,
) -> INT_PTR {
    match u_msg {
        WM_INITDIALOG => {
            // set icon
            let h_icon = LoadIconA(
                GetModuleHandleA(ptr::null_mut()),
                MAKEINTRESOURCEA(IDI_ICON1 as _),
            );
            SendMessageA(h_dlg, WM_SETICON, ICON_BIG as WPARAM, h_icon as LPARAM);

            // check "fullscreen"
            PostMessageA(GetDlgItem(h_dlg, ID_FS as _), BM_SETCHECK, BST_CHECKED, 0);

            // populate combobox
            let mut select_index = AVAILABLE_RESOLUTIONS.len() - 1;
            for (res_index, &(res_width, res_height)) in AVAILABLE_RESOLUTIONS.iter().enumerate() {
                let mut buff: [c_char; 20] = mem::uninitialized();
                sprintf_s(&mut buff[0], 20, cstr!("%ix%i"), res_width, res_height);
                SendDlgItemMessageA(
                    h_dlg,
                    40005,
                    CB_ADDSTRING,
                    0,
                    &buff[0] as *const c_char as _,
                );

                if res_width == 1920 && res_height == 1080 {
                    select_index = res_index;
                }
            }

            SendDlgItemMessageA(h_dlg, ID_RESOLUTION as _, CB_SETCURSEL, select_index, 0);

            SetWindowTextA(h_dlg, cstr!("You lost the game"));
        }
        WM_CLOSE => {
            EndDialog(h_dlg, 0);
        }
        WM_COMMAND => {
            if w_param == IDB_START as usize {
                let selected_res =
                    SendDlgItemMessageA(h_dlg, ID_RESOLUTION as _, CB_GETCURSEL, 0, 0) as usize;
                let (selected_width, selected_height) = AVAILABLE_RESOLUTIONS[selected_res];
                CONFIG.display_width = selected_width;
                CONFIG.display_height = selected_height;
                CONFIG.is_fullscreen = IsDlgButtonChecked(h_dlg, ID_FS as _) != 0;
                CONFIG.is_prerender = IsDlgButtonChecked(h_dlg, ID_PR as _) != 0;

                EndDialog(h_dlg, 1);
            } else if w_param == IDB_EXIT as usize {
                EndDialog(h_dlg, 0);
            }
        }
        WM_LBUTTONDOWN => {
            return SendMessageA(h_dlg, WM_NCLBUTTONDOWN, HTCAPTION as _, 0);
        }
        _ => {
            return 0;
        }
    };

    1
}

pub fn show() -> Option<Config> {
    // Populate the available resolution list
    unsafe {
        AVAILABLE_RESOLUTIONS.clear();

        let mut dm: DEVMODEA = mem::uninitialized();
        let mut mode_num = 0;
        let mut last_width = 0;
        let mut last_height = 0;

        while EnumDisplaySettingsA(ptr::null_mut(), mode_num, &mut dm) != 0 {
            mode_num += 1;
            if (dm.dmPelsWidth == last_width && dm.dmPelsHeight == last_height)
                || dm.dmBitsPerPel != 32
            {
                continue;
            }

            last_width = dm.dmPelsWidth;
            last_height = dm.dmPelsHeight;

            AVAILABLE_RESOLUTIONS.push((last_width, last_height));
        }
    }

    let exit_button = unsafe {
        DialogBoxParamA(
            GetModuleHandleA(ptr::null_mut()),
            MAKEINTRESOURCEA(IDD_INIT as _),
            ptr::null_mut(),
            Some(dlg_proc),
            0,
        )
    };

    match exit_button {
        1 => Some(unsafe { CONFIG }),
        _ => None,
    }
}
