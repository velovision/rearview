![](readme_assets/velovision-rearview-banner.jpg)

[![Rust](https://github.com/velovision/rearview/actions/workflows/cross-compile-armv7.yml/badge.svg)](https://github.com/velovision/rearview/actions/workflows/cross-compile-armv7.yml)

Velovision Rearview is an open-source, wireless camera for cycling AI applications.

# To use Velovision Rearview

1. [Buy it](https://velovision.com)
2. Put it on your bike with the included saddle rail mount
3. Turn it on. It records automatically.
4. To see recorded videos, connect to its Wifi from your computer and go to `http://192.168.9.1`

# To develop Velovision Rearview

See the following installation & web API specifications:

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

If no clients are connected after one minute, Velovision Rearview will switch to Standalone Mode until streaming mode is re-started. See [Standalone Mode](#standalone-mode) for more information.

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

## SSH
```
ssh velovision-rearview.local
```

## **GET Requests**: Idempotent reading status from the Rearview

Functionality | HTTP Method | Path | Example `curl` Command | Return information and status code | Details
--- | --- | --- | --- | --- | ---
Basic test connection | GET | / | `curl http://192.168.9.1:8000` | "Welcome to Velovision Rearview", 200 |
Is the video stream live? | GET | /camera-stream-status | `curl http://192.168.9.1:8000/camera-stream-status` | "true" or "false", 200 | We assume the video stream is a TCP Stream at port 5000 and return its status.
Battery state of charge (%) | GET | /battery-percent | `curl http://192.168.9.1:8000/battery-percent` | Success: Percentage(int), 200. Failure: "Failed to get battery state of charge", 500 | Percent is rounded to nearest integer, e.g. `87`, with no percent sign. May exceed 100.
Battery cell voltage in millivolts | GET | /battery-millivolts | `curl http://192.168.9.1:8000/battery-millivolts` | Success: mV(int), 200. Failure: "Failed to get battery voltage", 500 | Rounded to the nearest mV, e.g. `3845`.
CPU Temperature (degrees Celcius) | GET | /cpu-temp | `curl http://192.168.9.1:8000/cpu-temp` | Success: CPU temperature, 200. Failure: "Failed to read CPU temperature", 500. | Temperature is rounded nearest integer, e.g. `50` with no degrees C sign.

If the server receives a `GET` request without one of the above valid `Path`s, it returns "Unknown GET request" with status code 501.

## **PUT Requests**: Idempotent sending control commands to the Rearview

Functionality | HTTP Method | Path | Example `curl` Command | Return information and status code | Details
--- | --- | --- | --- | --- | ---
Start blinking LED | PUT | /blink-on | `curl -X PUT http://192.168.9.1:8000/blink-on` | "Turned on LED", 200 | 
Stop blinking LED | PUT | /blink-off | `curl -X PUT http://192.168.9.1:8000/blink-off` | "Turn off LED", 200 |
Turn on streaming mode | PUT | /restart-stream-mode | `curl -X PUT http://192.168.9.1:8000/restart-stream-mode` | "Restarted streaming mode", 200 | Starts `velovision-camera-mjpeg-over-tcp.service` and waits up to one minute for a client to connect to it. If no client is connected, reverts to `velovision-standalone-mode.service`.

If the server receives a `PUT` request without one of the above valid `Path`s, it returns "Unknown PUT request" with status code 501.

# Standalone Mode

If no client is connected to port 5000 after one minute of boot, Velovision Rearview will stop the TCP stream and start standalone mode, which saves videos to the onboard SD card.

Making a `restart-stream-mode` PUT request put it in streaming mode and wait another minute.

Standalone mode means stopping `systemd/velovision-camera-mjpeg-over-tcp.service`, and starting `systemd/velovision-standalone-mode.service`.

Videos are saved as H.264-encoded `.mkv` files in 1-minute chunks to `/opt/velovision/standalone_videos`. If the number of files reaches 360 (corresponding to 6 hours or ~27GB), old files will be overwritten. The file names do not reflect this rotation - they will always be named `log0000.mkv` to `log0359.mkv`. Therefore, this server has a GET endpoint that returns both the path and latest update date/time:

Note: The chunks are loaded into memory before transfer, so they should not be large. For example, 10 minute chunks are 750MB which is too large.

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
+ microSD card should be 32GB (~27GB expected usage from standalone video recording)

# Common Setup

## OS & Software

Start by flashing latest version of Raspberry Pi OS to microSD card. 

### Enable I2C

Directions from https://ozzmaker.com/i2c/

```
sudo apt install i2c-tools
```

Add the following lines to `/etc/modules`:
```
i2c-dev
i2c-bcm2708
```

### Boot Config

Edit `/boot/config.txt` and append (replace any competing ones):

```bash
# Set according to camera sensor
dtoverlay=ov5647
max_framebuffers=2

# Customize shutdown pin
# see `dtoverlay -h gpio-poweroff` for options
dtoverlay=gpio-poweroff

# Pull up pin 21 (power LED) immediately on boot
# stil leaves option to control it after boot
gpio=21=op,dh
```

The result should be:
```bash
# I2C interface
dtparam=i2c_arm=on
dtparam=i2c1=on

camera_auto_detect=1
display_auto_detect=1

# Match with camera sensor
dtoverlay=ov5647
max_framebuffers=2

# Set Pin 17 as shutdown button
dtoverlay=gpio-shutdown,gpio_pin=17,active_low=1,gpio_pull=up,debounce=1000

# Set pin 21 to turn on immediately
gpio=21=op,dh
```


Confirm fuel gauge IC (address 36):
```
sudo i2cdetect -y 1
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
dhcp-option=3
```
The `dhcp-option=3` tells DNSmasq not to provide a default gateway, meaning iPhones connected to this Raspberry Pi will be able to connect to the services running on the Pi, but also recognize that the internet is not accessible through the wifi and will smartly use cellular to connect to the internet.

Enable systemd services:
```bash
sudo systemctl unmask hostapd
sudo systemctl enable hostapd
sudo systemctl enable dnsmasq
```

Reboot.

## Custom name using Avahi

Edit the file:
```
sudo vim /etc/hostname
```
to a single line that will be the network name:
```
velovision-rearview
```

Also edit
```
sudo vim /etc/hosts
```
Line that corresponds to the name to:
```
127.0.1.1	velovision-rearview
```

Now we can SSH into the device with:
```
ssh velovision-rearview.local
```

## Hardware

### Minimal Quiescent Power

A powered off raspberry pi still consumes a non-negligible amount of power.

Rearview I/O board version 1.4 introduces some additional components to actually cut off power to the raspberry pi when shut off.

A MAX16054 latching push-button controller represents the 'desired' state. Then, it is the responsibility of `supreme-server` to detect what that 'desired' state is and shut down the pi.

### Aero Fairings

3D print your own fairings

## H.264 over TCP

```
gst-launch-1.0 libcamerasrc ! videoconvert ! 'video/x-raw,width=640,height=360' ! v4l2h264enc extra-controls=\"controls,video_bitrate=1000000\" ! 'video/x-h264,level=(string)5,framerate=30/1' ! h264parse ! matroskamux ! tcpserversink host=0.0.0.0 port=5000
```
+ Bitrate of 1,000,000 at 640x360 is acceptable
+ Bitrate of 2,000,000 at 640x360 is quite good
+ Only 30% of one core CPU! Lower than any other pipeline.
+ `v4l2h264enc` uses hardware acceleration

For testing (on Mac or Linux), run this basic python script to view the stream:
```
python3 display_h264_over_tcp.py
```

## Experimenting with SRT

Currently, the rear camera video stream is MJPEG over TCP.
The performance is not satisfactory. 640x360, quality=30 results in severe frame drops outdoors.
320x180 is better but too low of a resolution.

My goal is to run hardware-accelerated H265 over SRT. So far I haven't succeeded.

MJPEG over SRT
+ Works
+ Good frame rate / latency
+ 50% more CPU than MJPEG over TCP (60% vs 40% of a single core)
```
gst-launch-1.0 libcamerasrc ! video/x-raw, width=640, height=360, framerate=30/1 ! jpegenc quality=70 ! multipartmux ! srtsink uri=srt://:5000/
``` 

Non-accelerated H264 over SRT 
+ Works
+ Low frame rate / latency
+ Almost 80% CPU across all four cores (Extremely bad)
```
gst-launch-1.0 libcamerasrc ! video/x-raw, width=640, height=360, framerate=30/1 ! x264enc tune=zerolatency ! video/x-h264, profile=high ! mpegtsmux ! srtsink uri=srt://:5000/
```

Accelerated H264 over SRT
+ 40% of a single core CPU: good
```
gst-launch-1.0 libcamerasrc ! videoconvert ! 'video/x-raw,width=640,height=360' ! v4l2h264enc extra-controls=\"controls,video_bitrate=1000000\" ! 'video/x-h264,level=(string)5,framerate=30/1' ! mpegtsmux ! srtsink uri=srt://:5000
```
When using VLC Player on Mac to test it, the stream has to be re-started on the Pi, presumably for some initial handshake thing.

# Todo

+ [ ] Update system date/time from connected iPhone (drifts if out of battery)

# Trying out RTSP server

Install prerequisites
```
sudo apt install cmake

sudo apt-get install libglib2.0-dev

sudo apt-get install libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev libgstreamer-plugins-bad1.0-dev gstreamer1.0-plugins-base gstreamer1.0-plugins-good gstreamer1.0-plugins-bad gstreamer1.0-plugins-ugly gstreamer1.0-libav gstreamer1.0-tools gstreamer1.0-x gstreamer1.0-alsa gstreamer1.0-gl gstreamer1.0-gtk3 gstreamer1.0-pulseaudio
```

Download from version 1.18 from https://gstreamer.freedesktop.org/src/gst-rtsp-server/

```
wget https://gstreamer.freedesktop.org/src/gst-rtsp-server/gst-rtsp-server-1.18.6.tar.xz
```
The later version don't work because they require a higher version of gstreamer-1.0 that isn't distributed by apt

```
tar -xvf gst-rtsp=server-1.18.6.tar.xz
```

```
cd gst-rtsp-server-1.18.6
```

```
meson build
```

```
cd build
meson compile
```

```
cd examples
```

Non-accelerated H264 (bad performance)
```
./test-launch "(libcamerasrc ! video/x-raw, width=640, height=360, framerate=30/1 ! x264enc tune=zerolatency ! video/x-h264, profile=high ! rtph264pay name=pay0 pt=96 )"
```

