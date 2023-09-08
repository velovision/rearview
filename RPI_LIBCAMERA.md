# Raspberry Pi's Libcamera Library and Command Line

In order to use hardware accelerated H.264 encoding (standalone mode), we use libcamera.

First, record as h.264 intermediate format:
```
libcamera-vid --width 1296 --height 972 --timeout 5000 --output intermediate.h264 --nopreview --codec h264 --flush
```
+ 1296x972 is one of the resolutions available that keeps all the pixels (maximum FOV)
+ Timeout is in ms


```
mkvmerge -o output.mkv intermediate.h264
```
+ Saved file is `output.mkv`


TODO/TO find out

+ [ ] How to use `--segment`, `--wrap`, `--start` arguments  in libcamera-vid to set up circular recording
+ [ ] Whether sending SIGINT to the h.264 recording process corrupts it
+ [ ] How to implement the MKV file conversion while or after recording.
