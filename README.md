# injesh

## 概要

- distrolessやscratchなどのシェルが無いコンテナ内に入り込みデバックするためのアプリケーション
- cgroup、ipc、net、pid、user、utsなどの名前空間をデバック対象コンテナと共有することでコンテナ内に入り込むことができる

## 利用方法

### 設定ファイルや.injeshディレクトリなどを初期化

injeshをインストールした後、一度だけ実行する

```bash
$ injesh init
```

### デバッグコンテナを新規作成し、デバックコンテナ内に入る

NAMEはデバックコンテナの名前。指定がない場合は自動生成。  
- デバックコンテナを生成した際に実行するコマンド。CMDの指定がない場合、デフォルトのシェルを利用
    ```bash
    $ injesh launch　[CONTAINER_ID or CONTAINER_NAME] [NAME] [CMD]
    ```
- 指定PATHのrootfsを基に起動
    ```bash
    $ injesh launch [CONTAINER_ID or CONTAINER_NAME] --rootfs=/path/to/rootfs [NAME]　[CMD]
    ```
- lxd image server からrootfsをDLしてから起動
    ```bash
    $ injesh launch [CONTAINER_ID or CONTAINER_NAME] --rootfs-image=<image-name> [NAME]　[CMD]
    ```
- 指定docker containerのrootfsを基に起動
    ```bash
    $ injesh launch [CONTAINER_ID or CONTAINER_NAME] --rootfs-docker=<docker-container-id> [NAME]　[CMD]
    ```
- 指定lxd containerのrootfsを基に起動
    ```bash
    $ injesh launch [CONTAINER_ID or CONTAINER_NAME] --rootfs-lxd=<lxd-container-name> [NAME]　[CMD]
    ```

### 既存の任意のコンテナに入りCMDを実行する

CMDがない場合は設定値ファイルのシェルを起動

```bash
$ injesh exec [NAME]　[CMD]
```

### デバッグコンテナを削除

```bash
$ injesh delete [NAME]
```

### デバッグコンテナを一覧表示

```bash
$ injesh list
```

### デバッグコンテナの指定ファイルをDL

```bash
$ injesh file pull [NAME]:/path/to/file /path/to/dest
```

### ホストの指定ファイルをデバッグコンテナの指定PATHにupload

```bash
$ injesh file push /path/to/file [NAME]:/path/to/dest
```

## ディレクトリ構造

```
.
|--config            # sample config file
|--images
|  |--busybox-1.32.1.tar.bz2  # lxd image server からDLして作成したrootfs
|--containers
|  |--bbox-goweb      # debug container name
|  |  |--setting.yaml # config file
|  |  |--merged
|  |  |--rootfs       # base rootfs
|  |  |--target_id    # docker container id (名前からIDを特定するため)
|  |
|  |--ubuntu-goweb
|  |  |--rootfs -> /path/to/custom/rootfs  # カスタムrootfsが散らばると汚いのでリンクを張る
|  |  |--target_id
|  |  |--upper
|  |  |--setting.yaml
```