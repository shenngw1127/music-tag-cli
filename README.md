tw2s
====================

[![CI](https://github.com/magiclen/tw2s/actions/workflows/ci.yml/badge.svg)](https://github.com/magiclen/tw2s/actions/workflows/ci.yml)

A simple tool for converting Traditional Chinese(TW) to Simple Chinese.

## Help

```
EXAMPLES:
tw2s                   # Convert each of input lines from Traditional Chinese to Simple Chinese
tw2s cht.txt chs.txt   # Convert cht.txt (in Traditional Chinese) to chs.txt (in Simple Chinese)
tw2s a.cht.txt         # Convert a.cht.txt (in Traditional Chinese) to a.chs.txt (in Simple Chinese)

Usage: tw2s [OPTIONS] [TW_PATH] [S_PATH]

Arguments:
  [TW_PATH]  Assign the path of your Traditional Chinese document. It should be a file path
  [S_PATH]   Assign the path of your Simple Chinese document. It should be a file path

Options:
  -f, --force    Force to output if the output file exists
  -h, --help     Print help
  -V, --version  Print version
```

## License

[MIT](LICENSE)