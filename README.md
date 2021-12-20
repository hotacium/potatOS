# potatOS

前提:
- `cargo-make` のインストール
- `qemu-system-x86_64` のインストール
- `OVMF.fd` を `/usr/share/OVMF/x64/OVMF.fd` に配置する

実行方法:
```
# 必要があれば, cargo-make のインストール
cargo install cargo-make
# 実行
makers run
```
