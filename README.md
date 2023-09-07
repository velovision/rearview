# rearview

[![Rust](https://github.com/velovision/rearview/actions/workflows/cross-compile-armv7.yml/badge.svg)](https://github.com/velovision/rearview/actions/workflows/cross-compile-armv7.yml)

Raspberry Pi Zero 2W - based rearview accessory for HYDO Velovision

# Installation

Run [Common Setup (below)](#common-setup).

For developing (compiling on the Raspberry Pi), run [DEV_SETUP.md](DEV_SETUP.md)

Run the installation script
```
sudo ./install.sh dev
```
Running with `dev` argument does the following:
+ Creates `/opt/velovision/standalone_videos` path
+ Copies service files in `systemd` directory to `/etc/systemd/system`

For production, use `prod` argument:
```
sudo ./install.sh prod
```
+ manually copy the executable of this rust project called `supreme-server` to `/opt/velovision` directory. Download from: [releases](https://github.com/velovision/rearview/releases)

# Usage

On boot, Velovision Rearview unit creates a Wifi hotspot called `Velovision Rearview`. Connect to it from a Mac or Linux computer.

It runs an HTTP server at port 8000 to allow clients to get status information and send control commands,
and a raw TCP stream at port 5000 which streams MJPEG video from its camera. 

If no clients are connected after one minute, Velovision Rearview will switch to Standalone Mode. See [Standalone Mode](#standalone-mode) for more information.

We can use [VLC](https://www.videolan.org/vlc/) media player to view the video stream. 
+ Open VLC
+ File > Open Network. In the URL, enter: `tcp://192.168.9.1:5000`
+ You should see the camera video stream with a slight lag (the lag is intentional and caused by VLC)
+ See [`MJPEGView.swift`](MJPEGView.swift) to see a basic SwiftUI implementation of parsing and displaying the JPEG images from the raw TCP stream.

We can use `curl` to test out Rearview's HTTP control interface. 
Replace the curly bracket items by referring to the table below.
```
curl -X {HTTP Method} http://{IP}:{Port}/{Path}
```

By default:
+ IP: `192.168.9.1`
+ Port `8000`

## **GET Requests**: Idempotent reading status from the Rearview

Functionality | HTTP Method | Path | Example `curl` Command | Return information and status code | Details
--- | --- | --- | --- | --- | ---
Basic test connection | GET | / | `curl http://192.168.9.1:8000` | "Welcome to Velovision Rearview", 200 |
Is the video stream live? | GET | /camera-stream-status | `curl http://192.168.9.1:8000/camera-stream-status` | "true" or "false", 200 | We assume the video stream is a TCP Stream at port 5000 and return its status.
Battery state of charge (%) | GET | /battery-percent | `curl http://192.168.9.1:8000/battery-percent` | Percentage(int), 200 | Percent is rounded to nearest integer, e.g. `87`, with no percent sign. May exceed 100.
CPU Temperature (degrees Celcius) | GET | /cpu-temp | `curl http://192.168.9.1:8000/cpu-temp` | Success: CPU temperature, 200. Failure: "Failed to read CPU temperature", 500. | Temperature is rounded nearest integer, e.g. `50` with no degrees C sign.

If the server receives a `GET` request without one of the above valid `Path`s, it returns "Unknown GET request" with status code 501.

## **PUT Requests**: Idempotent sending control commands to the Rearview

Functionality | HTTP Method | Path | Example `curl` Command | Return information and status code | Details
--- | --- | --- | --- | --- | ---
Start blinking LED | PUT | /blink-on | `curl -X PUT http://192.168.9.1:8000/blink-on` | "Turned on LED", 200 | 
Stop blinking LED | PUT | /blink-off | `curl -X PUT http://192.168.9.1:8000/blink-off` | "Turn off LED", 200 |
Turn on video stream | PUT | /camera-stream-on | `curl -X PUT http://192.168.9.1:8000/camera-stream-on` | Success: "Turned camera stream on", 200. Failure: "Failed to turn on camera stream", 500 | Starts `camera-mjpeg-over-tcp.service` systemd service which runs a Gstreamer pipeline.
Turn off video stream | PUT | /camera-stream-off | `curl -X PUT http://192.168.9.1:8000/camera-stream-off` | Success: "Turned camera stream off", 200. Failure: "Failed to turn off camera stream", 500 | Stops `camera-mjpeg-over-tcp.service` systemd service.

If the server receives a `PUT` request without one of the above valid `Path`s, it returns "Unknown PUT request" with status code 501.

# Standalone Mode

If no client is connected to port 5000 after one minute of boot or within any ten-second window after that, Velovision Rearview will stop the TCP stream and start standalone mode, which saves videos to the onboard SD card.

Standalone mode means stopping `systemd/velovision-camera-mjpeg-over-tcp.service`, and starting `systemd/velovision-standalone-mode.service`.

Videos are saved as H.264-encoded `.mkv` files in 1-minute chunks to `/opt/velovision/standalone_videos`. If the number of files reaches 120, old files will be overwritten. The file names do not reflect this rotation - they will always be named `log0000.mkv` to `log0119.mkv`. Therefore, this server has a GET endpoint that returns both the path and latest update date/time:

Functionality | HTTP Method | Path | Example `curl` Command | Return information and status code | Details
--- | --- | --- | --- | --- | ---
Get path and update time of standalone videos | GET | /list-local-videos | `curl http://192.168.9.1:8000/list-local-videos` | ```[{"path": "/opt/standalone_mode/videos/loop0001.mkv","date_updated":"2023-06-17T09:13:00"},...]```, 200 |

Then, use the "path" from the above GET request to send a POST request, which returns the video file itself

Functionality | HTTP Method | Path | Example `curl` Command | Return information and status code | Details
--- | --- | --- | --- | --- | ---
Download specified video which was recorded in standalone mode | POST | /download-video | `curl -X POST -o DOWNLOAD_AS_NAME.mkv -d "/PATH/TO/VIDEO/ON/PI.mkv" http://192.168.9.1:8000/download-video` | Matroska video saved to client, 200 | 

# Hardware Specifications

+ Tested: 3.5 hours of runtime (camera streaming, no LED) on 1800mAh 103450 li-ion battery
+ Extrapolates to 5.5 hours of runtime on 2800mAh (143450 battery)

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
