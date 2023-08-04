# rearview
Raspberry Pi Zero 2W - based rearview accessory for HYDO Velovision

+ [ ] TODO: Verify procedure from fresh install.


# Wifi Hotspot

This section is based on [Sparkfun's tutorial](https://learn.sparkfun.com/tutorials/setting-up-a-raspberry-pi-3-as-an-access-point/all), but with modifications that were required for proper functionality on my Raspberry Pi Zero 2W:

## My Raspberry Pi

+ Raspberry Pi Zero 2W
+ Debian Bullseye
+ Camera: IMX219 (https://www.seeedstudio.com/IMX219-160-Camera-160-FOV-Applicable-for-Jetson-Nano-p-4603.html?queryID=457af3e50e18cf4380e82c2d008ceca1&objectID=4603&indexName=bazaar_retailer_products)
+ `uname -a`: `Linux raspberrypi 6.1.21-v7+ #1642 SMP Mon Apr  3 17:20:52 BST 2023 armv7l GNU/Linux`
+ `cat /proc/version`: `Linux version 6.1.21-v7+ (dom@buildbot) (arm-linux-gnueabihf-gcc-8 (Ubuntu/Linaro 8.4.0-3ubuntu1) 8.4.0, GNU ld (GNU Binutils for Ubuntu) 2.34) #1642 SMP Mon Apr  3 17:20:52 BST 2023`

## Install packages
```
sudo apt-get install -y hostapd dnsmasq
```

## Configure Static IP 

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

## Configure Hostapd

Create `/etc/hostapd/hostapd.conf` if it doesn't exist:

Replace the values for `ssid` and `wpa_passphrase` as desired.
Select any `channel` value between 1 and 11.
```
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
```
DAEMON_CONF="/etc/hostapd/hostapd.conf"
```

## Configure Dnsmasq

Dnsmasq is a DNS server, meaning it gives devices that connect to this wifi hotspot an IP address.

Back up the pre-existing configuration:
```
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
```
sudo systemctl enable hostapd
sudo systemctl enable dnsmasq
```

Reboot.

# Camera

## Install gstreamer

```
sudo apt-get install -y gstreamer1.0-tools gstreamer1.0-alsa \
  gstreamer1.0-plugins-base gstreamer1.0-plugins-good \
  gstreamer1.0-plugins-bad gstreamer1.0-plugins-ugly \
  gstreamer1.0-libav
```

## Run gstreamer pipeline

```
gst-launch-1.0 libcamerasrc ! video/x-raw, colorimetry=bt709, format=NV12, width=640, height=360 , framerate=30/1 ! jpegenc ! multipartmux ! tcpserversink host=0.0.0.0 port=5000
```

Explanation
+ This pipeline captures image frames from the attached CSI camera, converts it to JPEG, and serves it over TCP on port 5000.
+ The resolution, 640x360, is surprising because it is not listed in `libcamera-hello --list-cameras. This is a good resolution to use because it doesn't crop much and is small enough for 30fps.

### View on another device

Install gstreamer on Ubuntu computer with Wifi, connect to Raspberry Pi's Wifi, then run:

```
gst-launch-1.0 tcpclientsrc host=192.168.9.1 port=5000 ! multipartdemux ! jpegdec ! autovideosink
```

Alteratively, use VLC Player -> Media -> Network and enter:
```
tcp://192.168.9.1:5000
```
Note, VLC apparently introduces a long buffer, so the stream is delayed by about 1 second compared to the gstreamer pipeline above.

### View in iOS

See [MJPEGView.swift](MJPEGView.swift) for a minimal Swift/SwiftUI implementation for iPhone.

Usage: Connect iPhone to wifi hotspot, then within your ContentView,
```
MJPEGView(url: URL(string: "http://192.168.9.1:5000")!)
```

# Internet Proxy

(For development)

Since the Pi's Wifi is now used to create a hotspot, it cannot be used to access the internet.

We use a MacBook connected to the internet via a wired connection to act as a proxy for the Pi via Wifi.

**Install Squid, a proxy server for mac**:
```
brew install squid
```

**Edit the configuration file:**
```
vim /opt/homebrew/etc/squid.conf
```
to:
```
acl localnet src 192.168.0.0/16
http_access allow localnet
http_port 3128
```
Then restart squid with new configuration:
```
squid -k reconfigure
```

Assuming the network looks like this:

Device | IP
--- | ---
Raspberry Pi (DHCP server and wifi hotspot) | 192.168.9.1
MacBook (connected to Pi's wifi) | 192.168.9.160

On the Pi, append the following lines to `~/.bashrc`:
```
export http_proxy=http://192.168.9.160:3128
export https_proxy=http://192.168.9.160:3128
```

Note the IP address is the MacBook's, and the port is defined in above squid configuration.

Then as a test, run on the Pi:
```
curl ident.me # it should return your public IPv4 address.
```

## Clone Github repository over this proxy

Trying to clone a github repo will fail unless we set up a SSH ProxyCommand.

```
sudo apt-get install netcat
```
Add to `~/.ssh/config`:
```
Host github.com
  User git
  ProxyCommand nc -X connect -x macbook_ip:3128 %h %p
```
where `macbook_ip` was 192.168.9.160 above.

This configuration tells SSH to use the nc command as a proxy for all connections to github.com. The -X connect -x macbook_ip:3128 options tell nc to use the MacBook's Squid proxy.

Now `git clone` with SSH should work.
```
git clone git@github.com:velovision/rearview.git
```

# Install Rust

(Development)

According to [Dygear](https://gist.github.com/tstellanova/0a6d8a70acc58a0d5be13ebaa7c935d4?permalink_comment_id=4647130#gistcomment-4647130), the swapfile size must be changed (necessity has not been tested):

```
sudo dphys-swapfile swapoff
sudo vim /etc/dphys-swapfile
```
Change `CONF_SWAPSIZE=100` to `CONF_SWAPSIZE=512`
```
sudo dphys-swapfile setup
sudo dphys-swapfile swapon
sudo reboot
```
Finally, install rust:
```
curl https://sh.rustup.rs -sSf | sh
```


# Single button (short GPIO pins 5,6) for Startup & Shutdown

Edit `/boot/config.txt` and append:

```
# Make startup pins (5,6) also shutdown pins
# see `dtoverlay -h gpio-shutdown` for options
dtoverlay=gpio-shutdown,gpio_pin=3,active_low=1,gpio_pull=up,debounce=1000
```

This uses the default values except the debounce, which is set to 1 seconds here (press and hold to shut down)