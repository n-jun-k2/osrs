# Rust OS memo

## QEMUを使ったバイナリファイルの実行

まず作成したバイナリファイルがブート可能かを調べる
```bash
$ file /work/img.hex
/work/img.hex: DOS/MBR boot sector
```

もしブート可能でない場合以下のコマンドで変換できる
```bash
$ xxd -r -p /work/img.hex > img.bin
```

QEMUを使用し作成したブート可能なバイナリデータを読み込ませる方法
```bash
$ qemu-system-x86_64 -drive file=img.hex,format=raw -nographic
```

## Rustについて

#### Rustのツールチェイン

バージョンを固定するには`rust-toolchain.yaml`というファイルをRustのプロジェクト内に作成しそれぞれ項目を設定する

```toml
[toolchain]
channel = "<使用するRustのバージョン>""
components = ["<使用するコンポーネント:コンパイラやパッケージマネージャ>"]
targets = ["<コンパイルターゲット>"]
profile = "<>"
```

反映されているかどうかは`rustup show`で確認する

```bash
$ rustup show
Default host: x86_64-unknown-linux-musl
rustup home:  /usr/local/lib/rust/rustup

installed toolchains
--------------------
nightly-x86_64-unknown-linux-musl (default)
nightly-2024-01-01-x86_64-unknown-linux-musl (active)

active toolchain
----------------
name: nightly-2024-01-01-x86_64-unknown-linux-musl
active because: overridden by '/mnt/work/repo/wasabi/rust-toolchain.toml'
installed targets:
  x86_64-unknown-linux-gnu
  x86_64-unknown-linux-musl
```

もし`channel`に指定したrustが上手くいかない場合は以下の様にすると上手くいく場合がある

```bash
$ rustup set default-host x86_64-unknown-linux-gnu
$ rustup toolchain install nightly-2024-01-01
```

以下の様になる

```bash
/mnt/work/repo/wasabi # rustup default
nightly-x86_64-unknown-linux-musl (default)
/mnt/work/repo/wasabi # rustup show active-toolchain
nightly-2024-01-01-x86_64-unknown-linux-gnu (overridden by '/mnt/work/repo/wasabi/rust-toolchain.toml')
```
## コマンドメモ:

```bash
# UEFI用ビルドコマンド
cargo build --target x86_64-unknown-uefi

# ファイルの種類を調べるコマンド
file (target file path)

# UEFIの画面表示
qemu-system-x86_64 -bios ./third_party/ovmf/RELEASEX64_OVMF.fd -display gtk


# UEFIの画面表示+プログラム実行

cp repo/wasabi/target/x86_64-unknown-uefi/debug/wasabi.efi mnt/EFI/BOOT/BOOTX64.EFI

qemu-system-x86_64 -bios ./third_party/ovmf/RELEASEX64_OVMF.fd -display gtk -drive format=raw,file=fat:rw:mnt

```