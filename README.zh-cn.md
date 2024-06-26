# Music Tag Cli [English](README.md) | [中文繁體](README.zh-tw.md)

这是一个用于在命令行中编辑音乐标签的简单工具。您可以通过SSH、NFS或Samba（Windows共享文件夹）连接您的音乐服务器，然后您就可以使用它修改文件中的音乐标签。`music-tag-cli`可以批量修改文件夹及其子文件夹的文件，或仅仅修改单个文件。它可以插入或追加序列数，也可以将繁体中文转换为简体中文。`music-tag-cli`支持FLAC, APE, WAV, AIFF, WV, TTA, MP3, M4A, OGG, MPC, OPUS, WMA, DSF, DFF, MP4等音频文件格式。

## 安装

请阅读[安装指南](INSTALL.md)。如果您只是想在Windows平台下使用此程序，可以直接下载。

## 标签表

| 标签         | 名称       | 类型 |
|--------------|------------|------|
| title        | 标题       | 文本 |
| artist       | 艺术家     | 文本 |
| album        | 专辑名     | 文本 |
| comment      | 注释、评论 | 文本 |
| genre        | 风格       | 文本 |
| album-artist | 专辑艺术家 | 文本 |
| composer     | 作曲家     | 文本 |
| year         | 年度       | 数字 |
| date         | 日期       | 日期 |
| track-number | 曲目编号   | 数字 |
| track-total  | 曲目总数   | 数字 |
| disc-number  | 盘片编号   | 数字 |
| disc-total   | 盘片总数   | 数字 |
| copyright    | 版权       | 文本 |
| lyrics       | 歌詞       | 文本 |

## 帮助

**注意**：`music-tag-cli`可以快速地修改任何可以处理的音乐标签文件，但是如果操作不当，可能会造成您的音乐文件标签混乱，甚至清空标签内容，如果您不能确定命令的结果，请一定先用`--dry-run`选项模拟执行，并仔细查看日志后再执行实际操作。

### 子命令

| 子命令         | 说明                                                                                               |
|----------------|----------------------------------------------------------------------------------------------------|
| view           | 查看标签                                                                                           |
| clear          | 清除标签的值                                                                                       |
| conv-en        | 转换英文文本标签，转小写（lowercase）、大写（uppercase）或者首字母大写（titlecase）                |
| conv-utf8      | 转换文本标签为UTF-8编码                                                                            |
| conv-zh        | 转换中文文本标签，繁简转换                                                                         |
| exp            | 把标签导出到文件                                                                                   |
| imp            | 从文件导入标签                                                                                     |
| lrc            | 导出歌词到到`.lrc`文件，或者从`.lrc`文件导入歌词                                                   |
| mod-num        | 加/减方式修改数字标签                                                                              |
| mod-text-const | 修改文本标签，包括增加（add）/替换（replace）/删除（remove）一个固定值，也可以执行截断（truncate） |
| mod-text-regex | 以正则表达式替换的方式修改文本标签                                                                 |
| set-const      | 设置标签为固定值                                                                                   |
| set-name       | 从文件名设置标签                                                                                   |
| set-seq        | 设置标签为序列值                                                                                   |
| ren            | 用标签值重命名文件                                                                                 |
| help           | 帮助                                                                                               |

### 示例

注意：全部示例中的文件路径都是以 **Unix/Linux/Mac** 方式表示的，如果您使用的是 **Windows** 系列操作系统，请使用Windows格式的路径代替，例如："C:\some-path"。注意全部字符必须包含在UNICODE字符集中；如果路径中存在空格，需要用`"`包裹。

#### 通用选项

以下选项适用于全部修改/设置/转换标签的命令。即只有`exp`、`view`命令**不能**使用。

```shell
    --dry-run                    如果设置为true，只显示如何修改标签，并不真正执行写操作。并且信息会记录在日志文件中。这对于批量修改前的验证，非常有用！
-q, --quiet                      如果设置为true，只在控制台显示错误信息
```

以下选项可以用于**除**`imp`、`set-seq`之外的全部命令。

```shell
    --where <WHERE_CLAUSE>
          `Where`子句用于谓词判定。这与SQL了类似，支持`NOT` `AND` `OR`逻辑操作符和`=` `<` `<=` `>` `>=` `!=` `<>`比较操作符，也可以使用`LIKE`，它支持`%` `_`通配符，`ILIKE`与之类似，但它是忽略大小写的。注意在字符串中的单引号`'`字符需要使用转义，即`''`，和SQL字符串类似。
```
  
注意：`=` `!=` `<>`用于文本标签时是大小写敏感的。

示例

```shell
# 只显示曲目编号在10～100之间的标签
music-tag-cli view "~/Music/Music/John Denver" --where "track-number >= 10 and track-number <= 100"
```

#### view

查看标签

```shell
# 查看文件中的全部标签
music-tag-cli view ~/Music/Music

# 查看文件中的全部标签，包含属性信息
music-tag-cli view --with-properties "~/Music/Music/John Denver"

# 只查看指定的标签
music-tag-cli view -t title,artist,album-artist "~/Music/Music/John Denver"
```

#### clear

清除标签的值。

**注意**：请小心使用此功能，最好先检查使用`--dry-run`的结果是否正确，因为清除后**不可恢复**。

```shell
# 清除注释和版权
music-tag-cli clear -t comment,copyright "~/Music/Music"
```

#### conv-en

转换英文文本标签，转小写（lowercase）、大写（uppercase）或者首字母大写（titlecase）。

```shell
# 把标题转为首字母大写
music-tag-cli conv-en -p titlecase -t title "~/Music/Music/dir2"

# 把注释转为小写
music-tag-cli conv-en -p lowercase -t comment "~/Music/Music/dir2"

# 把版权转为大写
music-tag-cli conv-en -p uppercase -t copyright "~/Music/Music/dir2"
```

#### conv-utf8

转换文本标签为UTF-8编码
  
**注意**：请小心使用此功能，最好先检查使用`--dry-run`的结果是否正确，因为有些转换是**不可逆**的。

```shell
# 把全部文本标签从 Windows-1252 转为 UTF-8
music-tag-cli conv-utf8 -e Windows-1252 "~/Music/Music/old mp3"

# 把全部文本标签从 Shift_JIS 转为 UTF-8
music-tag-cli conv-utf8 -e shift_jis "~/Music/Music/日本語"
```

#### conv-zh

中文文本标签繁简转换，如果要了解更多规则，请看[这里](https://github.com/BYVoid/OpenCC)。

```shell
# 把全部中文文本标签转为简体
music-tag-cli conv-zh -p t2s "~/Music/Music"

# 把全部中文文本标签转为繁体
music-tag-cli conv-zh -p s2t "~/Music/Music"
```

#### exp

把标签导出到JSON文件。
  
如果输出文件已经存在，程序将退出。

```shell
# 简单导出
music-tag-cli exp -o "../backup/all.json" "~/Music/Music"

# 导出，包括属性信息
music-tag-cli exp -o "../backup/all.json" --with-properties "~/Music/Music"
```

#### imp

从JSON文件导入标签。（不会导入`props`即属性信息。）
  
如果没有设置`--dry-run`选项，当首次发现JSON元素异常时，程序会中断，但是之前的元素会保存成功。

```shell
# 简单导入
music-tag-cli imp "../backup/all.json"

# 导入，文件路径`path`会拼接到`~/Music/Music`之后
music-tag-cli imp -b "~/Music/Music" "../backup/all.json"
```

#### lrc

导出歌词到`.lrc`文件，或者从`.lrc`文件导入歌词。歌词文件和音乐文件主干名相同，后缀必须是`.lrc`。
  
导出时，如果存在歌词文件不会覆盖。导入时，如果不存在歌词文件会忽略。

导出、导入时都可以指定文件编码，默认是`UTF-8`。

```shell
# 导出歌词
music-tag-cli lrc -d export "~/Music/Music/"

# 使用Windows-1252编码导出
music-tag-cli lrc -d export -e Windows-1252 "~/Music/Music/"

# 导入歌词，使用Windwos-1252编码
music-tag-cli lrc -d import -e Windows-1252 -b "~/Music/Music"
```

#### mod-num
  
加/减方式修改数字标签，但是修改后的值必须大于0。不会影响值为空的标签。

```shell
# 每个曲目编号加1
music-tag-cli mod-num -t track-number -o 1 "~/Music/Music/John Denver"

# 每个曲目编号减2
music-tag-cli mod-num -t track-number -c decrease -o 2 "~/Music/Music/John Denver"
```

#### mod-text-const

修改文本标签，包括增加（add）/替换（replace）/删除（remove）一个固定值，也可以执行截断（truncate）。

##### add 增加

```shell
# 注释的第2个符，将加入` baisc`，例如，原来："1. from url"，修改后："1. basic from url"
music-tag-cli mod-text-const -t comment "~/Music/Music/dir2" add -o 2 -a " basic"
```
  
##### remove 删除

```shell
# 删除标题从结尾开始数的第4～5个字符
music-tag-cli mod-text-const -t title "~/Music/Music/dir2" remove -d end -b 3 -e 5
```

##### replace 替换

```shell
# 替换 `john denver` 为 `John Denver`
music-tag-cli mod-text-const -t artist,album-artist "~/Music/Music/John Denver" replace -i --from "john denver" --to "John Denver"
```

##### truncate 截断

```shell
# 注释标签只保留前20个字符
music-tag-cli mod-text-const -t comment "~/Music/Music/dir2" truncate -l 20
```

#### mod-text-regex

以正则表达式替换的方式修改文本标签，支持组的捕获和全局的大小写敏感/不敏感方式。

注意：**不**支持前向、后向断言！

```shell
# Windows CMD
music-tag-cli mod-text-regex -t comment "C:\Music\Music\dir2" -i --from "^(From)\s+" --to "something ${1}, "

# Linux/Mac, `$`需要被转义为`\$`
music-tag-cli mod-text-regex -t comment "~/Music/Music/dir2" -i --from "^(From)\s+" --to "something \${1}, "
```

#### set-const

设置标签为固定值， 如需更多信息，请输入 `music-tag-cli set-const -h`

```shell
# 为指定的文本标签设置固定值
music-tag-cli set-const -t artist,album-artist "~/Music/Music/John Denver" text "John Denver"

# 为指定的数字标签设置固定值
music-tag-cli set-const -t track-total "~/Music/Music/John Denver" num 10
music-tag-cli set-const -t disc-number,disc-total "~/Music/Music/John Denver" num 1 --padding 1
```

#### set-name

从文件名设置标签（只使用文件名主干，不包含路径和扩展名）。

```shell
# Windows CMD
music-tag-cli set-name --template "${track-number} - ${title} - ${artist}" "C:\Music\Music\dir"

# Linux/Mac, `$`需要被转义为`\$`
music-tag-cli set-name --template "\${track-number} - \${title} - \${artist}" "~/Music/Music/John Denver"
```

##### set-seq

设置标签为序列值，顺序根据文件名排序（对于每个文件夹，序列值会重置）。部分参数：

- start: 开始值，默认1
- step:  增量值，默认1
- padding: 格式占位数，默认2

```shell
# 设置曲目编号为序列值
music-tag-cli set-seq -t track-number "~/Music/Music/John Denver"

# 对现有标题追加序列值
music-tag-cli set-seq -t title -m append "~/Music/Music/John Denver"
```

如需更多信息，请输入 `music-tag-cli set-const -h`

#### ren
  
使用标签值重命名文件（只修改文件名主干，路径和扩展名保持不变）。

如果标签值为空，会使用空字符串代替。如果全部为空字符串，不会执行重命名。

```shell
# Windows CMD
music-tag-cli ren --template "${track-number}.${title} - ${artist}" "C:\Music\Music\dir"

# Linux/Mac, `$`需要被转义为`\$`
music-tag-cli ren --template "\${track-number}.\${title} - \${artist}" "~/Music/Music/John Denver"
```

### 清除文本标签的多种选择

**推荐**使用`clear`命令。

```shell
music-tag-cli clear -t copyright ./set-const/001.dsf
```

另外下面几个命令可以把标签值设置为长度为零字符串，但都不如`clear`命令干净彻底。

```shell
# 使用set-const命令，可以把任意文本标签设置为长度为零字符串，还可搭配`--set-when`选项使用
music-tag-cli set-const -t copyright ./set-const/001.dsf text ""

# 使用mod-text-const命令的remove子命令，可以把任意存在值的文本标签设置为长度为零字符串
music-tag-cli mod-text-const -t copyright ./mod-text-const/001.dsf remove -b 0

# 使用mod-text-const命令的truncate子命令，可以把任意存在值的文本标签设置为长度为零字符串
music-tag-cli mod-text-const -t copyright ./mod-text-const/001.dsf truncate -l 0
```

### 日志

位于`./logs`目录下

### 配置文件

位于`~/.music-tag-cli.toml`。默认值如下：

```toml
# 可以是`trace` `debug` `info` `warn` `error`
log_level="info"
# 可以是`taglib` `audiotags`
tag_lib="taglib"
```

注意：

`audiotags`是一个原生的Rust软件包，但是现在功能有限，还不能提供此应用的全部功能，目前只是试验性的。

## License

[GPL-v3](LICENSE)
