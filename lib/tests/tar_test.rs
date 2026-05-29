use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use blake3::Hasher;
use qzip_lib::tar::file_tar::{file_entar, file_untar};

fn hash_file(path: &Path) -> [u8; 32] {
    let mut file = File::open(path).unwrap();
    let mut hasher = Hasher::new();
    let mut buf = [0u8; 65536];
    loop {
        if let Ok(n) = file.read(&mut buf) {
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
        }
    }
    *hasher.finalize().as_bytes()
}

#[test]
fn test_file_entar_untar() {
    let manifest_path = Path::new(env!("CARGO_MANIFEST_DIR"));
    let test_files_folder = manifest_path.join("tests").join("test_files");
    let tmp_folder = manifest_path.join("tests").join("tmp_enttar");

    let _ = fs::remove_dir_all(&tmp_folder);
    fs::create_dir_all(&tmp_folder).unwrap();

    let tar_file = tmp_folder.join("output.tar");

    file_entar(test_files_folder.as_path(), tar_file.as_path()).unwrap();

    let extract_folder = tmp_folder.join("extracted");
    fs::create_dir_all(&extract_folder).unwrap();
    file_untar(tar_file.as_path(), extract_folder.as_path()).unwrap();

    let to_do_list = vec![
        "中文示例/中文示例.txt",
        "bar/apple.txt",
        "long.txt",
        "banana.txt",
        "😁.txt",
        "empty.txt",
    ];

    for s in &to_do_list {
        let orig = test_files_folder.join(s);
        let extracted = extract_folder.join("test_files").join(s);
        assert_eq!(hash_file(&orig), hash_file(&extracted), "hash mismatch: {s}");
    }

    fs::remove_dir_all(&tmp_folder).unwrap();
}

#[test]
fn test_file_untar_existing() {
    let manifest_path = Path::new(env!("CARGO_MANIFEST_DIR"));
    let tar_file = manifest_path.join("tests").join("folder.tar");
    let tmp_folder = manifest_path.join("tests").join("tmp_untar");

    let _ = fs::remove_dir_all(&tmp_folder);
    fs::create_dir_all(&tmp_folder).unwrap();

    file_untar(tar_file.as_path(), tmp_folder.as_path()).unwrap();

    let to_do_list = vec![
        "test_files/中文示例/中文示例.txt",
        "test_files/bar/apple.txt",
        "test_files/long.txt",
        "test_files/banana.txt",
        "test_files/😁.txt",
    ];

    for s in &to_do_list {
        let extracted = tmp_folder.join(s);
        assert!(extracted.exists(), "missing: {s}");
    }

    fs::remove_dir_all(&tmp_folder).unwrap();
}

#[test]
fn test_file_entar_existing() {
    let manifest_path = Path::new(env!("CARGO_MANIFEST_DIR"));
    let test_files_folder = manifest_path.join("tests").join("test_files");
    let tmp_folder = manifest_path.join("tests").join("tmp_enttar2");

    let _ = fs::remove_dir_all(&tmp_folder);
    fs::create_dir_all(&tmp_folder).unwrap();

    let tar_file = tmp_folder.join("folder.tar");

    file_entar(test_files_folder.as_path(), tar_file.as_path()).unwrap();
    assert!(tar_file.exists());

    fs::remove_dir_all(&tmp_folder).unwrap();
}