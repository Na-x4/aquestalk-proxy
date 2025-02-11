# AquesTalk-proxy

32bit バイナリでしか動かなかった旧ライセンス版 AquesTalk を外部プロセスで実行することで利用できるようにするプログラム。
インターフェースに標準入出力または TCP ソケットを使用することができる。

AquesTalk のライセンス変更については[公式ブログ][blog.a-quest]を参照してください。

## How To Use

### Windows

[リリースページ][release]から zip ファイルをダウンロードして解凍

```
> chcp 65001
> echo {"koe":"こんにちわ、せ'かい"} | aquestalk-proxy.exe
{"isSuccess":true,"response":{"type":"Wav","wav":"UklGRoxd...AA=="},"request":{"koe":"こんにちわ、せ'かい"}}
```

## Protocol

AquesTalk-proxy はシンプルな JSON ストリーミングプロトコルです。
`Request` メッセージを送信すると、`Response` メッセージで応答します。
応答は改行 (`\n`: LF) 区切りで 1 行が 1 つのメッセージに対応します。

`Response.willClose` が `true` ではない間、何度でも `Request` メッセージを送信できます。
リクエストの間に区切り文字は必要ありません。

`Response.willClose` が `true` を返した場合は回復不能なエラーが発生しています。
TCP モードの場合はサーバー側の接続がクローズするため、再接続が必要になります。
標準入出力モードの場合にはプロセスが終了します。再度実行してください。

## Message

TypeScript での定義

```ts
interface Request {
  type?: string; // 声種 デフォルト: f1
  koe: string; // 音声記号列
  speed?: number; // 発話速度[%] 50-300 の間で指定 デフォルト: 100 値を大きく設定するほど、速くなる
}

interface Response {
  isSuccess: boolean; // true -> リクエストの結果が成功
  willClose?: boolean; // true -> 続けて新たなリクエストを受付不可
  response:
    | {
        type: "Wav"; // -> WAV データ
        wav: string; // Base64 エンコードされた WAV データ
      }
    | {
        type: "AquestalkError"; // -> AquesTalk ライブラリ内エラー
        code?: number; // エラーコード (AquesTalk ライブラリ内でエラーが発生した場合)
        message: string; // エラーメッセージ
      }
    | {
        type: "JsonError"; // -> JSON 構文エラーまたは型エラー
        message: string; // エラーメッセージ
      }
    | {
        type: "IoError"; // -> 入出力エラー
        message: string; // エラーメッセージ
      };
  request?: any; // 対応するリクエスト (JSON 構文エラーまたは入出力エラーが発生しなかった場合)
}
```

## Options

```
aquestalk-proxyd.exe [OPTIONS] [MODE]
```

| オプション            | 説明                                         | デフォルト                          |
| --------------------- | -------------------------------------------- | ----------------------------------- |
| `-p`, `--path` `PATH` | AquesTalk ライブラリのディレクトリパスを指定 | `-p カレントディレクトリ/aquestalk` |

AquesTalk ライブラリのディレクトリ構成は以下のようにする

```
aquestalk/
  +- [声種1]/
  |    +- AqLicense.txt
  |    +- AquesTalk.dll
  |    +- AquesTalkDa.dll
  +- [声種2]/
  |    +- AqLicense.txt
  |    +- AquesTalk.dll
  |    +- AquesTalkDa.dll
  ⋮
```

| モード  | 説明                          |
| ------- | ----------------------------- |
| `tcp`   | TCP ソケットモード            |
| `stdio` | 標準入出力モード (デフォルト) |

### Standard IO Mode (標準入出力モード)

```
aquestalk-proxyd.exe stdio [OPTIONS]
```

オプションなし

### TCP Socket Mode (TCP ソケットモード)

```
aquestalk-proxyd.exe tcp [OPTIONS]
```

| オプション              | 説明                                                                                                     | デフォルト                            |
| ----------------------- | -------------------------------------------------------------------------------------------------------- | ------------------------------------- |
| `-l`, `--listen` `ADDR` | 待ち受けするアドレスとポートを指定する。複数指定可能。                                                   | `-l 127.0.0.1:21569` `-l [::1]:21569` |
| `-n`, `--threads` `NUM` | リクエストを処理するスレッド数を指定。同時に処理可能なリクエスト数となる。                               | `-n 1`                                |
| `--timeout` `MILLIS`    | タイムアウトするまでの時間 (ms) を指定する。前回の要求から指定した時間要求が無い場合接続をクローズする。 | 指定なし                              |
| `--limit` `BYTES`       | 1 回の接続で可能な要求の長さを指定する。                                                                 | 指定なし                              |

## Develop

`i686-pc-windows-gnu` をターゲットとしてビルドできるように Rust をセットアップする。

```
$ git clone https://github.com/Na-x4/aquestalk-proxy.git
$ cd aquestalk-proxy
$ ./scripts/extract-aqtk.sh
$ cargo run --target=i686-pc-windows-gnu -p aquestalk-proxyd --release -- tcp &
$ cargo test --target=i686-pc-windows-gnu
```

## Licence

`lib` ディレクトリ以下のソースコードは MIT license と Apache License (Version 2.0) のデュアルライセンスの下で頒布されています。
それ以外のソースコードは GNU Affero General Public License の下で頒布されています。

本プログラムは、株式会社アクエストの規則音声合成ライブラリ「AquesTalk」を使用しています。
`aquestalk` ディレクトリ以下のファイル、及び `aqtk_mv_20090609.zip` ファイルの著作権は同社に帰属します。
詳細は `AqLicense.txt` をご覧ください。

`aquestalk-proxyd` で使用している OSS は以下の通りです。

| Name                                                                   | License                                                    | Author(s)                                                                                                                                            |
| ---------------------------------------------------------------------- | ---------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| [base64](https://github.com/marshallpierce/rust-base64) 0.22.1         | [Apache-2.0] OR [MIT]                                      | <ul><li>Marshall Pierce &lt;<marshall@mpierce.org>&gt;</li></ul>                                                                                     |
| [cfg-if](https://github.com/alexcrichton/cfg-if) 1.0.0                 | [Apache-2.0] OR [MIT]                                      | <ul><li>Alex Crichton &lt;<alex@alexcrichton.com>&gt;</li></ul>                                                                                      |
| [encoding_rs](https://github.com/hsivonen/encoding_rs) 0.8.35          | ([Apache-2.0] OR [MIT]) AND [BSD-3-Clause][whatwg license] | <ul><li>Mozilla Foundation</li></ul>                                                                                                                 |
| [getopts](https://github.com/rust-lang/getopts) 0.2.21                 | [Apache-2.0] OR [MIT]                                      | <ul><li>The Rust Project Developers</li></ul>                                                                                                        |
| [hermit-abi](https://github.com/hermit-os/hermit-rs) 0.3.9             | [Apache-2.0] OR [MIT]                                      | <ul><li>Stefan Lankes</li></ul>                                                                                                                      |
| [itoa](https://github.com/dtolnay/itoa) 1.0.14                         | [Apache-2.0] OR [MIT]                                      | <ul><li>David Tolnay &lt;<dtolnay@gmail.com>&gt;</li></ul>                                                                                           |
| [libc](https://github.com/rust-lang/libc) 0.2.169                      | [Apache-2.0] OR [MIT]                                      | <ul><li>The Rust Project Developers</li></ul>                                                                                                        |
| [libloading](https://github.com/nagisa/rust_libloading/) 0.8.6         | [ISC]                                                      | <ul><li>Simonas Kazlauskas &lt;<libloading@kazlauskas.me>&gt;</li></ul>                                                                              |
| [memchr](https://github.com/BurntSushi/memchr) 2.7.4                   | [MIT] OR [Unlicense]                                       | <ul><li>Andrew Gallant &lt;<jamslam@gmail.com>&gt;</li></ul>                                                                                         |
| [num_cpus](https://github.com/seanmonstar/num_cpus) 1.16.0             | [Apache-2.0] OR [MIT]                                      | <ul><li>Sean McArthur &lt;<sean@seanmonstar.com>&gt;</li></ul>                                                                                       |
| [optional_take](https://github.com/Na-x4/optional_take) 0.1.0          | [Apache-2.0] OR [MIT]                                      | <ul><li>Na-x4 &lt;<Na-x4@outlook.com>&gt;</li></ul>                                                                                                  |
| [proc-macro2](https://github.com/dtolnay/proc-macro2) 1.0.93           | [Apache-2.0] OR [MIT]                                      | <ul><li>David Tolnay &lt;<dtolnay@gmail.com>&gt;</li><li>Alex Crichton &lt;<alex@alexcrichton.com>&gt;</li></ul>                                     |
| [quote](https://github.com/dtolnay/quote) 1.0.38                       | [Apache-2.0] OR [MIT]                                      | <ul><li>David Tolnay &lt;<dtolnay@gmail.com>&gt;</li></ul>                                                                                           |
| [ryu](https://github.com/dtolnay/ryu) 1.0.19                           | [Apache-2.0] OR [BSL-1.0]                                  | <ul><li>David Tolnay &lt;<dtolnay@gmail.com>&gt;</li></ul>                                                                                           |
| [serde](https://github.com/serde-rs/serde) 1.0.217                     | [Apache-2.0] OR [MIT]                                      | <ul><li>Erick Tryzelaar &lt;<erick.tryzelaar@gmail.com>&gt;</li><li>David Tolnay &lt;<dtolnay@gmail.com>&gt;</li></ul>                               |
| [serde_derive](https://github.com/serde-rs/serde) 1.0.217              | [Apache-2.0] OR [MIT]                                      | <ul><li>Erick Tryzelaar &lt;<erick.tryzelaar@gmail.com>&gt;</li><li>David Tolnay &lt;<dtolnay@gmail.com>&gt;</li></ul>                               |
| [serde_json](https://github.com/serde-rs/json) 1.0.138                 | [Apache-2.0] OR [MIT]                                      | <ul><li>Erick Tryzelaar &lt;<erick.tryzelaar@gmail.com>&gt;</li><li>David Tolnay &lt;<dtolnay@gmail.com>&gt;</li></ul>                               |
| [syn](https://github.com/dtolnay/syn) 2.0.98                           | [Apache-2.0] OR [MIT]                                      | <ul><li>David Tolnay &lt;<dtolnay@gmail.com>&gt;</li></ul>                                                                                           |
| [threadpool](https://github.com/rust-threadpool/rust-threadpool) 1.8.1 | [Apache-2.0] OR [MIT]                                      | <ul><li>The Rust Project Developers</li><li>Corey Farwell &lt;<coreyf@rwell.org>&gt;</li><li>Stefan Schindler &lt;<dns2utf8@estada.ch>&gt;</li></ul> |
| [unicode-ident](https://github.com/dtolnay/unicode-ident) 1.0.16       | ([MIT] OR [Apache-2.0]) AND [Unicode-3.0]                  | <ul><li>David Tolnay &lt;<dtolnay@gmail.com>&gt;</li></ul>                                                                                           |
| [unicode-width](https://github.com/unicode-rs/unicode-width) 0.1.14    | [Apache-2.0] OR [MIT]                                      | <ul><li>kwantam &lt;<kwantam@gmail.com>&gt;</li><li>Manish Goregaokar &lt;<manishsmail@gmail.com>&gt;</li></ul>                                      |
| [whatwg/encoding](https://github.com/whatwg/encoding)                  | [CC BY 4.0, BSD-3-Clause][whatwg license]                  | <ul><li>WHATWG (Apple, Google, Mozilla, Microsoft)</li></ul>                                                                                         |
| [windows-targets](https://github.com/microsoft/windows-rs) 0.52.6      | [Apache-2.0] OR [MIT]                                      | <ul><li>Microsoft</li></ul>                                                                                                                          |

[blog.a-quest]: http://blog-yama.a-quest.com/?eid=970181
[release]: https://github.com/Na-x4/aquestalk-proxy/releases
[apache-2.0]: https://www.apache.org/licenses/LICENSE-2.0
[mit]: https://opensource.org/licenses/MIT
[whatwg license]: https://raw.githubusercontent.com/whatwg/encoding/refs/heads/main/LICENSE
[unlicense]: https://unlicense.org/
[isc]: https://opensource.org/licenses/ISC
[bsl-1.0]: https://www.boost.org/LICENSE_1_0.txt
[unicode-3.0]: https://www.unicode.org/license.txt
