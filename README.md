# rust tar

pure rust 实现 tar 格式的打包与解包。

## 功能

- **打包** `file_entar(input_path, output_file)` — 将文件夹打包成 tar 文件
- **解包** `file_untar(input_file, output_path)` — 将 tar 文件解压到目标目录

## 参数说明

```rust
// 打包: input_path 是待打包的文件夹, output_file 是输出的 tar 文件路径
file_entar(input_path: &Path, output_file: &Path)

// 解包: input_file 是 tar 文件路径, output_path 是解压到的目录
file_untar(input_file: &Path, output_path: &Path)
```

## 示例

```bash
# 打包文件夹
# 将 tests/test_files 文件夹打包为 output.tar
cargo run --bin rust_tar -- entar tests/test_files output.tar

# 解包 tar 文件
# 将 output.tar 解压到 ./output 目录
cargo run --bin rust_tar -- untar output.tar ./output
```

## 测试

```bash
cargo test --package rust_tar-lib
```

测试会进行 entar → untar 循环，验证解压后的文件与原文件 hash 一致。

## 项目结构

```
rust_tar/
├── bin/              # 可执行入口
│   └── src/main.rs
├── lib/              # 核心库
│   ├── src/tar/       # tar 打包解包实现
│   └── tests/         # 测试文件
└── Cargo.toml
```

## TODO
显然我们可以为它添加多线程功能，并且更动态地处理buffer区

LongLink是 Tar 格式的一个难点

它只有 Tar 功能，或许应该让它接入gz或者bz

增加 Shell 语法也是一个不错的选择
