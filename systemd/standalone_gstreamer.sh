#!/bin/bash

# This script implements Gstreamer pipeline that records video to the SD card (/opt/velovision/standalone_videos)
# with additional logic that ensures that videos are overwritten circularly such that the oldest video is always overwritten first.

# Get the last updated file
DIR="/opt/velovision/standalone_videos/"
LAST_UPDATED_FILE=$(ls -t ${DIR}log*.mkv | head -n 1)

# print last updated file
echo "Last updated file:"
echo $LAST_UPDATED_FILE

# If no file is found, set NUMBER to 0
if [[ -z "$LAST_UPDATED_FILE" ]]; then
    NUMBER=-1
else
    # Extract the number from the filename using awk
    NUMBER=$(echo $LAST_UPDATED_FILE | awk -F'log|.mkv' '{print $2}')
    echo "Number:"
    echo $NUMBER
fi

# Remove leading zeros from the number
# because files are named log0001.mkv, log0002.mkv, etc.
# and leading zeros are interpreted as octal numbers
NO_LEADING_ZEROS_NUMBER=$(echo $NUMBER | sed 's/^0*//')

# print number without leading zeros
echo "Number without leading zeros:"
echo $NO_LEADING_ZEROS_NUMBER

# Increment number for the next file
NEXT_NUM=$((NO_LEADING_ZEROS_NUMBER + 1))

# print number
echo "Next number:"
echo $NEXT_NUM

# Launch GStreamer pipeline with the updated starting number
gst-launch-1.0 libcamerasrc ! video/x-raw,width=1280,height=720,format=NV12,framerate=30/1 ! v4l2convert ! v4l2h264enc ! video/x-h264,level='(string)4' ! h264parse ! splitmuxsink location=/opt/velovision/standalone_videos/log%04d.mkv start-index=$NEXT_NUM max-size-time=60000000000 max-files=360 muxer=matroskamux
