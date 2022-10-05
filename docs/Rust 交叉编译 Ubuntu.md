# Rust 交叉编译出 Ubuntu 程序

添加目标平台:

```bash

rustup target add x86_64-unknown-linux-musl

```

创建 config

```bash

brew install FiloSottile/musl-cross/musl-cross

mkdir -p ~/.cargo

touch ~/.cargo/config

```

config 文件内容

```bash

[target.x86_64-unknown-linux-musl]
linker = "x86_64-linux-musl-gcc"

```

Build

```bash

TARGET_CC=x86_64-linux-musl-gcc cargo build --release --target x86_64-unknown-linux-musl

```

## References

+ [Cross-compiling Rust From Mac to Linux](https://betterprogramming.pub/cross-compiling-rust-from-mac-to-linux-7fad5a454ab1)
