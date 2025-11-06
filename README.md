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
$ ciphercanvas generate --ssid MyNetwork --password MyPassword --encryption wpa --output qrcode.png --size 512 --foreground "#000000" --background "#ffffff"
```

Alternatively, you can use a configuration file (e.g., `config.toml`) to specify default values:

``` console
$ ciphercanvas generate --config ./config.toml --ssid MyNetwork --output qrcode.svg
```

To save frequently used settings to the default configuration file:

``` console
$ ciphercanvas save-settings --settings '[wifi]\nssid="MyNetwork"\n[qrcode]\npassword="MyPassword"'
```

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
