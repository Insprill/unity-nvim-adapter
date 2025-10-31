[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]




<!-- PROJECT LOGO -->
<br />
<div align="center">
  <h1>unity-nvim-adapter</h1>
  <p>
    A compatibility layer between Unity's Visual Studio Code integration and Neovim.
    <br />
    <br />
    <a href="https://github.com/Insprill/unity-nvim-adapter/issues">Report Bug</a>
    ·
    <a href="https://github.com/Insprill/unity-nvim-adapter/issues">Request Feature</a>
  </p>
</div>




## Installation

### Adapter

To install the adapter, ensure you have [Rust][rustup] installed, then run the commands below:
```bash
git clone https://github.com/Insprill/unity-nvim-adapter && cd unity-nvim-adapter
cargo build --release
cp target/release/unity-nvim-adapter ~/.local/share/code
```

The most important part of the above is renaming the binary to `code`.
This is *required* for Unity to integrate with it properly.

### Neovim Plugin

The Neovim plugin can be installed like any other plugin.

When using with lazy.nvim,
ensure to set `lazy = false` so the pipe can be created as soon as Neovim starts.
```lua
{
  "insprill/unity-nvim-adapter",
  lazy = false,
}
```

### Unity

In your Unity project, ensure you have the [Visual Studio Editor][visual-studio-editor] package installed.

Go to `Edit` > `Preferences` > `External Tools`,
click on the "External Script Editor" dropdown,
select `Browse`, then find and select the adapter "code" binary (`~/.local/share/code` if you copied the commands above).

Now if you open a script or click on a log line, it will open in Neovim :)




## Configuration

The configuration is located at `~/.config/unity-nvim-adapter.toml`.

Example default configuration
```toml
use_neovide = false
```




<!-- MARKDOWN LINKS & IMAGES -->
[contributors-shield]: https://img.shields.io/github/contributors/Insprill/unity-nvim-adapter.svg?style=for-the-badge
[contributors-url]: https://github.com/Insprill/unity-nvim-adapter/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/Insprill/unity-nvim-adapter.svg?style=for-the-badge
[forks-url]: https://github.com/Insprill/unity-nvim-adapter/network/members
[stars-shield]: https://img.shields.io/github/stars/Insprill/unity-nvim-adapter.svg?style=for-the-badge
[stars-url]: https://github.com/Insprill/unity-nvim-adapter/stargazers
[issues-shield]: https://img.shields.io/github/issues/Insprill/unity-nvim-adapter.svg?style=for-the-badge
[issues-url]: https://github.com/Insprill/unity-nvim-adapter/issues

[visual-studio-editor]: https://docs.unity3d.com/Packages/com.unity.ide.visualstudio@2.0/
[rustup]: https://rustup.rs/
