# Rust OS memo

[Rust UEFI Book](https://rust-osdev.github.io/uefi-rs/introduction.html)
[UEFI 仕様 2.11](https://uefi.org/specs/UEFI/2.11/)

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

### ライフタイム

全参照にはライフタイムがあり、参照を使用する関数や構造体にはライフタイム引数を指定する必要があります。

```rust
stract Example<'a> {
    raw: &'a [u8; 256]
}

fn lifetime_func<'a>(input: &'a str) -> &'a str {
    input
}
```

#### ライフタイム省略

[参考](https://doc.rust-jp.rs/book-ja/ch10-03-lifetime-syntax.html#%E3%83%A9%E3%82%A4%E3%83%95%E3%82%BF%E3%82%A4%E3%83%A0%E7%9C%81%E7%95%A5)

コンパイラの参照解析に落とし込まれたパターンは、ライフタイム省略規則と呼ばれます。
省略規則は、完全な推論を提供しません。

関数やメソッドの引数のライフタイムは、入力ライフタイムと呼ばれ、 戻り値のライフタイムは出力ライフタイムと称されます。

> 毎回書くと冗長になるため、よくあるパターンだけは自動的に推論するように

#### 規則1
---
「参照である各引数は、独自のライフタイム引数を得る」

つまり：

```rust
fn foo<'a>(x: &'a i32);         // 引数1つ → ライフタイム 'a
fn foo<'a, 'b>(x: &'a i32, y: &'b i32);  // 引数2つ → 'a, 'b
```
省略して書いても、コンパイラはそれぞれの参照に個別のライフタイムをつけて解釈します。



#### 規則2
---
「1つだけ入力ライフタイムがあるなら、そのライフタイムがすべての出力に代入される」

```rust

fn get<'a>(x: &'a i32) -> &'a i32  // 明示
// 省略形:
fn get(x: &i32) -> &i32            // 自動的に上記と等価

```


#### 規則3
---

「メソッドで、入力に &self または &mut self があるなら、self のライフタイムが出力に使われる」

```rust
impl<'a> Wrapper<'a> {
    fn get(&self) -> &str          // ライフタイム省略されているが…
    // 実際は:
    fn get<'a>(&'a self) -> &'a str
}
```


### スタティックライフタイム

[参考](https://doc.rust-jp.rs/rust-by-example-ja/scope/lifetime/static_lifetime.html)

Rustにおける`'static`ライフタイムは、最も長いライフタイムです。

参照のライフタイムが`'static`であることは、参照が指し示す値がプログラムの実行中に渡って生き続けることを示します。 また、より短いライフタイムに圧縮することも可能です。

#### 文字列リテラル
---
`"これは静的文字列"`はバイナリに埋め込まれるため、プログラムの寿命中ずっと有効です。


```rust
let msg: &'static str = "これは静的文字列";
```

#### static変数
---
CONFIGは 'static ライフタイムを持ち、プログラム終了まで保持されます。

```rust
static CONFIG: Config = Config::new();
```

#### ヒープに確保して、明示的にリークさせた値
---
Box::leakにより、ヒープ上に確保された値の所有権が失われ、プログラム終了まで残ります。


```rust
let s: &'static str = Box::leak(Box::new("leaked string".to_string()));
```


#### `'static`と`&'static`の違い

- `&'static T`
  - 値`T`への参照が`'static`(参照先がプログラム中ずっと有効)
- `'static T`
  - 所有する値自体が`'satatic`(例：static変数、グローバル変数等)

### null未チェックバージョン
- [参照1](https://doc.rust-lang.org/std/primitive.pointer.html#null-unchecked-version)

- [参照2](https://doc.rust-lang.org/reference/lifetime-elision.html)

Rustでは、生ポインタ（*const T や *mut T）を安全な参照（&T や &mut T）に変換する操作は、unsafe を伴います。これはコンパイラが「そのポインタが有効で非nullである」と保証できないためです。

このような変換は以下のように行います：

```rust
unsafe { &*value }
```
これは、ポインタが指すメモリ位置から参照を作るという意味で、「null未チェックバージョン」と呼ばれます。
ポインタがnullである場合、未定義動作（undefined behavior）になります。


##### ❗補足：関数内で生成したデータの参照を返すケース
以下のような「関数内で生成した一時的な値の参照を返す」ことは安全でもunsafeでも不可能です：

```rust
fn get_ref() -> &i32 {
    let x = 42;
    &x  // ❌ これはコンパイルエラー。xは関数のスコープで消えるため
}
```

このようなケースはライフタイムが明らかに無効になるため、Rustの借用チェッカーが静的に拒否します。

ただし、グローバル変数やBox::leak()、static変数など、寿命が十分に長いデータからであれば参照は返せます。以下のように：

```rust
fn get_ref() -> &'static i32 {
    Box::leak(Box::new(42))  // ✅ 'static な参照を返す
}
```


## Rustのツールチェイン

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