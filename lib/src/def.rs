pub(super) type Byte = u8;
pub(super) type IOError = std::io::Error;
pub(super) type FMTError = std::fmt::Error;

#[repr(C)] // C 不对齐方式
pub(super) struct CommonHeader {
    pub(super) path: [Byte; 100],
    pub(super) mode: [Byte; 8],
    pub(super) owner_user: [Byte; 8],         // octal
    pub(super) group_user: [Byte; 8],         // octal
    pub(super) file_size: [Byte; 12],         // octal
    pub(super) modification_time: [Byte; 12], // octal unix time format
    pub(super) checksum: [Byte; 8],
    pub(super) type_flag: Byte,
    pub(super) type_info: [Byte; 100],
    pub(super) ustar_indicator: [Byte; 6], // always be ustar\000 Hex is [75, 73, 74, 61, 72, 20]
    pub(super) ustar_version: [Byte; 2],   // always be 00\0 Hex is [20, 00]
    pub(super) owner_user_name: [Byte; 32],
    pub(super) owner_group_name: [Byte; 32],
    pub(super) device_major: [Byte; 8],
    pub(super) device_minor: [Byte; 8],
    pub(super) filename_prefix: [Byte; 155], // what's this ?
    pub(super) paddings: [Byte; 12],
}
impl CommonHeader {
    pub(super) fn new() -> Self {
        Self {
            path: [0u8; 100],
            mode: [0u8; 8],
            owner_user: [0u8; 8],
            group_user: [0u8; 8],
            file_size: [0u8; 12],
            modification_time: [0u8; 12],
            checksum: [0u8; 8],
            type_flag: 0,
            type_info: [0u8; 100],
            ustar_indicator: [0u8; 6],
            ustar_version: [0u8; 2],
            owner_user_name: [0u8; 32],
            owner_group_name: [0u8; 32],
            device_major: [0u8; 8],
            device_minor: [0u8; 8],
            filename_prefix: [0u8; 155],
            paddings: [0u8; 12],
        }
    }
}

#[test]
fn test_header_size() {
    assert_eq!(size_of::<CommonHeader>(), 512); // a header must be 512 size or a chunk
}
