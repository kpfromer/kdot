# kdot

## Description

A dot file manager (similiar to [stow](https://linux.die.net/man/8/stow)) for Arch Linux using Rust.

This also my first Rust project so I am learning here! :)

## Usage

### Configuration File

At the root of our dotfiles you need to create a file called `kdot.json` with the following structure:

```json
{
  "modules": [
    {
      "name": "bash",
      "location": {
        "from": "bash",
        "to": "/home/user"
      }
    },
    {
      "name": "polybar",
      "location": {
        "from": "polybar",
        "to": "/home/user/.config/polybar"
      }
    }
  ]
}
```

Here we have defined the `bash` module and `polybar` module.

### Commands

- `kdot link [modules]` - links the module to the `to` location.
- `kdot unlink [modules]` - unlinks the module to the `from` location.
- `kdot sync [modules]` - unlinks and relinks the module.

`modules` can be one or more modules (seperated by spaces).

Also read the help dialog via `kdot --help`. It will always be up to date.

## License

This project is using the [MIT license](LICENSE).