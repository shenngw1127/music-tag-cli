# 更新日誌 [English](CHANGELOG.md) | [中文简体](CHANGELOG.zh-cn.md)

## 1.0.4

- (檔案) 修正拼寫錯誤
- (功能) `lrc -d export`、`exp`和`ren`命令增加`--filename-exist-policy`選項（短名為`-x`），可以是`skip`（預設）, `keep-both`或`overwrite`
- (修復) `WHERE`表示式可以使用`lyrics`欄位

## 1.0.3

- (檔案) 增加`taglib-rust`的版本相容性列表
- (功能) 增加命令 `clear`
- (功能) 增加命令 `lrc`
- (功能) 增加文字標籤`lyrics`
- (修復)  `exp`匯出JSON檔案時沒有跳脫`\`

## 1.0.2

- (重構) 創建`Action`，然後調用方法`do_any(&mut self)`
- (文檔) 正規表示式替換不支援正向/反向預查
- (修復) 正規表示式替換只替換了第一個出現的
- (功能) 增加命令`imp`
- (功能) 增加命令`exp`
- (功能) 增加命令`ren`
- (功能) 增加命令`set-name`

## 1.0.1

- (修復) 在設定`disc-total`之後設定`disc-number`，`disc-total`會消失
- (修復) 日誌未記錄最後的錯誤資訊
- (功能) `conv-en`命令支援標題大小寫(`titlecase`)模式
- (功能) 為全部除`set-seq`的命令增加`--where`謂詞判斷

## 1.0.0

初始版本
