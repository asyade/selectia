import os
import json
import socket
import struct
import time
import re
import sys

import demucs.api
import torch

def version(_payload):
    device = "cuda" if torch.cuda.is_available() else "cpu"
    return {
        "version": "0.1.0",
        "torch_device": device,
    }
    
def separate(payload):
    input_file = payload["input_file"]
    output_dir = payload["output_dir"]
    
    log(f"Begin separation of {input_file} to {output_dir}")
    
    separator = demucs.api.Separator(model="htdemucs", segment=4, overlap=0.1, )
    origin, separated = separator.separate_audio_file(input_file)
    
    stems = []
    for stem, source in separated.items():
        stem_path = f"{output_dir}/{stem}.wav"
        demucs.api.save_audio(source, stem_path, samplerate=separator.samplerate)
        stems.append({ "path": stem_path, "stem": stem })
    
    return { "status": "success", "stems": stems }

remote_procedure = {
    "Version": version,
    "Separate": separate,
}


def send(packet):
    packet_content = json.dumps(packet)
    packed_int = struct.pack('!i', len(packet_content))
    string_bytes = packet_content.encode('utf-8')
    buffer = packed_int + string_bytes
    sock.sendall(buffer)
    
def log(message, level = "info"):
    send({"id":"Log", "message": message, "level": level})

def require_environ(name):
    value = os.environ.get(name)
    if (value is not None):
        return value
    else:
        print("Missing required environ", name)
        exit(1)

def handle(message):
    if message['id'] == "Call":
        try:
            payload = remote_procedure[message['procedure_id']](message['payload'])
            send({"id": "CallBack", "call_id": message['call_id'], "payload": payload})
        except Exception as e:
            log(f"remote procedure error: {e}")
            send({"id": "CallBack", "call_id": message['call_id'], "payload": None})
    else:
        log(f"unknown packet: {message['id']}")

try:
    print("connecting to host ...")
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.connect(("127.0.0.1", int(require_environ("PORT"))))
    print("Connected !")
    while True:
        packet_length_data = sock.recv(4)
        if len(packet_length_data) < 4:
            print("Connection closed by client")
            break
        packet_length = struct.unpack('!i', packet_length_data)[0]
        packet_data = sock.recv(packet_length)
        if len(packet_data) < packet_length:
            print("Incomplete packet data")
            break
        raw = packet_data.decode('utf-8')
        message = json.loads(raw);        
        to_send = send({"id": "Ack", "request": message})
        try:
            handle(message)
        except Exception as e:
            print(e)
            log(f"failed to handle request {message}: {e}", level="Error")

except Exception as e:
    print(f"Error: {e}")
finally:
    sock.close()
    