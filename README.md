# rearview

[![Rust](https://github.com/velovision/rearview/actions/workflows/cross-compile-armv7.yml/badge.svg)](https://github.com/velovision/rearview/actions/workflows/cross-compile-armv7.yml)

Raspberry Pi Zero 2W - based rearview accessory for HYDO Velovision

# Usage

A deployed Rearview unit creates a Wifi hotspot called: `Velovision Rearview`, and runs an HTTP server at `http://192.168.9.1`, port `8000`.

The API interface is as follows:

Functionality | HTTP Method | Path | Example `curl` Command | Return information and status code
--- | --- | --- | --- | ---
Basic test connection | GET | / | `curl http://192.168.9.1:8000` | "Welcome to Velovision Rearview", 200

# Todo

+ [ ] Standalone mode (record video to SD card, rear light)
+ [ ] Implement PWM rear light control HTTP interface
+ [ ] Put systemd service text in rust code to simplify deployment
+ [ ] LEGO-style hardware assembly guide

# Architecture

+ Rust server communicates with iPhone via HTTP
+ Rust server orchestrates all hardware activations / subroutines, such as LED light and gstreamer pipeline.
+ Gstreamer pipeline is run as a systemd service and streaming video as MJPEG over TCP. See [GSTREAMER_PIPELINE_EXPLAINED.md](GSTREAMER_PIPELINE_EXPLAINED.md).

# Deployment Setup

Execute [Common Setup (below)](#common-setup) first. Then, download the latest `tar.gz` from [releases](https://github.com/velovision/rearview/releases), extract it (as `supreme-server`), copy it to the Pi, and run it with `sudo`.

# Development Setup

In addition to the [Common Setup (below)](#common-setup), see [DEV_SETUP.md](DEV_SETUP.md) to:
+ Install Rust compiler / toolchain.
+ Connect Pi to internet via connected client as proxy.
+ Clone this repo.

# Common Setup

## OS & Software

Start by flashing latest version of Raspberry Pi OS to microSD card. 

### Boot Config

Edit `/boot/config.txt` and append (replace any competing ones):

```bash
# Set according to camera sensor
dtoverlay=ov5647
max_framebuffers=2

# Customize shutdown pin
# see `dtoverlay -h gpio-shutdown` for options
dtoverlay=gpio-shutdown,gpio_pin=17,active_low=1,gpio_pull=up,debounce=1000

# Pull up pin 21 (power LED) immediately on boot
# stil leaves option to control it after boot
gpio=21=op,dh
```

This uses the default values except the debounce, which is set to 1 seconds here (press and hold to shut down)

+ Overcome "Both I2C and power-on compete for GPIO pin 3" problem with [this solution](https://raspberrypi.stackexchange.com/a/85316)
+ Pin 3 is fixed as power-on,
+ We use Pin 17 as power-off,
+ and tie them together with a diode to have a unified power on/off button.

### Install gstreamer

```bash
sudo apt-get install -y gstreamer1.0-tools gstreamer1.0-alsa \
  gstreamer1.0-plugins-base gstreamer1.0-plugins-good \
  gstreamer1.0-plugins-bad gstreamer1.0-plugins-ugly \
  gstreamer1.0-libav
```

### Systemd Services

Copy `*.service` files in `systemd-services` directory to `/etc/systemd/system` directory.

## Wifi Hotspot

This section is based on [Sparkfun's tutorial](https://learn.sparkfun.com/tutorials/setting-up-a-raspberry-pi-3-as-an-access-point/all), but with modifications that were required for proper functionality on my Raspberry Pi Zero 2W:

Works on `uname -a`: `Linux raspberrypi 6.1.21-v7+ #1642 SMP Mon Apr  3 17:20:52 BST 2023 armv7l GNU/Linux`

```bash
sudo apt-get install -y hostapd dnsmasq
```

**Configure Static IP**

Append this line to `/etc/dhcpcd.conf`:
```
denyinterfaces wlan0
```

Append these lines to `/etc/network/interfaces`:
```
auto lo
iface lo inet loopback
auto eth0
iface eth0 inet dhcp

allow-hotplug wlan0
iface wlan0 inet static
	address 192.168.9.1
	netmask 255.255.255.0
	network 192.168.9.0
	broadcast 192.168.9.255
```

**Configure Hostapd**

Create `/etc/hostapd/hostapd.conf` if it doesn't exist:

Replace the values for `ssid` and `wpa_passphrase` as desired.
Select any `channel` value between 1 and 11.
```bash
interface=wlan0
ssid=RPIZERO
hw_mode=g
channel=6
ieee80211n=1
wmm_enabled=1
ht_capab=[HT40][SHORT-GI-20][DSSS_CCK-40]
macaddr_acl=0
auth_algs=1
ignore_broadcast_ssid=0
wpa=2
wpa_key_mgmt=WPA-PSK
wpa_passphrase=raspberry
rsn_pairwise=CCMP
```

Tell `hostapd` how to find this configuration file. Edit the `DAEMON_CONF` line in `/etc/default/hostapd` :
```bash
DAEMON_CONF="/etc/hostapd/hostapd.conf"
```

**Configure Dnsmasq**

Dnsmasq is a DNS server, meaning it gives devices that connect to this wifi hotspot an IP address.

Back up the pre-existing configuration:
```bash
sudo mv /etc/dnsmasq.conf /etc/dnsmasq.conf.bak`
```

Edit/create `/etc/dnsmasq.conf`:
```
interface=wlan0
listen-address=192.168.9.1
bind-interfaces
server=8.8.8.8
domain-needed
bogus-priv
dhcp-range=192.168.9.100,192.168.9.200,24h
```

Enable systemd services:
```bash
sudo systemctl unmask hostapd
sudo systemctl enable hostapd
sudo systemctl enable dnsmasq
```

Reboot.

## Hardware 

### Pins

GPIO Pin Number | Use
--- | ---
3 | Power-on and SCL
17 | Power-off
21 | Power button status LED
