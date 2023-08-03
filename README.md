# rearview
Raspberry Pi Zero 2W - based rearview accessory for HYDO Velovision

# Setup

+ [ ] TODO: Verify procedure from fresh install.

# Setup

## Wifi Hotspot

## Camera

### View on another device

```
gst-launch-1.0 tcpclientsrc host=192.168.9.1 port=5000 ! multipartdemux ! jpegdec ! autovideosink
```
