use std::ffi::c_void;
use std::mem::{size_of, zeroed};
use std::ptr::{addr_of, copy_nonoverlapping, null_mut};
use windows_sys::Win32::System::Memory::{MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE, VirtualAlloc, VirtualProtect};
use windows_sys::Win32::System::SystemInformation::{GetSystemInfo, SYSTEM_INFO};

unsafe fn allocate_page_near_address(target: *const c_void) -> *mut c_void {
    let mut sys_info: SYSTEM_INFO = zeroed();
    GetSystemInfo(&mut sys_info);
    let page_size = sys_info.dwPageSize as u64;

    let start_addr = (target as u64) & !(page_size - 1);
    let min_addr = u64::min(start_addr - 0x7FFFFF00, sys_info.lpMinimumApplicationAddress as u64);
    let max_addr = u64::max(start_addr + 0x7FFFFF00, sys_info.lpMaximumApplicationAddress as u64);

    let start_page = start_addr - (start_addr % page_size);

    let mut page_offset = 1;
    loop {
        let byte_offset = page_offset * page_size;
        let high_addr = start_page + byte_offset;
        let low_addr = match start_page > byte_offset {
            true => start_page - byte_offset,
            false => 0
        };
        let needs_exit = high_addr > max_addr && low_addr < min_addr;

        if high_addr < max_addr {
            let out_addr = VirtualAlloc(
                high_addr as *const c_void,
                page_size as usize,
                MEM_COMMIT | MEM_RESERVE,
                PAGE_EXECUTE_READWRITE
            );
            if !out_addr.is_null() {
                return out_addr;
            }
        }
        if low_addr > min_addr {
            let out_addr = VirtualAlloc(
                low_addr as *const c_void,
                page_size as usize,
                MEM_COMMIT | MEM_RESERVE,
                PAGE_EXECUTE_READWRITE
            );
            if !out_addr.is_null() {
                return out_addr;
            }
        }
        page_offset += 1;
        if needs_exit {
            break;
        }
    }
    null_mut()
}

#[cfg(target_pointer_width = "64")]
unsafe fn write_abs_jump(jump_mem: *mut c_void, target: *const c_void) {
    let mut jump_instructions= [
        0x49, 0xBA, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //mov r10, addr
        0x41, 0xFF, 0xE2 //jmp r10
    ];
    jump_instructions[2..10].copy_from_slice(&(target as u64).to_ne_bytes());
    copy_nonoverlapping(jump_instructions.as_ptr() as *const c_void, jump_mem, jump_instructions.len());
}

fn ptr_addrs(ptr: *const c_void) -> isize {
    (ptr as *const isize) as isize
}

pub unsafe fn install_hook(src: *const c_void, dst: *const c_void) {
    let fun_mem = allocate_page_near_address(src);
    assert!(!fun_mem.is_null());
    write_abs_jump(fun_mem, dst);

    let mut jump_instructions = [0xE9, 0x0, 0x0, 0x0, 0x0];
    let offset = (ptr_addrs(fun_mem) - (ptr_addrs(src) + jump_instructions.len() as isize)) as u32;
    jump_instructions[1..].copy_from_slice(&offset.to_ne_bytes());

    let mut protection = 0;
    VirtualProtect(
        src,
        jump_instructions.len(),
        PAGE_EXECUTE_READWRITE,
        &mut protection
    );
    let src = src as *mut c_void;

    copy_nonoverlapping(jump_instructions.as_ptr() as *const c_void, src, jump_instructions.len());

    VirtualProtect(
        src,
        jump_instructions.len(),
        protection,
        &mut protection
    );

}