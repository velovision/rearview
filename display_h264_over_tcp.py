# Python script to display video from an H.264 stream over TCP using OpenCV library.

# Import required libraries
import cv2
import sys


# Stream URL
# Replace this with your stream URL
stream_url = 'tcp://192.168.9.1:5000'

# Attempt to open the video stream
try:
    # Create a VideoCapture object
    cap = cv2.VideoCapture(stream_url)

    # Check if the stream is opened successfully
    if not cap.isOpened():
        print('Error: Could not open video stream.')
        sys.exit()

    # Read and display video frames in loop
    while True:
        # Read a frame
        ret, frame = cap.read()

        # If frame is read correctly ret is True
        if not ret:
            print('Error: Could not read frame.')
            break

        # Display the frame
        cv2.imshow('Video Stream', frame)

        # Press 'q' to quit
        if cv2.waitKey(1) & 0xFF == ord('q'):
            break

except Exception as e:
    print(f'An error occurred: {e}')
    sys.exit()

# Release the VideoCapture object and close windows
cap.release()
cv2.destroyAllWindows()

