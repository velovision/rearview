#!/bin/bash
if [[ $EUID -ne 0 ]]; then
    echo "This script must be run as root"
    exit 1
fi

# Check if exactly one argument is passed
if [[ $# -ne 1 ]]; then
    echo "Usage: $0 [dev|prod]"
    exit 1
fi

mkdir -p /opt/velovision/standalone_videos

cp ./systemd/*.service /etc/systemd/system/

mkdir -p /opt/velovision/scripts
cp ./systemd/standalone_gstreamer.sh /opt/velovision/scripts/standalone_gstreamer.sh

systemctl enable velovision-camera-mjpeg-over-tcp.service

if [ "$1" == "prod" ]; then
    systemctl enable velovision-supreme-server.service
fi
