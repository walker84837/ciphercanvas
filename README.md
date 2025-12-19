# ciphercanvas: Wi-Fi QR Code Generator

A robust and efficient command-line tool written in Rust that generates QR codes for Wi-Fi
networks. It takes inputs such as SSID, encryption type (WPA or WEP), password,
and desired output format, producing a QR code that simplifies Wi-Fi access
sharing.

## Table of Contents

  - [Usage](#usage)
  - [Contributing](#contributing)
  - [License](#license)

## Usage

To generate a Wi-Fi QR code using CipherCanvas, use the `generate` subcommand with the appropriate options:

``` console
$ ciphercanvas generate --ssid MyNetwork --encryption wpa --output qrcode.png --size 512 --foreground "#000000" --background "#ffffff"
```

### Options:

- `--verbose`, `-v`: Activate verbose mode for detailed logs.
- `--ssid <SSID>`, `-s <SSID>`: The Wi-Fi network's SSID (name). (Required)
- `--encryption <TYPE>`, `-e <TYPE>`: The encryption type used. Valid values are `wpa`, `wep`, or `None`. (Default: `wpa`)
- `--password-file <FILE_PATH>`: Read the Wi-Fi network's password from the specified file. If not provided, the password will be read from stdin.
- `--output <FILE_PATH>`, `-o <FILE_PATH>`: The output file to export the QR code image.
- `--size <PIXELS>`: The size of the QR code image in pixels (e.g., `512`). (Default: `512`)
- `--format <FORMAT>`: The output format of the image (e.g., `"svg"`, `"png"`). (Default: `"svg"`)
- `--foreground <COLOR>`: The foreground color of the QR code (e.g., `"#000000"`). (Default: `"#000000"`)
- `--background <COLOR>`: The background color of the QR code (e.g., `"#ffffff"`). (Default: `"#ffffff"`)
- `--overwrite`: Overwrite existing files without prompt. (Default: `false`)



## Contributing

We welcome contributions from the community! If you would like to contribute to
CipherCanvas, here is a few things you can do:

### Roadmap

- [ ] Support exporting images in other formats
- [ ] Switch to a crate which provides fancier logging (like `env_logger`)
- [ ] Create config file if not found ([directories](https://docs.rs/directories/6.0.0/directories/struct.ProjectDirs.html))
- [ ] Resolve issue where an image that is smaller than 256x256 is smaller
  - Consider using a bigger image and [rescaling](https://docs.rs/image/latest/image/imageops/fn.resize.html) it.

## License

This project is licensed under the [GNU General Public License
version 3.0](LICENSE.md). By contributing to this project, you agree to license
your contributions under the same license.
