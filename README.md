# AquesTalk-proxy

32bit バイナリでしか動かなかった旧ライセンス版 AquesTalk を Socket 越しに呼び出せるようにするプログラム。

AquesTalk のライセンス変更については[公式ブログ][blog.a-quest]を参照してください。

## Licence

The source code is licensed AGPLv3.

## How To Use

### Docker

```
$ docker build -t aquestalk-proxy https://github.com/Na-x4/aquestalk-proxy.git
$ docker run -d --platform=linux/386 -p 21569:21569 aquestalk-proxy
```

## Develop

```
$ git clone https://github.com/Na-x4/aquestalk-proxy.git
$ cd aquestalk-proxy
$ docker build -f Dockerfile.dev -t aquestalk-proxy-dev .
$ docker run -it --name=dev \
  --mount type=bind,source="$(pwd)"/,target=/home/user/aquestalk-proxy/ \
  -p 21569:21569 \
  -u $(id -u):$(id -g) \
  aquestalk-proxy-dev bash

user@docker:~$ cd aquestalk-proxy
user@docker:~$ cargo test
```

## Required Notices

- 本プログラムは、株式会社アクエストの規則音声合成ライブラリ「AquesTalk」を使用しています。
- `aquestalk` ディレクトリ以下のファイル、及び `aqtk_mv_20090609.zip` ファイルの著作権は同社に帰属します。
  - 詳細は `AqLicense.txt` をご覧ください。

[blog.a-quest]: http://blog-yama.a-quest.com/?eid=970181
