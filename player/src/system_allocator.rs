use core::alloc::{GlobalAlloc, Layout};
use winapi::um::heapapi::{GetProcessHeap, HeapAlloc, HeapFree};

//pub static mut PROCESS_HEAP: HANDLE = ptr::null_mut();

pub struct SystemAllocator;

unsafe impl GlobalAlloc for SystemAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        HeapAlloc(GetProcessHeap(), 0, layout.size()) as *mut _
    }
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        HeapFree(GetProcessHeap(), 0, ptr as *mut _);
    }
}
