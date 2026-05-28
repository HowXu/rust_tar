use crate::def::*;
use std::{
    collections::LinkedList,
    env,
    fmt::Error,
    fs::{self, File, OpenOptions},
    io::{ErrorKind, Read, Seek, SeekFrom, Write},
    mem,
    path::Path,
};

fn file_entar<'f>(input_path: &'f Path, output_file: &'f Path) -> Result<(), IOError> {
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
fn file_untar<'f>(input_file: &'f Path, output_path: &'f Path) -> Result<(), IOError> {
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
                if path.ends_with("/") {
                    // it's almost must
                    infos.push_back(Instance::new(Box::new(maybe_a_dir), true, 0, 0));
                } else {
                    // caculate the offset
                    let offset = u64::div_ceil(file_size, 512);
                    infos.push_back(Instance::new(
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
                        if let Err(_) = (&tmp).flush(){
                            println!("failed to flush file: {}",path_str);
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

#[test]
fn test_print_workspace() {
    // get the cur dir is better
    println!("current_exe is: {}", env::current_exe().unwrap().display());
    println!("current_dir is: {}", env::current_dir().unwrap().display());
}

#[test]
fn test_file_untar() -> Result<(), IOError> {
    let cur_path: &Path;
    let cur_path_str: String;
    let cur_dir = env::current_dir()?; // cur_dir will be re used
    if let Some(p) = cur_dir.as_path().to_str() {
        cur_path_str = String::from(p);
        cur_path = Path::new(p);
    } else {
        return Err(IOError::other("unavalaible cur dir"));
    }

    println!("cur is {}", cur_path_str);

    let test_files_folder = cur_path.join("tests"); // this is qzip\lib\tests
    let tmp_folder = cur_path.join("tests").join("tmp"); // this is qzip\lib\tests\tmp

    if !tmp_folder.exists() {
        fs::create_dir_all(&tmp_folder)?;
    } // tmp folder

    // do untar in tmp folder
    // do not give it state
    let result = file_untar(
        &test_files_folder.join("folder.tar").as_path(),
        &tmp_folder.clone().as_path(),
    );

    assert!(result.is_ok());

    use blake3::Hasher;
    use std::io::Read;

    let to_do_list = vec![
        "test_files/中文示例/中文示例.txt",
        "test_files/bar/apple.txt",
        "test_files/long.txt",
        "test_files/banana.txt",
        "test_files/😁.txt",
    ];
    to_do_list.iter().for_each(|s| {
        let mut test_file = File::open(&test_files_folder.join(s)).unwrap();
        let mut tmp_file = File::open(&tmp_folder.join(s)).unwrap();
        let mut hasher_a = Hasher::new();
        let mut hasher_b = Hasher::new();
        let mut buf = [0u8; 65536]; // 64KB 块更高效

        loop {
            if let Ok(n) = test_file.read(&mut buf){
                if n == 0 {
                break;
            }
            hasher_a.update(&buf[..n]);
            }
            
        }

        loop{
            if let Ok(n) = tmp_file.read(&mut buf){
                if n == 0 {
                break;
            }
            hasher_b.update(&buf[..n]);
            }
        }
        assert!(hasher_a.finalize().as_bytes() ==  hasher_b.finalize().as_bytes());
        println!("file hash success: {}",s);
    });

    // remove all
    fs::remove_dir_all(tmp_folder)?;
    Ok(())
}