# Music Tag Cli Install

[[_TOC_]]

## Pre requirement

- OS: Unix/Linux/Mac/Windows
- taglib (it must be compiled install, so you need)
  - Compile tools
    - CMake
    - GNU C/C++ or others
  - Dependencies
    - zlib v1.x
    - cppunit (optional)
- Compile tools
  - Rust chain: v1.70 or above
  - pkg-config
- Other Dependencies
  - opencc

Note: You can NOT install taglib by homebrew or apt-get, because that only 1.xx was in the repo. So you must download source code and install taglib via compile it by your self.

## Linux/Mac Steps

### Prepare compile taglib

#### Install compiler

Ubuntu

```shell
sudo apt-get install -y build-essential
```

Mac

```shell
xcode-select --install
```

#### Install `CMake`

Mac

```shell
brew install cmake
```

Ubuntu

```shell
# remove old version
sudo apt-get remove -y cmake

# Obtain a copy of kitware's signing key.
wget -O - https://apt.kitware.com/keys/kitware-archive-latest.asc 2>/dev/null | gpg --dearmor - | sudo tee /etc/apt/trusted.gpg.d/kitware.gpg >/dev/null
# Add kitware's repository to your sources list
sudo apt-add-repository "deb https://apt.kitware.com/ubuntu/ $(lsb_release -cs) main"
# Install cmake
sudo apt-get update
sudo apt-get install -y cmake
```

#### Install dependencies for taglib

Mac

```shell
brew install zlib
# Optional
brew install cppunit
```

Ubuntu

```shell
sudo apt-get install -y zlib1g zlib1g-dev
# Optional
sudo apt-get install -y libcppunit-dev
```

### Compile taglib

Pleas checkout the master branch and see [here](https://github.com/taglib/taglib/blob/master/INSTALL.md).

### Prepare compile Music-Tag-Cli

Install tools for compiling

- Rust tools chain
  - Please see [here](https://www.rust-lang.org/tools/install)
- pkg-config

    Mac

    ```shell
    brew install pkg-config
    ```

    Ubuntu

    ```shell
    sudo apt-get install -y pkg-config
    ```

### Install dependencies for Music-Tag-Cli

Mac

```shell
brew install opencc
```

Ubuntu

```shell
sudo apt-get install -y opencc
```

If you want to get static linked version, maybe just compile it by your self.

### Compile Music-Tag-Cli

#### Version match

| music-tag-cli | taglib-rust |                    taglib                     |
|:-------------:|:-----------:|:---------------------------------------------:|
|     1.0.0     |    2.0.0    | SHA1 20cec27ac0d9367d98cca0c7b07009f4587e1c6d |
|     1.0.1     |    2.0.1    | SHA1 20cec27ac0d9367d98cca0c7b07009f4587e1c6d |
|     1.0.2     |    2.0.1    | SHA1 20cec27ac0d9367d98cca0c7b07009f4587e1c6d |
|     1.0.3     |    2.0.2    | SHA1 20cec27ac0d9367d98cca0c7b07009f4587e1c6d |
|     1.0.4     |    2.0.2    | SHA1 20cec27ac0d9367d98cca0c7b07009f4587e1c6d |

#### Set environment

```shell
export PKG_CONFIG_PATH="any location contains .pc files, separated by `:`"
export OPENCC_LIBS="opencc:marisa"
```

#### Static linked

If you wish static linked version, you must read details.

1. Check the parameters for [installing taglib](https://github.com/taglib/taglib/blob/master/INSTALL.md).
2. Check openCC was installed with static linked lib mode. If not, you must compile it by your self. (Note: it depends on python-2.7)
3. Please see [here](https://github.com/magiclen/opencc-rust), and set environment as

    ```shell
    export OPENCC_STATIC=1
    ```

4. And you must add below for your cargo tool chains.

    ```toml
    target-feature=+crt-static
    ```

#### Prepare taglib-rust

Please select one method below:

1. Method 1

    ```shell
    git checkout https://github.com/shenngw1127/taglib-rust
    ```

    `taglib-rust` and `music-tag-lib` must be in same parent folder, then check `Cargo.toml` in Music-Tag-Cli project must be:

    ```toml
    [dependencies]
    ...
    taglib = { path = "../taglib-rust", features = ["use-pkgconfig"] }
    ```

2. Method 2

    Please check `Cargo.toml` in Music-Tag-Cli project must be:

    ```toml
    [dependencies]
    ...
    taglib = { git = "https://github.com/shenngw1127/taglib-rust", features = ["use-pkgconfig"] }
    ```

#### Compile

For compiling, just use flowing command.

```shell
# Release
cargo build --release
# Debug
cargo build
```

## Windows

If you use Windows OS, you can download the binary file from below internet drive directly. The compiled version was tested on `Windows 7 SP1 x86_64`, 8, 8.1, 10 and 11 should be fine. You may also distribute this program free,  please retain the license information.

- Download from [Google Drive](https://drive.google.com/drive/folders/1bil1Kk-gBq7naRsaA4H8bHkvtJ_xYv7d?usp=sharing)
- Download from [Baidu pan](https://pan.baidu.com/s/1sMSg9QTgC5QcoCsqaac4dQ?pwd=3crt)
