use std::{fmt::Error, fs::File};
use crate::def::*;

fn file_entar<'f>(input:&'f mut File, output:&'f mut File) -> Result<(),Error>{
    Ok(())
}

#[test]
fn test_header_size(){
    assert_eq!(size_of::<CommonHeader>(),500); // it must be 500 size
}
