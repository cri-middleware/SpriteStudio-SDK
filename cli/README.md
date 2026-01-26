# ssconverter-cli
A command-line tool that parses `sspj` projects and creates `ssab` files.

# Usage
ssconverter-cli has two modes, **single mode** and **batch mode**.

- single mode
    - The mode converts a sspj project or a sspj zip file.
- batch mode
    - The mode converts multiple sspj projects.

## single mode

```bash
$ ssconverter-cli [OPTIONS] <sspj_file>
$ ssconverter-cli [OPTIONS] <sspj_zip_file>
```

### OPTIONS:
- `-e`, `--embedded`
    - embed assets into a ssab file
- `-o`, `--out-dir <directory>`
    - output directory path (available for single mode only)
- `-h`, `--help`
    - Print help

## batch mode

```bash
$ ssconverter-cli [OPTIONS] <sspj_file> <sspj_file> ...
$ ssconverter-cli [OPTIONS] <directory> <directory> ...
```

### OPTIONS:
- `-e`, `--embedded`
    - embed assets into a ssab file
- `-h`, `--help`
    - Print help

# For Develoeprs



