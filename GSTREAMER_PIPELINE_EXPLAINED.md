Honestly, Gstreamer is such a bad programming interface and the resulting pipeline is more like a magical incantation, but here it goes anyway.

## Record video to local storage in standalone mode

```
gst-launch-1.0 libcamerasrc ! video/x-raw,width=640,height=480,format=NV12,framerate=30/1 ! v4l2convert ! v4l2h264enc ! video/x-h264,level='(string)4' ! h264parse ! splitmuxsink location=test%04d.mkv max-size-time=12000000000 max-files=3 muxer=matroskamux
```
+ `matroskamux` and its `.mkv` file format creates files that are valid even when interrupted while writing. In practice, it records up to about one second before SIGINT is sent to the stream.

## Stream over TCP

## Run gstreamer pipeline

```
gst-launch-1.0 libcamerasrc ! libcamerasrc ! video/x-raw, width=640, height=360, framerate=30/1 ! jpegenc quality=30 ! multipartmux ! tcpserversink host=0.0.0.0 port=5000 buffers-soft-max=2 recover-policy=latest
```

Explanation
+ This pipeline captures image frames from the attached CSI camera, converts it to JPEG, and serves it over TCP on port 5000.
+ The resolution, 640x360, is surprising because it is not listed in `libcamera-hello --list-cameras. This is a good resolution to use because it doesn't crop much and is small enough for 30fps.
+ Remember that JPEG quality has a large impact on bitrate, and quality degradation is unnoticeable until below 20.
+ Importantly, the `buffers-soft-max` and `recover-policy` fix a memory leak that occurs in `tcpserversink`. The leak is gradual and tends to happen after 5 or so minutes when a client is connected.

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

