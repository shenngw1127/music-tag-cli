# Music Tag Cli [中文简体](README.zh-cn.md) | [中文繁體](README.zh-tw.md)

This is a simple tool for editing music tags in command line. You can connect to a private music server via SSH, NFS or Samba (Windows shared folder), and then you can use this tool to modify the music tags in the files. `music-tag-cli` can batch modify files in a folder and its subfolders, or just modify a single file. It was able to insert or append sequence numbers and also convert Traditional Chinese to Simplified Chinese. `music-tag-cli` supports FLAC, APE, WAV, AIFF, WV, TTA, MP3, M4A, OGG, MPC, OPUS, WMA, DSF, DFF, MP4 audio file formats.

## Install

Please read the [install guide](INSTALL.md). You can directly download the binary exe if you only wish run it on Windows platform.

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

## Help

**Note**: Using `music-tag-cli`, you can quickly modify any music tag file that can be processed, but if done improperly, it may cause confusion in the tags of your music files, or even clear the tag content. If you are not sure of any command result, please use the `--dry-run` option to simulate execution, and carefully check the logs before performing the actual operation.

### Subcommand

- view            View tags.
- set-const       Set a Constant value for tags.
- set-seq         Set Sequence value for tags
- mod-num         Modify numeric tags by increase / decrease an integer.
- mod-text-const  Modify text tags by add / replace / remove a Constant value, also could do truncate.
- mod-text-regex  Modify text tags by REGEX replace.
- conv-en         Convert text tags in English between lowercase, uppercase and titlecase.
- conv-zh         Convert text tags in Chinese between Traditional and Simplified.
- conv-utf8       Convert text tags to UTF-8 encoding.
- help            Print this message or the help of the given subcommand(s)

### EXAMPLES

Note: All file path in examples is **Unix/Linux/Mac** mode, if you use **Windows** series, please use path in Windows format. e.g. "C:\some-path". Please make sure all characters must be in UNICODE character set. It must surrund by `"` if it contains space character.

- General Options

  These options are available for all command to modify / set / conv tags.

  ```shell
      --dry-run                    Only show how to modify tags, but do NOT write any file, if it was set as true.
  -q, --quiet                      Only show error in console, if it was set as true.
  ```

  This option are available for all commands except `set-seq`.

  ```shell
      --where <WHERE_CLAUSE>
          `Where` clause for prediction. It is like SQL, supported `NOT` `AND` `OR` logic operators, `=` `<` `<=` `>` `>=` `!=` `<>` comparison operators, `LIKE` also is supported with `%` `_` wildcards, `ILIKE` is same but case insensitive. Note: `'` should be escaped as `''` like in SQL string.
  ```
  
  Note: `=` `!=` `<>` for text tag is case sensitive.

  for example:

  ```shell
  # only view tags that's track-number between 10 - 100
  music-tag-cli view "~/Music/Music/John Denver" --where "track-number >= 10 and track-number <= 100"
  ```

- view

  View tags.

  ```shell
  # View all tags in all files
  music-tag-cli view ~/Music/Music

  # View all tags with properties
  music-tag-cli view --with-properties "~/Music/Music/John Denver"

  # View only specified tags
  music-tag-cli view -t title,artist,album-artist "~/Music/Music/John Denver"
  ```

- set-const

  Set a Constant value for tags, for more options, please type `music-tag-cli set-const -h`

  ```shell
  # Set a constant for some text tags in files
  music-tag-cli set-const -t artist,album-artist "~/Music/Music/John Denver" text "John Denver"

  # Set a constant for some numeric tags in files
  music-tag-cli set-const -t track-total "~/Music/Music/John Denver" num 10
  music-tag-cli set-const -t disc-number,disc-total "~/Music/Music/John Denver" num 1 --padding 1
  ```

- set-seq

  Set a sequence for some tag in files, sorted by file name (each folder reset the sequence). Some arguments:

  - start: 1 (defaut)
  - step: 1 (defaut)
  - padding: 2 (defaut)

  ```shell
  # Set numeric track-number as sequence
  music-tag-cli set-seq -t track-number "~/Music/Music/John Denver"

  # Append title to sequece
  music-tag-cli set-seq -t title -m append "~/Music/Music/John Denver"
  ```

  for more options, please type `music-tag-cli set-const -h`

- mod-num
  
  Modify numeric tags by increase / decrease an integer, and must be great than 0. It will **NOT** affect empty tags.

  ```shell
  # each track-number plus 1
  music-tag-cli mod-num -t track-number -o 1 "~/Music/Music/John Denver"

  # each track-number subset 2
  music-tag-cli mod-num -t track-number -c decrease -o 2 "~/Music/Music/John Denver"
  ```

- mod-text-const

  Modify text tags by add / replace / remove a Constant value, also could do truncate.

  - add

    ```shell
    # comment will at first 2 charcters insert ` baisc`, e.g. orininal: "1. from url", new: "1. basic from url"
    music-tag-cli mod-text-const -t comment "~/Music/Music/dir2" add -o 2 -a " basic"
    ```
  
  - remove

    ```shell
    # remove the character position 4 to 5 from the end of title
    music-tag-cli mod-text-const -t title "~/Music/Music/dir2" remove -d end -b 3 -e 5
    ```

  - replace

    ```shell
    # replace `john denver` to `John Denver`
    music-tag-cli mod-text-const -t artist,album-artist "~/Music/Music/John Denver" replace -i --from "john denver" --to "John Denver"
    ```

  - truncate

    ```shell
    # comment will be truncated as 20 characters from beginning
    music-tag-cli mod-text-const -t comment "~/Music/Music/dir2" truncate -l 20
    ```

- mod-text-regex

  Modify text tags by REGEX replace, support group capture, and global case sensitive/insensitive.

  ```shell
  music-tag-cli mod-text-regex -t comment "~/Music/Music/dir2" -i --from "^(From)\s+" --to "something ${1}, "
  ```

- conv-en

  Convert text tags in English between lowercase, uppercase and titlecase.

  ```shell
  # Convert all title tag to titlecase
  music-tag-cli conv-en -p titlecase -t title "~/Music/Music/dir2"

  # Convert all comment tag to lowercase
  music-tag-cli conv-en -p lowercase -t comment "~/Music/Music/dir2"
  
  # Convert all copyright tag to uppercase
  music-tag-cli conv-en -p uppercase -t copyright "~/Music/Music/dir2"
  ```

- conv-zh

  Convert text tags in Chinese between Traditional and Simplified, for more profiles, please see [here](https://github.com/BYVoid/OpenCC).

  ```shell
  # Convert all text tag from Traditional Chinese to Simple Chinese
  music-tag-cli conv-zh -p t2s "~/Music/Music"
  
  # Convert all text tag from Simple Chinese to Traditional Chinese
  music-tag-cli conv-zh -p s2t "~/Music/Music"
  ```

- conv-utf8

  Convert text tags to UTF-8 encoding.

  **Note**: Please use this function with caution. It is the best to check whether the result of using `--dry-run` is correct at first, because some encoding conversion was **irreversible**.

  ```shell
  # Convert all text tag from Windows-1252 to UTF-8 encoding
  music-tag-cli conv-utf8 -e Windows-1252 "~/Music/Music/old mp3"

  # Convert all text tag from Shift_JIS to UTF-8 encoding
  music-tag-cli conv-utf8 -e shift_jis "~/Music/Music/日本語"
  ```

### Clear text tags

You could use some command set the text tags' value to zero length string:

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

Location: `~/.music-tag-cli.toml`. Deault value as below:

```toml
# `trace` `debug` `info` `warn` `error`
log_level="info"
# `taglib` `audiotags`
tag_lib="taglib"
```

Note:

`audiotags` is an carate in Rust, but it has no enough function for this application now. It is only experimental.

## License

[GPL-v3](LICENSE)
