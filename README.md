<h1 align="center">minipacked</h1>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg?style=flat" alt="License"></a>
  <img src="https://img.shields.io/badge/Rust-2024-orange?style=flat&logo=rust" alt="Rust 2024">
  <img src="https://img.shields.io/badge/platform-linux%20%7C%20macOS-lightgrey?style=flat" alt="Platforms">
</p>

<p align="center"><i>Pack files and directories into portable (or even encrypted) containers with a small CLI.</i></p>

---

## Usage

### Pack a File

```sh
minipack file.txt
```

### Pack a Directory

```sh
minipack -r project/
```

### Unpack an Archive

```sh
miniunpack test.minipacked
```

---

## Compression Modes

- **Default** -- Balanced mode for normal use
- **`--fast`** -- Lower compression, faster packing
- **`--compact`** -- Higher compression, slower packing, smaller output

---

## Contributing

Pull requests are welcome. For larger changes, open an issue or discussion first so behavior and file format changes can be reviewed before implementation.

Before submitting changes, make sure the project still builds.

---

## License

minipacked is licensed under MIT.
