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

### Docker

```
$ docker pull nax4/aquestalk-proxy
$ echo "{\"koe\":\"こんにちわ、せ'かい\"}" | docker run -i --rm --platform=linux/386 nax4/aquestalk-proxy
{"isSuccess":true,"response":{"type":"Wav","wav":"UklGRoxd...AA=="},"request":{"koe":"こんにちわ、せ'かい"}}
```

## Protocol

AquesTalk-proxy はシンプルな JSON ストリーミングプロトコルです。
`Request` メッセージを送信すると、`Result` メッセージで応答します。

`Result.willClose` が `true` ではない間、何度でも `Request` メッセージを送信できます。
リクエストの間に区切り文字は必要ありません。

`Result.willClose` が `true` を返した場合は回復不能なエラーが発生しています。
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

interface Result {
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

## Develop

```
$ cargo install cross
$ git clone https://github.com/Na-x4/aquestalk-proxy.git
$ cd aquestalk-proxy
$ cross test --target=i686-pc-windows-gnu
```

## Licence

- `lib` ディレクトリ以下のソースコードは MIT license と the Apache License (Version 2.0) のデュアルライセンスの下で頒布されています。
- それ以外のソースコードは GNU Affero General Public License の下で頒布されています。

- 本プログラムは、株式会社アクエストの規則音声合成ライブラリ「AquesTalk」を使用しています。
  - `aquestalk` ディレクトリ以下のファイル、及び `aqtk_mv_20090609.zip` ファイルの著作権は同社に帰属します。
  - 詳細は `AqLicense.txt` をご覧ください。

[blog.a-quest]: http://blog-yama.a-quest.com/?eid=970181
[release]: https://github.com/Na-x4/aquestalk-proxy/releases
