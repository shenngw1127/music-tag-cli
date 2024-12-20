# Music Tag Cli [中文简体](README.zh-cn.md) | [中文繁體](README.zh-tw.md)

This is a simple tool for editing music tags in command line. You can connect to a private music server via SSH, NFS or Samba (Windows shared folder), and then you can use this tool to modify the music tags in the files. `music-tag-cli` can batch modify files in a folder and its subfolders, or just modify a single file. It was able to insert or append sequence numbers and also convert Traditional Chinese to Simplified Chinese. `music-tag-cli` supports FLAC, APE, WAV, AIFF, WV, TTA, MP3, M4A, OGG, MPC, OPUS, WMA, DSF, DFF, MP4 audio file formats.

## Install

Please read the [installation guide](INSTALL.md). You can directly download the binary exe if you only wish run it on Windows platform.

## Tag list

| Tag          | Name         | Type    |
|--------------|--------------|---------|
| title        | title        | text    |
| artist       | artist       | text    |
| album        | album title  | text    |
| comment      | comment      | text    |
| genre        | genre        | text    |
| album-artist | album artist | text    |
| composer     | composer     | text    |
| year         | year         | numeric |
| date         | date         | date    |
| track-number | track number | numeric |
| track-total  | track total  | numeric |
| disc-number  | disc number  | numeric |
| disc-total   | disc total   | numeric |
| copyright    | copyright    | text    |
| lyrics       | lyrics       | text    |

## Help

**Note**: Using `music-tag-cli`, you can quickly modify any music tag file that can be processed, but if done improperly, it may cause confusion in the tags of your music files, or even clear the tag content. If you are not sure of any command result, please use the `--dry-run` option to simulate execution, and carefully check the logs before performing the actual operation.

### Subcommand

| Subcommand     | Description                                                                                   |
|----------------|-----------------------------------------------------------------------------------------------|
| view           | View tags                                                                                     |
| clear          | Remove value of tags                                                                          |
| conv-en        | Convert text tags in English between lowercase / uppercase / tilecase.                        |
| conv-utf8      | Convert text tags to UTF-8 encoding.                                                          |
| conv-zh        | Convert text tags in Chinese characters between Traditional / Simplified /Japanese Shinjitai. |
| exp            | Export tags to file.                                                                          |
| imp            | Import tags from file.                                                                        |
| lrc            | Export / Import lyrics to / from `.lrc` file.                                                 |
| mod-num        | Modify numeric tags by increase/decrease an integer.                                          |
| mod-text-const | Modify text tags by add/replace/remove a constant value,also could truncate.                  |
| mod-text-regex | Modify text tags by REGEX replace.                                                            |
| set-const      | Set a constant value for tags.                                                                |
| set-name       | Set tags from filename.                                                                       |
| set-seq        | Set sequence value for tags.                                                                  |
| ren            | Rename file with tags.                                                                        |
| help           | Print this message or the help of the given subcommand(s)                                     |

### EXAMPLES

Note: All file path in examples is **Unix/Linux/Mac** mode, if you use **Windows** series, please use path in Windows format. e.g. "C:\some-path". Please make sure all characters must be in UNICODE character set. It must surround by `"` if it contains space character.

#### General Options

These options are available for all command to modify / set / conv tags. (Except `exp`, `view` commands.)

```shell
    --dry-run                    Only show how to modify tags, but do NOT write any file, if it was set as true.
-q, --quiet                      Only show error in console, if it was set as true.
```

This option are available for all commands **except** `imp`, `set-seq`.

```shell
    --where <WHERE_CLAUSE>
        `Where` clause for prediction. It is like SQL, supported `NOT` `AND` `OR` logic operators, `=` `<` `<=` `>` `>=` `!=` `<>` comparison operators, `LIKE` also is supported with `%` `_` wildcards, `ILIKE` is same but case insensitive. Note: `'` should be escaped as `''` like in SQL string.
```

Note: `=` `!=` `<>` for text tag is case-sensitive.

for example:

```shell
# only view tags that's track-number between 10 - 100
music-tag-cli view "~/Music/Music/John Denver" --where "track-number >= 10 and track-number <= 100"
```

#### view

View tags.

```shell
# View all tags in all files
music-tag-cli view ~/Music/Music

# View all tags with properties
music-tag-cli view --with-properties "~/Music/Music/John Denver"

# View only specified tags
music-tag-cli view -t title,artist,album-artist "~/Music/Music/John Denver"
```

#### clear

Remove value of tags.

**Note**: Please use this feature with caution, and it is best to check whether the results are correct using `--dry-run` first, because clearing is **irreversible**.

```shell
# Clear comment and copyright
music-tag-cli clear -t comment,copyright "~/Music/Music"
```

#### conv-en

Convert text tags in English between lowercase, uppercase and titlecase.

```shell
# Convert all title tag to titlecase
music-tag-cli conv-en -p titlecase -t title "~/Music/Music/dir2"

# Convert all comment tag to lowercase
music-tag-cli conv-en -p lowercase -t comment "~/Music/Music/dir2"

# Convert all copyright tag to uppercase
music-tag-cli conv-en -p uppercase -t copyright "~/Music/Music/dir2"
```

#### conv-utf8

Convert text tags to UTF-8 encoding.

**Note**: Please use this function with caution. It is the best to check whether the result of using `--dry-run` is correct at first, because some encoding conversion was **irreversible**.

```shell
# Convert all text tag from Windows-1252 to UTF-8 encoding
music-tag-cli conv-utf8 -e Windows-1252 "~/Music/Music/old mp3"

# Convert all text tag from Shift_JIS to UTF-8 encoding
music-tag-cli conv-utf8 -e shift_jis "~/Music/Music/日本語"
```

#### conv-zh

Convert text tags in Chinese between Traditional and Simplified, for more profiles, please see [here](https://github.com/BYVoid/OpenCC).

```shell
# Convert all text tag from Traditional Chinese to Simple Chinese
music-tag-cli conv-zh -p t2s "~/Music/Music"

# Convert all text tag from Simple Chinese to Traditional Chinese
music-tag-cli conv-zh -p s2t "~/Music/Music"
```

#### exp

Export tags to file in JSON format.

Program will exit if output file exists.

```shell
# Export basic
music-tag-cli exp -o "../backup/all.json" "~/Music/Music"

# Export with properties
music-tag-cli exp -o "../backup/all.json" --with-properties "~/Music/Music"
```

#### imp

Import tags from JSON file. (`props` was NOT processed.)

It will break when first JSON element validate fail, but all before it will be saved if it does NOT set `--dry-run` option.

```shell
# Import basic
music-tag-cli imp "../backup/all.json"

# Import it, path will be joined after `~/Music/Music`
music-tag-cli imp -b "~/Music/Music" "../backup/all.json"
```

#### lrc

Export lyrics to a `.lrc` file, or import lyrics from a `.lrc` file. The lyrics file has the same name as the music file, and the extension must be `.lrc`.

When exporting, if the lyrics file exists, it will not be overwritten. When importing, if the lyrics file does not exist, it will be ignored.

You can specify the file encoding when exporting or importing. The default is `UTF-8`.

```shell
# Export lyrics
music-tag-cli lrc -d export "~/Music/Music/"

# Export lyrics using Windows-1252 encoding
music-tag-cli lrc -d export -e Windows-1252 "~/Music/Music/"

# Import lyrics, using Windows-1252 encoding
music-tag-cli lrc -d import -e Windows-1252 -b "~/Music/Music"
```

#### mod-num
  
Modify numeric tags by increase / decrease an integer, and it must be greater than 0. It will **NOT** affect empty tags.

```shell
# each track-number plus 1
music-tag-cli mod-num -t track-number -o 1 "~/Music/Music/John Denver"

# each track-number subset 2
music-tag-cli mod-num -t track-number -c decrease -o 2 "~/Music/Music/John Denver"
```

#### mod-text-const

Modify text tags by add / replace / remove a Constant value, also could truncate.

##### add

```shell
# comment will at first 2 characters insert ` basic`, e.g. original: "1. from url", new: "1. basic from url"
music-tag-cli mod-text-const -t comment "~/Music/Music/dir2" add -o 2 -a " basic"
```

##### remove

```shell
# remove the character position 4 to 5 from the end of title
music-tag-cli mod-text-const -t title "~/Music/Music/dir2" remove -d end -b 3 -e 5
```

##### replace

```shell
# replace `john denver` to `John Denver`
music-tag-cli mod-text-const -t artist,album-artist "~/Music/Music/John Denver" replace -i --from "john denver" --to "John Denver"
```

##### truncate

```shell
# comment will be truncated as 20 characters from beginning
music-tag-cli mod-text-const -t comment "~/Music/Music/dir2" truncate -l 20
```

#### mod-text-regex

Modify text tags by REGEX replace, support group capture, and global case-sensitive / case-insensitive.

Note: Lookahead / Lookbehind assertion is **NOT** supported!

```shell
# Windows CMD
music-tag-cli mod-text-regex -t comment "C:\Music\Music\dir2" -i --from "^(From)\s+" --to "something ${1}, "

# Linux/Mac, `$` must be escaped as `\$`
music-tag-cli mod-text-regex -t comment "~/Music/Music/dir2" -i --from "^(From)\s+" --to "something \${1}, "
```

#### set-const

Set a Constant value for tags, for more options, please type `music-tag-cli set-const -h`

```shell
# Set a constant for some text tags in files
music-tag-cli set-const -t artist,album-artist "~/Music/Music/John Denver" text "John Denver"

# Set a constant for some numeric tags in files
music-tag-cli set-const -t track-total "~/Music/Music/John Denver" num 10
music-tag-cli set-const -t disc-number,disc-total "~/Music/Music/John Denver" num 1 --padding 1
```

#### set-name

Set tags from filename (only use file stem, WITHOUT path and extension)

```shell
# Windows CMD
music-tag-cli set-name --template "${track-number} - ${title} - ${artist}" "C:\Music\Music\dir"

# Linux/Mac, `$` must be escaped as `\$`
music-tag-cli set-name --template "\${track-number} - \${title} - \${artist}" "~/Music/Music/John Denver"
```

#### set-seq

Set a sequence for some tag in files, sorted by file name (each folder reset the sequence). Some arguments:

- start: 1 (default)
- step: 1 (default)
- padding: 2 (default)

```shell
# Set numeric track-number as sequence
music-tag-cli set-seq -t track-number "~/Music/Music/John Denver"

# Append title to sequence
music-tag-cli set-seq -t title -m append "~/Music/Music/John Denver"
```

for more options, please type `music-tag-cli set-const -h`

#### ren
  
Rename filename with tags (only modify the file stem, WITHOUT path and extension).

It will be empty string if not found value of the tag. All value is empty will NOT rename it.

```shell
# Windows CMD
music-tag-cli ren --template "${track-number}.${title} - ${artist}" "C:\Music\Music\dir"

# Linux/Mac, `$` must be escaped as `\$`
music-tag-cli ren --template "\${track-number}.\${title} - \${artist}" "~/Music/Music/John Denver"
```

### Multiple ways for clearing text tags

**Recommended** use the `clear` command.

```shell
music-tag-cli clear -t copyright ./set-const/001.dsf
```

The following commands can also set the tag value to a zero-length string, but none of them are as clean and thorough as the `clear` command.

```shell
# set-const, also could use `--set-when` option
music-tag-cli set-const -t copyright ./set-const/001.dsf text ""

# mod-text-const remove subcommand
music-tag-cli mod-text-const -t copyright ./mod-text-const/001.dsf remove -b 0

# mod-text-const truncate subcommand
music-tag-cli mod-text-const -t copyright ./mod-text-const/001.dsf truncate -l 0
```

### Log

Log files location: `./logs`.

### Configuration file

Location: `~/.music-tag-cli.toml`. Default value as below:

```toml
# `trace` `debug` `info` `warn` `error`
log_level="info"
# `taglib` `audiotags`
tag_lib="taglib"
```

Note:

`audiotags` is a crate in Rust, but it has no enough function for this application now. It is only experimental.

## License

[GPL-v3](LICENSE)
