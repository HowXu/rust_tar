use std::{env, fs::{self, File}, path::Path};
use crate::def::*;

fn file_entar<'f>(input_path:&'f Path, output_file:&'f Path) -> Result<(),IOError>{
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
fn file_untar<'f>(input_file:&'f Path, output_path:&'f Path) -> Result<(),IOError>{
    // how can i use multiple threads to untar a file?
    let tar_file = File::open(input_file); // this is the pointer to the file / or stream
    // do not judge if the file exist, it's work of shell


    Ok(())
}

#[test]
fn test_print_workspace() {
    // get the cur dir is better
    println!("current_exe is: {}",env::current_exe().unwrap().display());
    println!("current_dir is: {}",env::current_dir().unwrap().display());
}

#[test]
fn test_file_untar() -> Result<(),IOError> {
    let cur_path: &Path;
    let cur_path_str: String;
    let cur_dir = env::current_dir()?; // cur_dir will be re used
    if let Some(p) = cur_dir.as_path().to_str() {
        cur_path_str = String::from(p);
        cur_path = Path::new(p);
    } else {
        return Err(IOError::other("unavalaible cur dir"));
    }

    println!("cur is {}",cur_path_str);

    let test_files_folder = cur_path.join("tests"); // this is qzip\lib\tests
    let tmp_folder = cur_path.join("tests").join("tmp"); // this is qzip\lib\tests\tmp

    if !tmp_folder.exists(){
        fs::create_dir_all(&tmp_folder)?;
    } // tmp folder

    // do untar in tmp folder
    // do not give it state
    let result = file_untar(&test_files_folder.join("folder.tar").as_path(), &tmp_folder.clone().as_path());


    // some asserts
    assert!(result.is_ok());


    // remove all
    fs::remove_dir_all(tmp_folder)?;
    Ok(())
}
