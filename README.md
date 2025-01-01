# rust_dirworker

## Info

This project's purpose is just to learn some rust.

Simple CLI tool that performs actions on paths specified in a JSON config.

Supported actions:
* `list`: print content of directory
* `analyze`: print number of files and size of directory
  * optionally with subdirectories if `include_directories` is true
* `backup`: create zip of content inside configured `backup_root_path` (zip name will be `<source_dir>_<timestamp>.zip`.)
* `clean`: delete content of directory
  * optionally with subdirectories if `include_directories` is true

## Usage

Usage:
```shell
dirworker example/config.json
```

Config example:
```json
{
    "directories": [
        {
            "action": "list",
            "path": "example/dummy",
            "include_directories": true
        },
        {
            "action": "analyze",
            "path": "example/dummy",
            "include_directories": true
        },
        {
            "action": "analyze",
            "path": "example/dummy",
            "include_directories": false
        },
        {
            "action": "backup",
            "path": "example/dummy"
        }
    ],
    "backup_root_path": "example/backup"
}
```


## Development

### Run tests

```shell
cargo test --verbose
```

### Format

```shell
cargo fmt --verbose
```
