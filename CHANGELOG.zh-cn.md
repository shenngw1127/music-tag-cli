# 更新日志 [English](CHANGELOG.md) | [中文繁體](CHANGELOG.zh-tw.md)

## 1.0.4

- (文档) 修正拼写错误
- (功能) `lrc -d export`、`exp`和`ren`命令增加`--filename-exist-policy`选项（短名为`-x`），可以是`skip`（默认）, `keep-both`或`overwrite`
- (修复) `WHERE`表达式可以使用`lyrics`字段

## 1.0.3

- (文档) 增加`taglib-rust`的版本兼容性列表
- (功能) 增加命令`clear`
- (功能) 增加命令`lrc`
- (功能) 增加文本标签`lyrics`
- (修复) `exp`导出JSON文件时没有转义`\`

## 1.0.2

- (重构) 创建`Action`，然后调用方法`do_any(&mut self)`
- (文档) 正则表达式替换不支持前向/后向断言
- (修复) 正则表达式替换只替换了第一个出现的
- (功能) 增加命令`imp`
- (功能) 增加命令`exp`
- (功能) 增加命令`ren`
- (功能) 增加命令`set-name`

## 1.0.1

- (修复) 在设置`disc-total`之后设置`disc-number`，`disc-total`会消失
- (修复) 日志记录最后的错误信息
- (功能) `conv-en`命令支持标题大小写(`titlecase`)模式
- (功能) 为全部除`set-seq`的命令增加`--where`谓词判断

## 1.0.0

初始版本
