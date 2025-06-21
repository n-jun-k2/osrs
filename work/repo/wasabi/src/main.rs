#![no_std]
#![no_main]
#![feature(offset_of)]

use core::mem::offset_of;
use core::mem::size_of;
use core::panic::PanicInfo;
use core::ptr::null_mut;
use core::slice;


type EfiVoid = u8;
type EfiHandle = u64;
type Result<T> = core::result::Result<T, &'static str>;


#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[must_use]
#[repr(u64)]
enum EfiStatus {
    Success = 0,
}

#[repr(C)]
struct EfiBootServicesTable {
    _reserved0: [u64; 40],
    locate_protocol: extern "win64" fn(
        protocol: *const EfiGuid,
        registration: *const EfiVoid,
        interface: *mut *mut EfiVoid,
    ) -> EfiStatus,
}
const _: () = assert!(offset_of!(EfiBootServicesTable, locate_protocol) == 320);


/// https://uefi.org/specs/UEFI/2.11/04_EFI_System_Table.html
#[repr(C)]
struct EfiSystemTable {
    _reserved0: [u64; 12],
    pub boot_services: &'static EfiBootServicesTable,
}
const _: () = assert!(offset_of!(EfiSystemTable, boot_services) == 96);

///EFI Graphics Output Protocolを示すGUIDの定義
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct EfiGuid {
    pub data0: u32,
    pub data1: u16,
    pub data2: u16,
    pub data3: [u8; 8],
}

// EFI_GRAPHICS_OUTPUT_MODE_INFORMATIONの構造体
#[repr(C)]
#[derive(Debug)]
struct EfiGraphicsOutputProtocolPixelInfo {
    version: u32,
    pub horizontalresolution: u32, // 水平方向の画素数
    pub verticalresolution: u32,   // 垂直方向の画素数　
    _padding0: [u32; 5],
    pub pixels_per_scan_line: u32, // 水平方向のデータに含まれる画素数
}
const _: () = assert!(size_of::<EfiGraphicsOutputProtocolPixelInfo>() == 36);


#[repr(C)]
#[derive(Debug)]
struct EfiGraphicsOutputProtocolMode<'a> {
    pub max_mode: u32,
    pub mode: u32,
    pub info: &'a EfiGraphicsOutputProtocolPixelInfo,
    pub size_of_info: u64,
    pub frame_buffer_base: usize,
    pub frame_buffer_size: usize,
}

#[repr(C)]
#[derive(Debug)]
struct EfiGraphicsOutputProtocol<'a> {
    reserved: [u64; 3],
    pub mode: &'a EfiGraphicsOutputProtocolMode<'a>,
}


// https://uefi.org/specs/UEFI/2.11/12_Protocols_Console_Support.html#efi-graphics-output-protocol
const EFI_GRAPHICS_OUTPUT_PROTOCOL_GUID : EfiGuid = EfiGuid {
    data0: 0x9042a9de,
    data1: 0x23dc,
    data2: 0x4a38,
    data3: [0x96,0xfb,0x7a,0xde,0xd0,0x80,0x51,0x6a],
};


/// EfiGraphicsOutputProtocolをUEFIから取得する関数
///
/// # Arguments
///
/// - `efi_system_table` (&EfiSystemTable): EFI System Table
///
/// # Returns
/// `Result<&'a EfiGraphicsOutputProtocol<'a>>`
fn locate_graphic_protocol<'a>(efi_system_table: &EfiSystemTable) -> Result<&'a EfiGraphicsOutputProtocol<'a>> {
    let mut graphic_output_protocol = null_mut::<EfiGraphicsOutputProtocol>();
    let status = (efi_system_table.boot_services.locate_protocol)(
        &EFI_GRAPHICS_OUTPUT_PROTOCOL_GUID,
        null_mut::<EfiVoid>(),
        &mut graphic_output_protocol 
            as *mut *mut EfiGraphicsOutputProtocol 
            as *mut *mut EfiVoid,
    );

    if status != EfiStatus::Success {
        return Err("Failed to locate graphics output protocol");
    }
    Ok(unsafe { &*graphic_output_protocol })
}


#[no_mangle]
fn efi_main(_img_handle: EfiHandle, efi_system_table: &EfiSystemTable) {
    let efi_graphics_output_protocol = locate_graphic_protocol(efi_system_table).unwrap();
    let vram_addr = efi_graphics_output_protocol.mode.frame_buffer_base;
    let vram_byte_size = efi_graphics_output_protocol.mode.frame_buffer_size;
    let vram = unsafe {
        slice::from_raw_parts_mut(
            vram_addr as *mut u32,
            vram_byte_size / size_of::<u32>()
        )
    };
    for e in vram {
        *e = 0xfff0ff;
    }
    loop {}
}


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
