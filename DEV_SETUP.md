# Install Rust

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
acl ssh_ports port 22
```
Then restart squid with new configuration:
```
squid -k reconfigure
```

If the above didn't work, `brew uninstall squid` and `brew install squid` again worked.

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





# Clone Github repository over this proxy

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


