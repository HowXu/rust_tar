use crate::def::*;
use std::{
    env,
    fs::{self, File},
    io::{ErrorKind, Read},
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
 */
fn file_untar<'f>(input_file: &'f Path, output_path: &'f Path) -> Result<(), IOError> {
    // how can i use multiple threads to untar a file?
    let mut tar_file = File::open(input_file)?; // this is the pointer to the file / or stream
    // do not judge if the file exist, it's work of shell

    let mut chunk: [Byte; 512] = [0u8; 512]; //  a buffer

    // TODO
    // 第一个块进行一次ustar的判断 然后可以获得一个filesize
    // 根据这个filesize 除以 512 向上取整读块 就能每次指针重写read都读到块了
    loop {
        match tar_file.read_exact(&mut chunk) {
            Ok(_) => {
                // an empty header
                let header: CommonHeader = unsafe { mem::transmute(chunk) };
                println!("ustar_indicator bytes: {:02x?}", &header.ustar_indicator);
                println!("ustar_version bytes: {:02x?}", &header.ustar_version);
                println!("file size bytes: {:02x?}", &header.file_size);
                // let's read some info from the header
                let path = String::from_utf8_lossy(&header.path)
                    .into_owned()
                    .trim_end_matches('\0')
                    .to_string();

                let ustar_indicator = String::from_utf8_lossy(&header.ustar_indicator).into_owned();
                let ustar_version = String::from_utf8_lossy(&header.ustar_version).into_owned();
                let file_size: u64;

                if let Ok(fsz) = u64::from_str_radix(
                    String::from_utf8_lossy(&header.file_size)
                        .into_owned()
                        .trim_end_matches('\0'),
                    8
                ) {
                    file_size = fsz;
                } else {
                    println!("Parse file size failed");
                    return Err(IOError::other("Parse file size failed"));
                }

                println!("file size is {}",file_size);

                // this two will help us judge if it's a tar file?

                // u8刚好是8位二进制 也就是单个16进制

                println!("info of header path is |{}|", &path);
                println!("size of the string path is {}", &path.len());
                println!("read ustar_indicator: |{}|", ustar_indicator);
                println!(
                    "size of the string ustar_indicator is {}",
                    &ustar_indicator.len()
                );
                println!("read ustar_version: |{}|", ustar_version);
                println!(
                    "size of the string ustar_version is {}",
                    &ustar_version.len()
                );
                // in unix everything is a file, expected folder so there will be a folder first

                // judge only path and something we need. for example we needn't permissions now.
                // get the information necessary
                let maybe_a_dir = output_path.join(&path);
                if path.ends_with("/") {
                    // it's almost must
                    fs::create_dir_all(&maybe_a_dir)?;
                }
                // else it is a file, we should touch it
                else {
                    File::create(&maybe_a_dir)?;
                    // if no, it will be created else cleared
                }
                assert!(maybe_a_dir.exists()); // judge if the folder/file is created
                // let's continue read the file data next chunk

                break;
            }
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => break, // finish read
            Err(e) => return Err(e),
        }
    }

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

    // some asserts
    assert!(result.is_ok());

    // remove all
    fs::remove_dir_all(tmp_folder)?;
    Ok(())
}
