import socket

NALU_TYPE_NAMES = {
    1: "Coded slice of a non-IDR picture",
    5: "Coded slice of an IDR picture",
    6: "Supplemental enhancement information (SEI)",
    7: "Sequence parameter set",
    8: "Picture parameter set",
    9: "Access unit delimiter",
    # ... include other NALU types as needed
}

def find_next_nalu(buffer):
    """
    Find the next NALU in the buffer.
    Returns the NALU data and the remaining buffer.
    """
    start_code = b'\x00\x00\x00\x01'
    start_pos = buffer.find(start_code, 1)  # Find the start of the next NALU
    if start_pos == -1:
        return None, buffer  # No complete NALU found

    nalu_data = buffer[:start_pos]  # Extract NALU
    remaining_buffer = buffer[start_pos:]  # Remaining data
    return nalu_data, remaining_buffer

def get_nalu_type(nalu):
    """
    Get the type of the NALU.
    """
    if len(nalu) > 4:
        nalu_type_code = nalu[4] & 0x1F
        return NALU_TYPE_NAMES.get(nalu_type_code, f"Unknown ({nalu_type_code})")
    return "Unknown"

def verify_h264_stream(host, port):
    """
    Connect to the TCP server and verify the H.264 stream.
    """
    with socket.create_connection((host, port)) as sock:
        print(f"Connected to {host}:{port}")
        buffer = b''
        try:
            while True:
                # Receive data from the server
                data = sock.recv(4096)
                if not data:
                    break

                buffer += data  # Append new data to buffer

                # Process complete NALUs in the buffer
                nalu, buffer = find_next_nalu(buffer)
                while nalu:
                    nalu_type = get_nalu_type(nalu)
                    print(f"Found NALU, Type: {nalu_type}, Length: {len(nalu)} bytes")
                    nalu, buffer = find_next_nalu(buffer)

        except Exception as e:
            print(f"Error: {e}")

if __name__ == "__main__":
    HOST = '192.168.9.1'  # Replace with the appropriate host
    PORT = 5000       # Replace with the appropriate port
    verify_h264_stream(HOST, PORT)
