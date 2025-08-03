# Mage Arena Flag Editor

> Please use this responsibly - don't ruin the fun of custom flags for everyone!

This is an unofficial application that allows you to set or get your custom flag in the game Mage Arena using the
Microsoft Bitmap Image format (`.bmp`).

You can run `.\mage_arena_flag_editor.exe --help` (or add `--help` to any command) for detailed instructions.

## Pre-compiled binary

You can download the pre-compiled binary from GitHub Releases.

I don't have a Windows Code Signing certificate (because they cost upwards of $1,000) so Windows may warn about the
pre-compiled binary.

The binaries are built from the source code using CI/CD (so what you see in the code is what you get), but you can
always refer to [Running from source](#running-from-source) to build it locally on your machine, if you'd prefer.

## Running from source

To run the program directly from the Rust source code,
[you'll need to have the Rust compiler installed](https://www.rust-lang.org/tools/install) (ideally version `1.88+`).

Then, swap references to `.\mage_arena_flag_editor.exe` with `cargo run` below.

## Exporting your flag

To export your flag as a bitmap (`.bmp`) image, use the `read` command (to read your flag from the registry):

```powershell
.\mage_arena_flag_editor.exe read
```

The file will be saved as `flag.bmp` in the current folder by default.
You can pass `--output-file` to change this location.

## Importing your flag

1. Export your flag as a 24-bit bitmap image (with an exact resolution of 100x66).
   You can save your flag as `custom_flag.bmp` and run the command from the same folder, or use the `--input-file` flag
   to pass a different path to your custom flag.
2. Use the `write` command (to write your flag to the registry):
   ```powershell
   .\mage_arena_flag_editor.exe write
   ```
3. This command may take a few seconds as it needs to map your custom image to the color palette supported by Mage
   Arena (you can speed things up by shrinking the `palette.bmp` image, but don't shrink it too much or you'll limit the
   colors that can be chosen even further).