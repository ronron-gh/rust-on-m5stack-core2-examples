# rust-on-m5stack-core2-examples
M5Stack Core2で実行できるRustのExample集です。  
[esp-idf-hal](https://github.com/esp-rs/esp-idf-hal/tree/master)を利用し、esp-idf-halのExampleや、書籍「基礎から学ぶ 組込みRust」（出版：株式会社C&R研究所　著者：中林 智之／井田 健太）でWio Terminal向けに実装しているプログラムをM5Stack Core2向けにカスタマイズしています。

> 現在ExampleはLチカとLCDグラフィック表示のみですが、徐々に増やしていきます。

**Contents:**
- [各Exampleの概要](#各exampleの概要)
  - [led\_blink](#led_blink)
  - [lcd\_ili9342c](#lcd_ili9342c)
- [開発環境インストール手順](#開発環境インストール手順)
  - [Rustインストール](#rustインストール)
  - [ESPツールチェーンをインストール](#espツールチェーンをインストール)
  - [cargo-generate (テンプレートからプロジェクトを作成するツール)をインストール](#cargo-generate-テンプレートからプロジェクトを作成するツールをインストール)
  - [テンプレートからプロジェクトを作成](#テンプレートからプロジェクトを作成)
  - [Build \& Flash](#build--flash)
  - [USBをWSL2にバインドする手順](#usbをwsl2にバインドする手順)
- [参考資料](#参考資料)


## 各Exampleの概要
> Note:  
> examplesフォルダのソースファイル(.rs)は次のコマンドでビルド、実行できます（事前に次節「開発環境インストール手順」を済ませてください）。

```
cargo build --example led_blink
cargo run --example led_blink
```

### led_blink
M5Stack Core2のGPIO G27のHigh/Lowを切り替えてLチカします。

![](images/led_blink.gif)

### lcd_ili9342c
M5Stack Core2に搭載されているLCD ili9342cに図形や画像を描画します。

> Note:  
> M5Stack Core2 v1.1ではLCDに電源供給する電源管理チップが異なるため動作しません。

![](images/lcd_ili9342c.png)


## 開発環境インストール手順
Ubuntu (WSL2)へのインストール手順を記載します。

### Rustインストール

```
$ sudo apt update
$ sudo apt upgrade
$ sudo apt install gcc
```

> Note:  
> RustコンパイラはCコンパイラのリンカを使うためgccをインストールしておく。

[Rustのページ](https://rustup.rs/)でインストールコマンドをコピーして実行。

```
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

環境変数を読み込むために、一度ターミナルを閉じて再ログインする。

インストールできたことを確認。
```
$ rustup --version
$ rustc --version
$ cargo --version
```

PC上でHello Worldの確認。  

```
$ cargo new hello
$ cd hello
$ cargo run
```

### ESPツールチェーンをインストール

```
$ cargo install espup --locked
$ espup install
```

環境変数設定。**これはターミナルを起動するたびに実行する必要がある**。
```
$ . ~/export-esp.sh
```

### cargo-generate (テンプレートからプロジェクトを作成するツール)をインストール

```
$ cargo install ldproxy
$ cargo install cargo-generate
```

### テンプレートからプロジェクトを作成
> Note:  
> 本リポジトリも同じテンプレートからプロジェクトを作成しています。本リポジトリをCloneすればプロジェクト作成はスキップできます。

作業フォルダで次のコマンドを実行する。
```
cargo generate --vcs none --git https://github.com/esp-rs/esp-idf-template cargo
```
対話形式でプロジェクト名、MCUなどを設定する。M5Stack Core2の場合、MCUはESP32。
![](images/cargo_generate.png)

### Build & Flash
> Note:  
> WSL2の場合は、事前に次節「USBをWSL2にバインドする手順」に沿ってUSBをWSL2側に接続してください。

作成したプロジェクトのフォルダに移動。
```
$ cd m5core2-esp-idf
```

ビルド、実行するとターミナルにシリアルログ（Hello world）が表示される。
```
$ cargo build
$ cargo run
```
![](images/cargo_run.png)

### USBをWSL2にバインドする手順
Windows側PowerShellでusbipdをインストール。
```
$ winget install --interactive --exact dorssel.usbipd-win
```

PCとM5StackをUSBで接続。**以降の手順はUSBを接続するたびに行う必要がある。**

PowerShellで以下を実行し、出力されたリストからM5Stackに接続されているUSBのbusidを探す（ここでは2-1）。
```
$ usbipd list
```
![](images/usbipd_list.png)

busidに対してバインドを実行。
```
$ usbipd bind --busid 2-1
```

もう一度リストを表示し、busidがsharedになったかを確認。（※上のスクリーンショットでは最初からsharedになっています。）
```
$ usbipd list
```

busidに対してWSL2への接続を実行。
```
$ usbipd attach --wsl --busid 2-1
```

Ubuntu(WSL2)側で接続されたUSBデバイスを確認。
```
$ dmesg | grep tty
```
![](images/grep_tty.png)

デバイスに対するアクセス権を設定。
```
$ sudo chmod 777 /dev/ttyACM0
```

## 参考資料
- [ESP32でstdなRust開発入門 -Lang-ship](https://lang-ship.com/blog/work/esp32-std-rust-1/)
- [M5StackをRustで動かす](https://zenn.dev/teruyamato0731/scraps/eaf1afddd92124)