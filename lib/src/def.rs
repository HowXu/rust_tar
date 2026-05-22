pub(super) type Byte = u8;
pub(super) type IOError = std::io::Error;
pub(super) type FMTError = std::fmt::Error;

#[repr(C)] // C 不对齐方式
pub(super) struct CommonHeader{
    path: [Byte;100],
    mode: [Byte;8],
    owner_user: [Byte;8], // octal
    group_user: [Byte;8], // octal
    file_size: [Byte;12], // octal
    modification_time: [Byte;12], // octal unix time format
    checksum: [Byte;8],
    type_flag: Byte,
    type_info: [Byte;100],
    ustar_indicator: [Byte;6], // always be "ustar"
    ustar_version: [Byte;2], // always be 00
    owner_user_name: [Byte;32],
    owner_group_name: [Byte;32],
    device_major: [Byte;8],
    device_minor: [Byte;8],
    filename_prefix: [Byte;155], // what's this ?
    paddings:[Byte;12]
}

#[test]
fn test_header_size(){
    assert_eq!(size_of::<CommonHeader>(),512); // a header must be 512 size or a chunk
}