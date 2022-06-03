# Bright

在指定时间点调节亮度

## Features

- [x] config file
- [x] 指定时间点, 不是区间
- [x] 为多个显示器单独指定亮度

## Config

### Locations

- `$XDG_CONFIG_HOME/bright/config.toml`
- `$HOME/.config/bright/config.toml`

### Format

```toml

# ex: [dev.'intel_backlight']
[dev.'{device_name}']

time_bright = [
  { time = '08:00:00', bright = 30 },
  { time = '10:00:00', bright = 60 },
]

```

## Todo

- [x] readme config file format
