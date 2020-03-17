//#![feature(abi_efiapi)]
use {std::{ptr::{null, null_mut}, ffi::c_void}, uefi::{Status, data_types::{Guid, Handle, chars::Char16, Event},
                table::{Header,Revision, runtime::{RuntimeServices, Time, TimeCapabilities, ResetType},
                    boot::{BootServices,Tpl, MemoryDescriptor, MemoryType, MemoryMapKey, EventType, EventNotifyFn},
                    SystemTableImpl as SystemTable},
                proto::console::text::{self, Input, RawKey, Output, OutputData}}};

static mut stdin : text::Input = text::Input{
    reset: {extern "efiapi" fn i(this: &mut Input, extended: bool) -> Status { unimplemented!() } i},
    read_key_stroke: {extern "efiapi" fn i(this: &mut Input, key: *mut RawKey) -> Status { unimplemented!() } i},
    wait_for_key: Event(null_mut())
};

static static_output_data : OutputData = OutputData{max_mode: 0, mode: -1, attribute: 0, cursor_column: 0, cursor_row: 0, cursor_visible: false};

const fn default_output(output_data : &OutputData) -> Output { Output{
    reset: {extern "efiapi" fn i(this: &Output, extended: bool) -> Status { unimplemented!() } i},
    output_string: {unsafe extern "efiapi" fn i(this: &Output, string: *const Char16) -> Status { unimplemented!() } i},
    test_string: {unsafe extern "efiapi" fn i(this: &Output, string: *const Char16) -> Status { unimplemented!() } i},
    query_mode: {extern "efiapi" fn i(this: &Output, mode: usize, columns: &mut usize, rows: &mut usize) -> Status { unimplemented!() } i},
    set_mode: {extern "efiapi" fn i(this: &mut Output, mode: usize) -> Status { unimplemented!() } i},
    set_attribute: {extern "efiapi" fn i(this: &mut Output, attribute: usize) -> Status { unimplemented!() } i},
    clear_screen: {extern "efiapi" fn i(this: &mut Output) -> Status { unimplemented!() } i},
    set_cursor_position: {extern "efiapi" fn i(this: &mut Output, column: usize, row: usize) -> Status { unimplemented!() } i},
    enable_cursor: {extern "efiapi" fn i(this: &mut Output, visible: bool) -> Status { unimplemented!() } i},
    data: &output_data,
} }

static mut stdout : text::Output = default_output(&static_output_data);
static mut stderr : text::Output = default_output(&static_output_data);
static runtime : RuntimeServices = RuntimeServices{ header: Header{ signature:0, revision:Revision(0), size:0, crc:0, _reserved:0 },
    get_time: {unsafe extern "efiapi" fn i(time: *mut Time, capabilities: *mut TimeCapabilities) -> Status { unimplemented!() } i},
    set_time: {unsafe extern "efiapi" fn i(time: &Time) -> Status { unimplemented!() } i},
    _pad: [0; 2],
    set_virtual_address_map: {unsafe extern "efiapi" fn i(map_size: usize, desc_size: usize, desc_version: u32, virtual_map: *mut MemoryDescriptor) -> Status { unimplemented!() } i},
    _pad2: [0; 5],
    reset: {unsafe extern "efiapi" fn i(rt: ResetType, status: Status, data_size: usize, data: *const u8) -> ! { unimplemented!() } i}
};
static boot : BootServices = BootServices{ header : Header{ signature:0, revision:Revision(0), size:0, crc:0, _reserved:0 },
    raise_tpl: {unsafe extern "efiapi" fn i(new_tpl: Tpl) -> Tpl { unimplemented!(); } i},
    restore_tpl: {unsafe extern "efiapi" fn i(old_tpl: Tpl) { unimplemented!(); } i},
    allocate_pages: {extern "efiapi" fn i(alloc_ty: u32, mem_ty: MemoryType, count: usize, addr: &mut u64) -> Status { unimplemented!(); } i},
    free_pages: {extern "efiapi" fn i(addr: u64, pages: usize) -> Status { unimplemented!(); } i},
    get_memory_map:
        {unsafe extern "efiapi" fn i(size: &mut usize, map: *mut MemoryDescriptor, key: &mut MemoryMapKey, desc_size: &mut usize, desc_version: &mut u32) -> Status
            { unimplemented!(); } i},
    allocate_pool: {extern "efiapi" fn i(pool_type: MemoryType, size: usize, buffer: &mut *mut u8) -> Status { unimplemented!(); } i},
    free_pool: {extern "efiapi" fn i(buffer: *mut u8) -> Status { unimplemented!(); } i},
    create_event: {unsafe extern "efiapi" fn i(ty: EventType, notify_tpl: Tpl, notify_func: Option<EventNotifyFn>, notify_ctx: *mut c_void, event: *mut Event) -> Status
        { unimplemented!(); } i},
    set_timer: {unsafe extern "efiapi" fn i(event: Event, ty: u32, trigger_time: u64) -> Status { unimplemented!() } i},
    wait_for_event: {unsafe extern "efiapi" fn i(number_of_events: usize, events: *mut Event, out_index: *mut usize) -> Status { unimplemented!(); } i},
    signal_event: 0,
    close_event: 0,
    check_event: 0,
    install_protocol_interface: 0,
    reinstall_protocol_interface: 0,
    uninstall_protocol_interface: 0,
    handle_protocol: {extern "efiapi" fn i(handle: Handle, proto: &Guid, out_proto: &mut *mut c_void) -> Status { unimplemented!(); } i},
    _reserved: 0,
    register_protocol_notify: 0,
    locate_handle: {unsafe extern "efiapi" fn i(search_ty: i32, proto: *const Guid, key: *mut c_void, buf_sz: &mut usize, buf: *mut Handle) -> Status { unimplemented!(); } i},
    locate_device_path: 0,
    install_configuration_table: 0,
    load_image: 0,
    start_image: 0,
    exit: 0,
    unload_image: 0,
    exit_boot_services: {unsafe extern "efiapi" fn i(image_handle: Handle, map_key: MemoryMapKey) -> Status { unimplemented!(); } i},
    get_next_monotonic_count: 0,
    set_watchdog_timer: {unsafe extern "efiapi" fn i(timeout: usize, watchdog_code: u64, data_size: usize, watchdog_data: *const u16) -> Status { unimplemented!(); } i},
    stall: {extern "efiapi" fn i(microseconds: usize) -> Status { unimplemented!(); } i},
    connect_controller: 0,
    disconnect_controller: 0,
    open_protocol: 0,
    close_protocol: 0,
    open_protocol_information: 0,
    protocols_per_handle: 0,
    locate_handle_buffer: 0,
    locate_protocol: {extern "efiapi" fn i(proto: &Guid, registration: *mut c_void, out_proto: &mut *mut c_void) -> Status { unimplemented!(); } i},
    install_multiple_protocol_interfaces: 0,
    uninstall_multiple_protocol_interfaces: 0,
    calculate_crc32: 0,
    copy_mem: {unsafe extern "efiapi" fn i(dest: *mut u8, src: *const u8, len: usize) { unimplemented!(); } i},
    set_mem: {unsafe extern "efiapi" fn i(buffer: *mut u8, len: usize, value: u8) { unimplemented!(); } i},
    create_event_ex: 0,
};

pub fn default_system_table() -> SystemTable {
        SystemTable{header:Header{signature:0,revision:Revision(0),size:0,crc:0,_reserved:0}, fw_vendor:null(), fw_revision:Revision(0),
            stdin_handle:Handle(null_mut()), stdin: unsafe{&mut stdin},
            stdout_handle:Handle(null_mut()), stdout: unsafe{&mut stdout},
            stderr_handle:Handle(null_mut()), stderr: unsafe{&mut stderr},
            runtime: &runtime, boot: &boot, nr_cfg: 0, cfg_table:null() }
}
