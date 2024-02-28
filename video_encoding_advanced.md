# Video Encoding: The Nitty Gritty

## Current Implementation

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

## Failed Attempts (Combinations of encoding & transport protocol)

### MJPEG over SRT
+ Works
+ Good frame rate / latency
+ 50% more CPU than MJPEG over TCP (60% vs 40% of a single core)
```
gst-launch-1.0 libcamerasrc ! video/x-raw, width=640, height=360, framerate=30/1 ! jpegenc quality=70 ! multipartmux ! srtsink uri=srt://:5000/
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

### Trying out RTSP server

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
