# Change Log [中文简体](CHANGELOG.zh-cn.md) | [中文繁體](CHANGELOG.zh-tw.md)

## 1.0.4

- (doc) spell check for markdown
- (feat) `lrc -d export`, `exp` and `ren` add `--filename-exist-policy` (short as `-x`) option, could be `skip`(default), `keepBoth` or `overwrite`
- (fix) add `lyrics` in `WHERE` field 

## 1.0.3

- (doc) add compatible list with taglib-rust
- (feat) add command `clear`
- (feat) add command `lrc`
- (feat) add `lyrics` as text tag
- (fix) exp JSON not escape `\`

## 1.0.2

- (refactor) create `Action` then call method `do_any(&mut self)`
- (doc) regex replace not support lookahead / lookbehind
- (fix) regex replace only first
- (feature) add command `imp`
- (feature) add command `exp`
- (feature) add command `ren`
- (feature) add command `set-name`

## 1.0.1

- (fix) set disc-number after setting disc-total, disc-total will be lost
- (fix) log the final error
- (feature) command `conv-en` support titlecase
- (feature) add `--where` prediction for all commands except `set-seq`

## 1.0.0

Initial version
