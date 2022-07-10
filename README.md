# Bright

Adjust the brightness at the configured point in time.

Individual configuration for multiple monitors.

## Usage

> `$ bright -h`

```
USAGE:
    bright <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    help    Print this message or the help of the given subcommand(s)
    run     adjust brightness
    show    show the brightness of multi monitors
```

> `$ bright run -h`

```
bright-run
adjust brightness

USAGE:
    bright run [OPTIONS]

OPTIONS:
    -c, --config <CONFIG_FILE>    config path
    -h, --help                    Print help information
```

## Config

### Locations

- `$XDG_CONFIG_HOME/bright/config.toml`
- `$HOME/.config/bright/config.toml`

### Format

```toml

# ex: [dev.'intel_backlight']
[dev.'{device_name}']

time_bright = [
  # timeitem transition
  { time = '08:00:00', bright = 30, transition = {type = "{Brust/Line}"}, },
  { time = '10:00:00', bright = 60 },
]

# interval tick (unit ms)
interval = 200

# min advance percentage per adjustment, default = 100
easing_percent = 100

# min adjustment brightness, default = 1
min_step = 1

# default transition
# type = Brust.
#   fast convert to target and then sleep to next timeline point
# type = Line.
#   convert to target, use the time between pre timeline point and next timeline point
transition = {type = "{Brust/Line}"}
```

## Features

- [x] Individual configuration for multiple monitors
- [x] Adjust the brightness at the configured point in time
- [x] Transition support [Brust/Line]

## Todo

- [x] readme config file format
- [ ] watch the config_file change (notify crate)
