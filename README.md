# runtime requirements

- ffmpeg
- ffprobe (usually installed with ffmpeg)

# compilation requirements

- [Rust](https://www.rust-lang.org/)
- cargo (usually installed with Rust)

# compile

```console
cargo build --release
```

# compile and run

```console
cargo run --release -- <ARGUMENTS>
```

suggested arguments: --help

# example usage

```console
drop_size --input-file videos/IMG_2880.mov --make-new-frames --start-color "#00457b" --end-color "#00457b" --threshold 0.04 --size-overestimate 400
```
