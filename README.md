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

# interval tick (unit ms)
interval = 200

# easing 缓动时前进百分比, default = 100
easing_percent = 100

# 最小变动亮度, default = 1
min_step = 1

# type = Brust.
#   fast convert to target and then sleep to next timeline point
# type = Line.
#   convert to target, use the time between pre timeline point and next timeline point

transition = {type = "{Brust/Line}"}
```

## Todo

- [x] readme config file format
