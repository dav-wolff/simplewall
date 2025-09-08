# simplewall

Simple wallpaper utility for Wayland compositors with layer shell support.

## Installation

### Using nix

simplewall is available as a flake.

Temporary shell:  
```sh
nix shell github:dav-wolff/simplewall
```

Flake input:  
```sh
inputs.simplewall.url = "github:dav-wolff/simplewall";
```

### Using cargo

```sh
cargo install --locked --git https://github.com/dav-wolff/simplewall
```

## Usage

Display a wallpaper:  
```sh
simplewall wallpaper.jpg
```

### Custom namespace

You can set a custom namespace to use for the wallpaper's layer shell surface.
This allows compositors to differentiate between multiple wallpapers,
e.g. niri can apply different wallpapers to workspaces and the overview.

Display a wallpaper with a namespace:  
```sh
simplewall wallpaper.jpg --namespace custom-wallpaper
```

### Multiple wallpapers

Multiple wallpapers can be displayed at once using `--` as a separator.

Display multiple wallpapers with different namespaces:  
```sh
simplewall wallpaper1.jpg -n main-wallpaper
    -- wallpaper2.jpg -n overlay-wallpaper
```
