# Music Tag Cli [English](README) | [中文简体](README_zh-cn)

這是一個用於在命令列中編輯音樂標籤的簡單工具。您可以透過SSH、NFS或Samba（Windows共享資料夾）連線您的音樂伺服器，然後您就可以使用它修改檔案中的音樂標籤。`music-tag-cli`可以批次修改資料夾及其子資料夾的檔案，或僅僅修改單個檔案。它可以插入或追加序列數，也可以將繁體中文轉換為簡體中文。`music-tag-cli`支援FLAC, APE, WAV, AIFF, WV, TTA, MP3, M4A, OGG, MPC, OPUS, WMA, DSF, DFF, MP4等音訊檔案格式。

## 安裝

請閱讀[安裝指南](INSTALL)。如果您只是想在Windows平臺下使用此程式，可以直接下載。

## 标签表

| 標籤         | 名稱       | 型別 |
|--------------|------------|------|
| title        | 標題       | 文字 |
| artist       | 藝術家     | 文字 |
| album        | 專輯名     | 文字 |
| comment      | 註釋、評論 | 文字 |
| genre        | 風格       | 文字 |
| year         | 年度       | 日期 |
| track        | 曲目編號   | 數字 |
| album-artist | 專輯藝術家 | 文字 |
| composer     | 作曲家     | 文字 |
| track-total  | 曲目總數   | 數字 |
| disc         | 碟片編號   | 數字 |
| disc-total   | 碟片總數   | 數字 |
| copyright    | 版權       | 文字 |

## 幫助

**注意**：`music-tag-cli`可以快速地修改任何可以處理的音樂標籤檔案，但是如果操作不當，可能會造成您的音樂檔案標籤混亂，甚至清空標籤內容，如果您不能確定命令的結果，請一定先用`--dry-run`選項模擬執行，並仔細檢視日誌後再執行實際操作。

### 子命令

- view            檢視標籤
- set-const       設定標籤為固定值
- set-seq         設定標籤為序列值
- mod-num         加/減方式修改數字標籤
- mod-text-const  修改文字標籤，包括增加（add）/替換（replace）/刪除（remove）一個固定值，也可以執行截斷（truncate）
- mod-text-regex  以正則表示式替換的方式修改文字標籤
- conv-en         轉換英文文字標籤，轉小寫（lowercase）或者轉大寫（uppercase）
- conv-zh         轉換中文文字標籤，繁簡轉換
- conv-utf8       轉換文字標籤為UTF-8編碼
- help            幫助

### 示例

注意：全部示例中的檔案路徑都是以 **Unix/Linux/Mac** 方式表示的，如果您使用的是 **Windows** 系列作業系統，請使用Windows格式的路徑代替，例如："C:\some-path"。注意全部字元必須包含在UNICODE字符集中；如果路徑中存在空格，需要用`"`包裹。

- 通用選項

  以下選項適用於全部修改/設定標籤的命令。

  ```shell
      --dry-run                    如果設定為true，只顯示如何修改標籤，並不真正執行寫操作。並且資訊會記錄在日誌檔案中。這對於批次修改前的驗證，非常有用！
  -q, --quiet                      如果設定為true，只在控制檯顯示錯誤資訊
  ```

- view

  檢視標籤

  ```shell
  # 檢視檔案中的全部標籤
  music-tag-cli view ~/Music/Music

  # 檢視檔案中的全部標籤，包含屬性資訊
  music-tag-cli view --with-properties "~/Music/Music/John Denver"

  # 只檢視指定的標籤
  music-tag-cli view -t title,artist,album-artist "~/Music/Music/John Denver"
  ```

- set-const

   設定標籤為固定值， 如需更多資訊，請輸入 `music-tag-cli set-const -h`

  ```shell
  # 為指定的文字標籤設定固定值
  music-tag-cli set-const -t artist,album-artist "~/Music/Music/John Denver" text "John Denver"

  # 為指定的數字標籤設定固定值
  music-tag-cli set-const -t track-total "~/Music/Music/John Denver" num 10
  music-tag-cli set-const -t disc-number,disc-total "~/Music/Music/John Denver" num 1 --padding 1
  ```

- set-seq

  設定標籤為序列值，順序根據檔名排序（對於每個資料夾，序列值會重置）。部分引數：

  - start: 開始值，預設1
  - step:  增量值，預設1
  - padding: 格式佔位數，預設2

  ```shell
  # 設定曲目編號為序列值
  music-tag-cli set-seq -t track-number "~/Music/Music/John Denver"

  # 對現有標題追加序列值
  music-tag-cli set-seq -t title -m append "~/Music/Music/John Denver"
  ```

  如需更多資訊，請輸入 `music-tag-cli set-const -h`

- mod-num
  
  加/減方式修改數字標籤，但是修改後的值必須大於0。不會影響值為空的標籤。

  ```shell
  # 每個曲目編號加1
  music-tag-cli mod-num -t track-number -o 1 "~/Music/Music/John Denver"

  # 每個曲目編號減2
  music-tag-cli mod-num -t track-number -c decrease -o 2 "~/Music/Music/John Denver"
  ```

- mod-text-const

  修改文字標籤，包括增加（add）/替換（replace）/刪除（remove）一個固定值，也可以執行截斷（truncate）

  - add 增加

    ```shell
    # 註釋的第2個符，將加入` baisc`，例如，原來："1. from url"，修改後："1. basic from url"
    music-tag-cli mod-text-const -t comment "~/Music/Music/dir2" add -o 2 -a " basic"
    ```
  
  - remove 刪除

    ```shell
    # 刪除標題從結尾開始數的第4～5個字元
    music-tag-cli mod-text-const -t title "~/Music/Music/dir2" remove -d end -b 3 -e 5
    ```

  - replace 替換

    ```shell
    # 替換 `john denver` 為 `John Denver`
    music-tag-cli mod-text-const -t artist,album-artist "~/Music/Music/John Denver" replace -i --from "john denver" --to "John Denver"
    ```

  - truncate 截斷

    ```shell
    # 註釋標籤只保留前20個字元
    music-tag-cli mod-text-regex -t comment "~/Music/Music/dir2" -i --from "^(From)\s+" --to "something ${1}, "
    ```

- mod-text-regex

  以正則表示式替換的方式修改文字標籤，支援組的捕獲和全域性的大小寫敏感/不敏感方式。

  ```shell
  music-tag-cli mod-text-regex -i --from "^(From)\s+" "something ${1}, " "~/Music/Music/dir2"
  ```

- conv-en

  轉換英文文字標籤，轉小寫（lowercase）或者轉大寫（uppercase）

  ```shell
  # 把註釋轉為小寫
  music-tag-cli conv-en -p lowercase -t comment "~/Music/Music/dir2"
  
  # 把版權轉為大寫
  music-tag-cli conv-en -p uppercase -t copyright "~/Music/Music/dir2"
  ```

- conv-zh

  中文文字標籤繁簡轉換，如果要了解更多規則，請看[這裡](https://github.com/BYVoid/OpenCC)。

  ```shell
  # 把全部中文文字標籤轉為簡體
  music-tag-cli conv-zh -p t2s "~/Music/Music"
  
  # 把全部中文文字標籤轉為繁體
  music-tag-cli conv-zh -p s2t "~/Music/Music"
  ```

- conv-utf8

  轉換文字標籤為UTF-8編碼

  **注意**：請小心使用此功能，最好先檢查使用`--dry-run`的結果是否正確，因為有些轉換是**不可逆**的。

  ```shell
  # 把全部文字標籤從 Windows-1252 轉為 UTF-8
  music-tag-cli conv-utf8 -e Windows-1252 "~/Music/Music/old mp3"

  # 把全部文字標籤從 Shift_JIS 轉為 UTF-8
  music-tag-cli conv-utf8 -e shift_jis "~/Music/Music/日本語"
  ```

### 清除文字標籤

可以把標籤設定為長度為零字串。有下面幾種命令可以實現：

```shell
# 使用set-const命令，可以把任意文字標籤設定為長度為零字串，還可搭配`--set-when`選項使用
music-tag-cli set-const -t copyright ./path/file text ""

# 使用mod-text-const命令的remove子命令，可以把任意存在值的文字標籤設定為長度為零字串
music-tag-cli mod-text-const -t copyright ./path/file remove -b 0

# 使用mod-text-const命令的truncate子命令，可以把任意存在值的文字標籤設定為長度為零字串
music-tag-cli mod-text-const -t copyright ./path/file truncate -l 0
```

### 日誌

位於`./logs`目錄下

### 配置檔案

位於`~/.music-tag-cli.toml`。預設值如下：

```toml
# 可以是`trace` `debug` `info` `warn` `error`
log_level="info"
# 可以是`taglib` `audiotags`
tag_lib="taglib"
```

注意：

`audiotags`是一個原生的Rust軟體包，但是現在功能有限，還不能提供此應用的全部功能，目前只是試驗性的。

## License

[GPL-v3](LICENSE)
