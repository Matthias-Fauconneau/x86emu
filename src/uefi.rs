//#![feature(abi_efiapi)]
use {std::{ptr::{null, null_mut}, ffi::c_void}, uefi::{Status, data_types::{Guid, Handle, chars::Char16, Event},
                proto::{console::text::{RawKey, OutputData, Input, Output}, loaded_image::LoadedImage, media::{fs::SimpleFileSystem, file::{FileImpl, FileMode, FileAttribute}}},
                table::{Header,Revision, runtime::{Time, TimeCapabilities, ResetType, RuntimeServices},
                    boot::{Tpl, MemoryDescriptor, MemoryType, MemoryMapKey, EventType, EventNotifyFn, BootServices},
                    SystemTableImpl as SystemTable},
        } };

pub fn new_input() -> Input { Input{
    reset: {extern "efiapi" fn i(_this: &mut Input, _extended: bool) -> Status { unimplemented!() } i},
    read_key_stroke: {extern "efiapi" fn i(_this: &mut Input, _key: *mut RawKey) -> Status { unimplemented!() } i},
    wait_for_key: Event(null_mut())
} }

pub fn new_output_data() -> OutputData { OutputData{max_mode: 0, mode: -1, attribute: 0, cursor_column: 0, cursor_row: 0, cursor_visible: false} }

pub fn new_output(output_data : &OutputData) -> Output { Output{
    reset: {extern "efiapi" fn i(_this: &Output, _extended: bool) -> Status { unimplemented!() } i},
    output_string: {extern "efiapi" fn i(_this: &Output, _string: *const Char16) -> Status { unimplemented!() } i},
    test_string: {extern "efiapi" fn i(_this: &Output, _string: *const Char16) -> Status { unimplemented!() } i},
    query_mode: {extern "efiapi" fn i(_this: &Output, _mode: usize, _columns: &mut usize, _rows: &mut usize) -> Status { unimplemented!() } i},
    set_mode: {extern "efiapi" fn i(_this: &mut Output, _mode: usize) -> Status { unimplemented!() } i},
    set_attribute: {extern "efiapi" fn i(_this: &mut Output, _attribute: usize) -> Status { unimplemented!() } i},
    clear_screen: {extern "efiapi" fn i(_this: &mut Output) -> Status { unimplemented!() } i},
    set_cursor_position: {extern "efiapi" fn i(_this: &mut Output, _column: usize, _row: usize) -> Status { unimplemented!() } i},
    enable_cursor: {extern "efiapi" fn i(_this: &mut Output, _visible: bool) -> Status { unimplemented!() } i},
    data: &output_data,
} }

pub fn new_runtime_services() -> RuntimeServices { RuntimeServices{ header: Header{ signature:0, revision:Revision(0), size:0, crc:0, _reserved:0 },
    get_time: {extern "efiapi" fn i(_time: *mut Time, _capabilities: *mut TimeCapabilities) -> Status { unimplemented!() } i},
    set_time: {extern "efiapi" fn i(_time: &Time) -> Status { unimplemented!() } i},
    _pad: [0; 2],
    set_virtual_address_map: {extern "efiapi" fn i(_map_size: usize, _desc_size: usize, _desc_version: u32, _virtual_map: *mut MemoryDescriptor) -> Status { unimplemented!() } i},
    _pad2: [0; 5],
    reset: {extern "efiapi" fn i(_rt: ResetType, _status: Status, _data_size: usize, _data: *const u8) -> ! { unimplemented!() } i}
}}

pub fn new_boot_services() -> BootServices { BootServices{ header : Header{ signature:0, revision:Revision(0), size:0, crc:0, _reserved:0 },
    raise_tpl: {extern "efiapi" fn i(_new_tpl: Tpl) -> Tpl { unimplemented!(); } i},
    restore_tpl: {extern "efiapi" fn i(_old_tpl: Tpl) { unimplemented!(); } i},
    allocate_pages: {extern "efiapi" fn i(_alloc_ty: u32, _mem_ty: MemoryType, _count: usize, _addr: &mut u64) -> Status { unimplemented!(); } i},
    free_pages: {extern "efiapi" fn i(_addr: u64, _pages: usize) -> Status { unimplemented!(); } i},
    get_memory_map:
        {extern "efiapi" fn i(_size: &mut usize, _map: *mut MemoryDescriptor, _key: &mut MemoryMapKey, _desc_size: &mut usize, _desc_version: &mut u32)
            -> Status { unimplemented!(); } i},
    allocate_pool: {extern "efiapi" fn i(_pool_type: MemoryType, _size: usize, _buffer: &mut *mut u8) -> Status { unimplemented!(); } i},
    free_pool: {extern "efiapi" fn i(_buffer: *mut u8) -> Status { unimplemented!(); } i},
    create_event: {extern "efiapi" fn i(_ty: EventType, _notify_tpl: Tpl, _notify_func: Option<EventNotifyFn>, _notify_ctx: *mut c_void, _event: *mut Event) -> Status
        { unimplemented!(); } i},
    set_timer: {extern "efiapi" fn i(_event: Event, _ty: u32, _trigger_time: u64) -> Status { unimplemented!() } i},
    wait_for_event: {extern "efiapi" fn i(_number_of_events: usize, _events: *mut Event, _out_index: *mut usize) -> Status { unimplemented!(); } i},
    signal_event: 0,
    close_event: 0,
    check_event: 0,
    install_protocol_interface: 0,
    reinstall_protocol_interface: 0,
    uninstall_protocol_interface: 0,
    handle_protocol: {extern "efiapi" fn i(_handle: Handle, _proto: &Guid, _out_proto: &mut *mut c_void) -> Status { unimplemented!(); } i},
    _reserved: 0,
    register_protocol_notify: 0,
    locate_handle: {extern "efiapi" fn i(_search_ty: i32, _proto: *const Guid, _key: *mut c_void, _buf_sz: &mut usize, _buf: *mut Handle) -> Status
        { unimplemented!(); } i},
    locate_device_path: 0,
    install_configuration_table: 0,
    load_image: 0,
    start_image: 0,
    exit: 0,
    unload_image: 0,
    exit_boot_services: {extern "efiapi" fn i(_image_handle: Handle, _map_key: MemoryMapKey) -> Status { unimplemented!(); } i},
    get_next_monotonic_count: 0,
    set_watchdog_timer: {extern "efiapi" fn i(_timeout: usize, _watchdog_code: u64, _data_size: usize, _watchdog_data: *const u16) -> Status { unimplemented!(); } i},
    stall: {extern "efiapi" fn i(_microseconds: usize) -> Status { unimplemented!(); } i},
    connect_controller: 0,
    disconnect_controller: 0,
    open_protocol: 0,
    close_protocol: 0,
    open_protocol_information: 0,
    protocols_per_handle: 0,
    locate_handle_buffer: 0,
    locate_protocol: {extern "efiapi" fn i(_proto: &Guid, _registration: *mut c_void, _out_proto: &mut *mut c_void) -> Status { unimplemented!(); } i},
    install_multiple_protocol_interfaces: 0,
    uninstall_multiple_protocol_interfaces: 0,
    calculate_crc32: 0,
    copy_mem: {extern "efiapi" fn i(_dest: *mut u8, _src: *const u8, _len: usize) { unimplemented!(); } i},
    set_mem: {extern "efiapi" fn i(_buffer: *mut u8, _len: usize, _value: u8) { unimplemented!(); } i},
    create_event_ex: 0,
} }

pub fn new_system_table<'boot, 'runtime>(stdin: &'boot Input, stdout: &'boot Output<'boot>, stderr: &'boot Output<'boot>,
    runtime: &'runtime RuntimeServices, boot:&'boot BootServices) -> SystemTable<'boot, 'runtime> { SystemTable{
        header:Header{signature:0,revision:Revision(0),size:0,crc:0,_reserved:0}, fw_vendor:null(), fw_revision:Revision(0),
        stdin_handle:Handle(null_mut()), stdin: &stdin,
        stdout_handle:Handle(null_mut()), stdout: &stdout,
        stderr_handle:Handle(null_mut()), stderr: &stderr,
        runtime: &runtime, boot: &boot, nr_cfg: 0, cfg_table:null()
} }

pub fn new_loaded_image(load_options: &[u16]) -> LoadedImage { LoadedImage{
    revision: 0, parent_handle: Handle(null_mut()), system_table: null(), device_handle: Handle(null_mut()),
    _file_path: null(), _reserved: null(),
    load_options_size: 0, load_options: load_options.as_ptr() as *const Char16,
    image_base: 0, image_size: 0,
    image_code_type: MemoryType::RESERVED,
    image_data_type: MemoryType::RESERVED,
    unload: {extern "efiapi" fn i(_image_handle: Handle) -> Status { unimplemented!() } i},
} }

pub fn new_simple_file_system() -> SimpleFileSystem { SimpleFileSystem {
    revision: 0, open_volume: {extern "efiapi" fn f(_this: &mut SimpleFileSystem, _root: &mut *mut FileImpl) -> uefi::Status { unimplemented!() } f}
} }

pub fn new_file_impl() -> FileImpl { FileImpl {
    revision: 0,
    open: {extern "efiapi" fn f(_this: &mut FileImpl, _new_handle: &mut *mut FileImpl, _filename: *const Char16, _open_mode: FileMode, _attributes: FileAttribute) -> Status {
        unimplemented!() } f},
    close: {extern "efiapi" fn f(_this: &mut FileImpl) -> Status {unimplemented!() } f},
    delete: {extern "efiapi" fn f(this: &mut FileImpl) -> Status {unimplemented!() } f},
    read: {extern "efiapi" fn f(this: &mut FileImpl, buffer_size: &mut usize, buffer: *mut u8) -> Status { unimplemented!() } f},
    write: {extern "efiapi" fn f(this: &mut FileImpl,buffer_size: &mut usize,buffer: *const u8) -> Status { unimplemented!() } f},
    get_position: {extern "efiapi" fn f(this: &mut FileImpl, position: &mut u64) -> Status { unimplemented!() } f},
    set_position: {extern "efiapi" fn f(this: &mut FileImpl, position: u64) -> Status { unimplemented!() } f},
    get_info: {extern "efiapi" fn f(this: &mut FileImpl,information_type: &Guid,buffer_size: &mut usize,buffer: *mut u8) -> Status { unimplemented!() } f},
    set_info: {extern "efiapi" fn f(this: &mut FileImpl,information_type: &Guid,buffer_size: usize,buffer: *const c_void) -> Status { unimplemented!() } f},
    flush: {extern "efiapi" fn f(this: &mut FileImpl) -> Status { unimplemented!() } f},
} }
