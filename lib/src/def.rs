use std::fs::Metadata;
use std::mem;
use std::time::UNIX_EPOCH;
use std::path::PathBuf;

pub(super) type Byte = u8;
pub(super) type IOError = std::io::Error;
pub(super) type FMTError = std::fmt::Error;

pub(super) const MAGIC_NUMBER: [Byte; 6] = [0x75, 0x73, 0x74, 0x61, 0x72, 0x20]; // "ustar\0"
pub(super) const MAGIC_VERSION: [Byte; 2] = [0x20, 0x00]; // " \0"

#[repr(C)] // C 不对齐方式
#[derive(Clone)]
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
    pub(super) ustar_indicator: [Byte; 6], // always be ustar空格 Hex is [75, 73, 74, 61, 72, 20]
    pub(super) ustar_version: [Byte; 2],   // always be 空格\0 Hex is [20, 00]
    pub(super) owner_user_name: [Byte; 32],
    pub(super) owner_group_name: [Byte; 32],
    pub(super) device_major: [Byte; 8],
    pub(super) device_minor: [Byte; 8],
    pub(super) filename_prefix: [Byte; 155], // what's this ?
    pub(super) paddings: [Byte; 12],
}

pub(super) struct UntarInstance {
    pub(super) path: Box<PathBuf>,
    pub(super) is_folder: bool,
    pub(super) file_size: u64,
    pub(super) seek_from: u64, // 512 * seek from
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

impl UntarInstance {
    pub(super) fn new(path: Box<PathBuf>, is_folder: bool, file_size: u64, seek_from: u64) -> Self {
        Self {
            path,
            is_folder,
            file_size,
            seek_from,
        }
    }
}

pub(super) struct EntarWrapper {
    pub(super) is_folder: bool,
    pub(super) instance: EntarInstance,
    pub(super) file_size: u64,
    pub(super) file_pointer: Box<PathBuf>,
}

struct EntarInstance {
    pub(super) path: [Byte; 100],
    pub(super) file_size: [Byte; 12],
    pub(super) modification_time: [Byte; 12],
    pub(super) mode: [Byte; 8],
    pub(super) checksum: [Byte; 8],
}

// it 绝对路径
// rel 相对路径 用来直接求字符串
// 涉及后续File IO问题，最好是有一层绝对路径方便处理
impl EntarWrapper {
    pub(super) fn new(it: PathBuf, rel: PathBuf, is_folder: bool, meta: Metadata) -> Self {
        // file size
        let file_size = meta.len();
        // file name and it's bytes
        let mut p: [Byte; 100] = [0u8; 100];
        if let Some(s) = rel.to_str() {
            // process the path to
            // win and linux we all need this
            let s_rp = s.replace('\\', "/");
            let src = s_rp.as_bytes();
            let len = s.len().min(100);
            p[..len].copy_from_slice(&src[..len]);
        } else {
            panic!("failed to parse PathBuf to_str");
        }

        // file size to bytes
        let file_size_to_bytes = format!("{:o}", file_size);
        let mut sz: [Byte; 12] = [0x30u8; 12];
        // process the path to
        let src = file_size_to_bytes.as_bytes();
        let len = file_size_to_bytes.len().min(11);
        sz[(11 - len)..11].copy_from_slice(&src[..len]);
        sz[11] = 0x00;

        // modification time
        let mut mdf_time: [Byte; 12] = [0u8; 12];
        if let Ok(mdf) = meta.modified() {
            if let Ok(unix_timestamp) = mdf.duration_since(UNIX_EPOCH) {
                let bytes = format!("{:o}", unix_timestamp.as_secs());
                let bits = bytes.as_bytes();
                mdf_time[..11].copy_from_slice(&bits);
            } else {
                panic!("failed to get metadata unix timestamp");
            }
        } else {
            panic!("failed to get metadata modified time");
        }

        let mode: [Byte; 8];
        // permissions
        if meta.permissions().readonly() {
            // 只读
            mode = [0x30, 0x30, 0x30, 0x30, 0x34, 0x34, 0x34, 0x00];
        } else {
            // 统一 777
            mode = [0x30, 0x30, 0x30, 0x30, 0x37, 0x37, 0x37, 0x00];
        }

        // do not modify checksum, it's work of transformer
        Self {
            is_folder,
            instance: EntarInstance {
                path: p,
                file_size: sz,
                modification_time: mdf_time,
                mode,
                checksum: [0u8; 8],
            },
            file_size,
            file_pointer: Box::new(it),
        }
    }

    pub(super) fn as_header(self: Self) -> CommonHeader {
        let mut header = CommonHeader::new();
        // caculate checksum
        header.ustar_version = MAGIC_VERSION;
        header.ustar_indicator = MAGIC_NUMBER;

        header.file_size = self.instance.file_size;
        header.mode = self.instance.mode;
        header.modification_time = self.instance.modification_time;
        header.path = self.instance.path;
        header.checksum = [0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20];
        header.owner_user = [0x30; 8];
        header.owner_user[7] = 0x00;
        header.group_user = [0x30; 8];
        header.group_user[7] = 0x00;
        if self.is_folder {
            header.type_flag = 0x35;
        } else {
            header.type_flag = 0x30;
        }
        // caculate
        let buffer: [Byte; 512] = unsafe { mem::transmute(header.clone()) };
        let mut chechsums = 0u64;
        for byte in buffer {
            chechsums += byte as u64;
        }
        // u64 to octal
        let checksum_s = format!("{:o}", chechsums);
        let checksum = checksum_s.as_bytes();
        let mut cs: [Byte; 8] = [0x30u8; 8];
        let len = checksum.len().min(6);

        cs[(6 - len)..6].copy_from_slice(&checksum);
        // 0 0 0 0 0 0 0 0
        // 6 - 5= 1
        cs[7] = 0x20;
        cs[6] = 0x00;
        header.checksum = cs;

        return header;
    }
}

#[test]
fn test_header_size() {
    assert_eq!(size_of::<CommonHeader>(), 512); // a header must be 512 size or a chunk
}
