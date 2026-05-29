use crate::def::*;
use std::{
    collections::{LinkedList, VecDeque},
    fs::{self, File, OpenOptions},
    io::{ErrorKind, Read, Seek, SeekFrom, Write},
    mem,
    path::{Path, PathBuf},
};

/**
 *  still there's no judge
 *  the parent of output_file must be exist and confirmed by user
 *  input path must be a folder and confirmed by user
 */
pub fn file_entar<'f>(input_path: &'f Path, output_file: &'f Path) -> Result<(), IOError> {
    // first get the information we need and push to a stack
    // e.g. filesize modifitime and ustar indicators
    let mut headers: Vec<EntarWrapper> = vec![];
    // do not forget the first folder
    let father: &Path;
    if let Some(_father) = input_path.parent() {
        father = _father;
    } else {
        println!("It's a top dir maybe / or other errors");
        return Err(IOError::other("It's a top dir maybe / or other errors"));
    }

    if let Ok(rel) = input_path.strip_prefix(father) {
        if let Ok(meta) = input_path.metadata() {
            headers.push(EntarWrapper::new(
                input_path.to_path_buf(),
                rel.to_path_buf(),
                true,
                meta,
            ));
        } else {
            println!("Error get input_file metadata");
            return Err(IOError::other("Error get input_file metadata"));
        }
    }

    if let Err(e) = process_files_recursively(father, input_path, &mut headers) {
        return Err(e);
    }

    let _tar_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(output_file);

    if let Ok(mut tar_file) = _tar_file {
        for header in headers {
            // 先拿文件

            let is_folder = header.is_folder;
            let size = header.file_size;
            let pointer = header.file_pointer.clone();
            // write header first
            let head_buffer: [Byte; 512] = unsafe { mem::transmute(header.as_header()) };
            if let Err(_) = tar_file.write(&head_buffer) {
                return Err(IOError::other("write header to tar file failed!"));
            }

            // write file content then
            if !is_folder {
                if let Ok(mut f) = File::open(pointer.as_path()) {
                    let bf_size = 512 as u64;
                    let mut writed = 0;

                    let mut content_buffer = vec![0u8; bf_size as usize];
                    while writed < size {
                        if let Ok(_) = f.read(&mut content_buffer) {
                            if let Ok(wd) = tar_file.write(&content_buffer) {
                                writed += wd as u64;
                                content_buffer = vec![0u8; bf_size as usize];
                            } else {
                                println!("Error write tar file");
                                return Err(IOError::other("Error write tar file"));
                            }
                        } else {
                            println!("Error read source file");
                            return Err(IOError::other("Error read source file"));
                        }
                    }
                } else {
                    println!("Error open the to be writed file");
                    return Err(IOError::other("Error open the to be writed file"));
                }
            }
        }
        // we need last two empty chunks
        let empty_chunk: [Byte; 512] = [0x00u8; 512];
        for _ in 0..2 {
            if let Err(_) = tar_file.write(&empty_chunk) {
                println!("Error write two empty chunks");
                return Err(IOError::other("Error write two empty chunks"));
            }
        }
        if let Err(_) = tar_file.flush() {
            println!("Error flush tar file");
            return Err(IOError::other("Error flush tar file"));
        }
    }

    Ok(())
}

// this should be width first
pub fn process_files_recursively<'f>(
    father: &'f Path,
    input_path: &'f Path,
    vec: &mut Vec<EntarWrapper>,
) -> Result<(), IOError> {
    let mut dirs: VecDeque<Box<PathBuf>> = VecDeque::new();
    for entry in fs::read_dir(input_path)? {
        let e = entry?;
        let p = e.path();

        // 其实这里rel可以是string
        if let Ok(rel) = p.strip_prefix(father) {
            if p.is_file() {
                let f: File;
                if let Ok(_f) = File::open(p.as_path()) {
                    f = _f;
                } else {
                    println!("open dir {}", (&p).display());
                    panic!("can't open this file now");
                }
                if let Ok(meta) = f.metadata() {
                    vec.push(EntarWrapper::new(
                        p.to_path_buf(),
                        rel.to_path_buf(),
                        false,
                        meta,
                    ));
                } else {
                    println!("meta get {}", (&p).display());
                    panic!("can't get this file metadata");
                }
            } else if p.is_dir() {
                if let Ok(meta) = e.metadata() {
                    vec.push(EntarWrapper::new(
                        p.to_path_buf(),
                        rel.to_path_buf(),
                        true,
                        meta,
                    ));
                    dirs.push_back(Box::new(p.as_path().to_path_buf()));
                } else {
                    println!("meta get {}", (&p).display());
                    panic!("can't get this file metadata");
                }
            }
        } else {
            println!("caculate related dir failed");
            return Err(IOError::other("caculate related dir failed"));
        }
    }

    for dir in dirs {
        if let Err(_e) = process_files_recursively(father, dir.as_path(), vec) {
            panic!("process_files_recursively failed !");
        }
    }

    Ok(())
}

/**
 *  the path to the file like qzip\lib\tests\folder.tar
 *  the path to untar the file like qzip\lib\tests\tmp. it's always mean under the folder
 *  so the result will be qzip\lib\tests\tmp\"WHAT under folder.tar"
 *  in this folder.tar, it will be qzip\lib\tests\tmp\test_files
 *  so the default value will be ./ if output_path do not offer
 *  but who cares ?
 *  No LongLinks, it's just a trick game
 */
pub fn file_untar<'f>(input_file: &'f Path, output_path: &'f Path) -> Result<(), IOError> {
    // how can i use multiple threads to untar a file?
    let tar_file = File::open(input_file)?; // this is the pointer to the file / or stream
    // do not judge if the file exist, it's work of shell

    let metadata = tar_file.metadata()?;
    let sz = metadata.len();
    let stop_index: u64 = (sz / 512) as u64 - 2; // it must be 512's full multis
    println!("We stop read at {}", stop_index);

    let mut chunk: [Byte; 512] = [0u8; 512]; //  a buffer
    let mut chunk_index: u64 = 0;
    // a ordered list contains the file information.
    // in great situations, it will not cause memory problem
    let infos = &mut LinkedList::new();
    // maybe one day it can be asyc and multi threads
    loop {
        // read_exact every read is 512 so what we need is seek to new place
        match (&tar_file).read_exact(&mut chunk) {
            Ok(_) => {
                // an empty header
                let header: CommonHeader = unsafe { mem::transmute(chunk) };
                // assert it is a header
                if !(header.ustar_indicator == MAGIC_NUMBER) {
                    println!("ustar indicator this: |{:02x?}|", header.ustar_indicator);
                    return Err(IOError::other(
                        "maybe a longlink tar file or ustar indicator failed",
                    ));
                }
                if !(header.ustar_version == MAGIC_VERSION) {
                    println!("ustar version this: |{:02x?}|", header.ustar_indicator);
                    return Err(IOError::other(
                        "maybe a longlink tar file or ustar version failed",
                    ));
                }

                // let's read some info from the header
                let path = String::from_utf8_lossy(&header.path)
                    .into_owned()
                    .trim_end_matches('\0')
                    .to_string();
                let file_size: u64;
                if let Ok(fsz) = u64::from_str_radix(
                    String::from_utf8_lossy(&header.file_size)
                        .into_owned()
                        .trim_end_matches('\0'),
                    8,
                ) {
                    file_size = fsz;
                } else {
                    println!("Parse file size failed");
                    return Err(IOError::other("Parse file size failed"));
                }

                println!("info of header path is |{}|", &path);
                println!("size of the string path is {}", &path.len());
                println!("file size bytes: {:02x?}", &header.file_size);
                println!("file size is {}", file_size);

                // get the information necessary
                let maybe_a_dir = output_path.join(&path);
                chunk_index += 1;
                println!("Get a path: {}", maybe_a_dir.to_str().unwrap());
                if header.type_flag == 0x35 {
                    // it's almost must
                    infos.push_back(UntarInstance::new(Box::new(maybe_a_dir), true, 0, 0));
                } else {
                    // caculate the offset
                    let offset = (file_size + 511) / 512;
                    // magic do is plus 511 and / 512 向上取整 equals u64::div_ceil
                    // but this is more magic doesn't it ?
                    // both two passed tests
                    infos.push_back(UntarInstance::new(
                        Box::new(maybe_a_dir),
                        false,
                        file_size,
                        chunk_index,
                    ));
                    chunk_index += offset;
                    (&tar_file).seek(std::io::SeekFrom::Start(chunk_index * 512))?;
                }

                if chunk_index == stop_index {
                    break;
                }
            }
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => break, // finish read
            Err(e) => return Err(e),
        }
    }

    // get the infos and have a print
    infos.iter().for_each(|info| {
        if info.is_folder {
            if let Err(e) = fs::create_dir_all((&info).path.as_path()) {
                println!("failed to create folder: {}", e);
                return ();
            }
        } else {
            let path = (&info).path.as_path();
            let path_str;
            if let Some(s) = path.to_str() {
                path_str = s;
            } else {
                path_str = "fffff";
            }
            if let Ok(mut tmp) = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(path)
            {
                // here is the file size and where we start, buf directly write multi bytes will make everything crazy
                let mut writed: u64 = 0;
                let sutaible_size: usize;
                if info.file_size > 4096 {
                    sutaible_size = 4096;
                } else if info.file_size > 512 {
                    sutaible_size = 512;
                } else {
                    sutaible_size = info.file_size as usize;
                }
                while writed < info.file_size {
                    // read from the file
                    if let Ok(_) = (&tar_file).seek(SeekFrom::Start(info.seek_from * 512 + writed))
                    {
                        let malloc_size: usize;
                        if info.file_size - writed > sutaible_size as u64 {
                            malloc_size = sutaible_size;
                        } else {
                            malloc_size = (info.file_size - writed) as usize;
                        }
                        let mut big_buffer = vec![0u8; malloc_size];
                        if let Ok(bytes_count) = (&tar_file).read(&mut big_buffer) {
                            if let Err(_) = tmp.seek(SeekFrom::End(0)) {
                                println!("failed to seek to end: {}", path_str);
                            }
                            if let Err(_) = tmp.write(&big_buffer) {
                                println!("failed to write to file: {}", path_str);
                            }
                            writed += bytes_count as u64; // in most cases bytes_count = sutaible_size
                        } else {
                            println!("failed to read tar file");
                        }
                        // flush
                        if let Err(_) = (&tmp).flush() {
                            println!("failed to flush file: {}", path_str);
                        }
                    } else {
                        println!("failed to set file seek: {}", path_str);
                    }
                }
                // tmp.write();
            } else {
                println!("failed to open or write file: {}", path_str);
                return ();
            }
        }
    });

    Ok(())
}
