# Video Encoding

This is an in-depth document about the development history of the gstreamer-based video streaming/saving pipeline. It is in chronological order, so scroll to the bottom for the current implementation.

Note: Velovision Rearview's IP address is set to `192.168.9.1` and in later iterations we also use `velovision-rearview.local`. Port for streaming is always `5000`.

## MJPEG-over-TCP (Success, but bad performance)

The initial approach was to use MJPEG over TCP, which means sending JPEG images of every frame over TCP.
The benefits were relative ease of handling the stream and good compatibility. The problem was bandwidth. On iPhone 12 mini, the high bandwidth requirement would mean very low resolution or quality, and also drop-outs.

## Record video to local storage in standalone mode

```
gst-launch-1.0 libcamerasrc ! video/x-raw,width=640,height=480,format=NV12,framerate=30/1 ! v4l2convert ! v4l2h264enc ! video/x-h264,level='(string)4' ! h264parse ! splitmuxsink location=test%04d.mkv max-size-time=12000000000 max-files=3 muxer=matroskamux
```
+ `matroskamux` and its `.mkv` file format creates files that are valid even when interrupted while writing. In practice, it records up to about one second before SIGINT is sent to the stream.

### Gstreamer pipeline for streaming

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

## MJPEG over SRT (Works but not on iPhone)
+ Works on VLC
+ iPhone has poor SRT support and the library that supposedly supports it failed to work for me.
+ SRT is a bit too smart and halts the stream when a client disconnects.
+ Good frame rate / latency
+ 50% more CPU than MJPEG over TCP (60% vs 40% of a single core)
```
gst-launch-1.0 libcamerasrc ! video/x-raw, width=640, height=360, framerate=30/1 ! jpegenc quality=70 ! multipartmux ! srtsink uri=srt://:5000/
```

In VLC Player -> Media -> Network, enter:
```
srt://192.168.9.1:5000
```

### Non-accelerated H264 over SRT 
+ Works
+ Low frame rate / latency
+ Almost 80% CPU across all four cores (Extremely bad)
```
gst-launch-1.0 libcamerasrc ! video/x-raw, width=640, height=360, framerate=30/1 ! x264enc tune=zerolatency ! video/x-h264, profile=high ! mpegtsmux ! srtsink uri=srt://:5000/
```

### Accelerated H264 over SRT
+ 40% of a single core CPU: good
```
gst-launch-1.0 libcamerasrc ! videoconvert ! 'video/x-raw,width=640,height=360' ! v4l2h264enc extra-controls=\"controls,video_bitrate=1000000\" ! 'video/x-h264,level=(string)5,framerate=30/1' ! mpegtsmux ! srtsink uri=srt://:5000
```
When using VLC Player on Mac to test it, the stream has to be re-started on the Pi, presumably for some initial handshake thing.

## RTSP server (Didn't work)

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

## H.264-over-TCP (Works using Python OpenCV and iPhone, but not well on VLC)

Gstream pipeline on Rearview:

```
gst-launch-1.0 libcamerasrc ! videoconvert ! 'video/x-raw,width=640,height=360' ! v4l2h264enc extra-controls=\"controls,video_bitrate=1000000,repeat_sequence_header=1\" ! 'video/x-h264,level=(string)5,framerate=30/1,stream-format=byte-stream' ! h264parse ! tcpserversink host=0.0.0.0 port=5000
```
Notable elements and options
+ `repeat_sequence_header=1` means that SPS and PPS are sent ahead of each IDR frame instead of just at the beginning.
+ `stream-format=byte-stream` is the default but we specify it here for the sake of explicitness. Byte stream format means that a delimiter is inserted to differentiate between frames.
+ Lack of `matroskamux` element between `h264parse` and `tcpserversink`. The matroskamux re-packages the raw H.264 into MKV format. `display_h264_over_tcp.py` works whether `matroskamux` is included or not.
+ Bitrate is set to 1Mbps, which was a good medium between quality and reliability in our testing with iPhone.
+ Sequence parameter set (SPS) and Picture parameter set (PPS) are sent ahead of each IDR frame. Without it, the iOS app's decoder gives an error when trying to decode the IDR frame.

./test-launch "(libcamerasrc ! video/x-raw, width=640, height=360, framerate=30/1 ! x264enc tune=zerolatency ! video/x-h264, profile=high ! rtph264pay name=pay0 pt=96 )"
```
