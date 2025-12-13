# Notes

## Making a clean project refresh.

```shell
cargo clean
cargo update
cargo build
```

## Working with the AWS S3 SDK, you'll need to install CMAKE, NASM and Visual Studio

- CMAKE: https://cmake.org/download/
- NASM: https://www.nasm.us
- VISUAL STUDIO: https://visualstudio.microsoft.com/downloads/

> After installation, ensure to add both to system PATH. Then restart system, and ensure that the project on the system is not too long, as that might trigger the windows "path too long" error.