# How to Configure a Raspberry Pi Zero 2W

From factory condition to ready for Velovision Rearview

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
