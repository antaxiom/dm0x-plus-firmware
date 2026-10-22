#!/usr/bin/env python3
"""
ToSeven DM01/DM02 Universal Multi-Baud Bootloader Detector & Sniffer
Tests 9600, 19200, 38400, 57600, and 115200 Baud with both:
1. 0x02 Probe byte
2. 21-byte 0x69-encrypted IAP Wakeup Frame (0xC9, 0x77...)
Listens for ANY response byte from the controller.
"""

import sys
import time
import glob
import os
import termios
import select

def find_port():
    ports = glob.glob('/dev/cu.usbserial*') + glob.glob('/dev/cu.wch*') + glob.glob('/dev/cu.*FTDI*')
    if not ports:
        print("[!] No /dev/cu.usbserial* ports found.")
        print("    Available ports:", glob.glob('/dev/cu.*'))
        sys.exit(1)
    return ports[0]

def configure_port(fd, baud_const):
    attrs = termios.tcgetattr(fd)
    attrs[0] = 0  # iflag
    attrs[1] = 0  # oflag
    attrs[2] &= ~(termios.CSIZE | termios.PARENB | termios.CSTOPB | termios.HUPCL)
    attrs[2] |= (termios.CS8 | termios.CREAD | termios.CLOCAL)
    attrs[3] = 0  # lflag
    attrs[4] = baud_const  # ispeed
    attrs[5] = baud_const  # ospeed
    attrs[6][termios.VMIN] = 0
    attrs[6][termios.VTIME] = 0
    termios.tcsetattr(fd, termios.TCSANOW, attrs)
    termios.tcflush(fd, termios.TCIOFLUSH)

def build_21b_wakeup():
    # 21-byte wakeup frame encrypted with XOR 0x69 for DM01
    # Plaintext: C9 77 00 00 00 6C 60 00 00 5D 29 61 08 09 5A 00 32 6E 69 39 83
    # XOR 0x69:
    wire = bytes.fromhex("a01e69696905096969344008616033695b070050ea")
    return wire

def run():
    port = find_port()
    print("=" * 65)
    print("  ToSeven Universal Multi-Baud Bootloader Detector")
    print(f"  Target Port: {port}")
    print("=" * 65)
    print("\n>>> INSTRUCTION: Turn Keysight Output (or toggle switch) OFF NOW! <<<")
    print(">>> Press ENTER when power is OFF to start multi-baud scanning...")
    try:
        input()
    except (KeyboardInterrupt, EOFError):
        return

    fd = os.open(port, os.O_RDWR | os.O_NOCTTY | os.O_NONBLOCK)
    wakeup_wire = build_21b_wakeup()

    baud_rates = [
        ("38400", termios.B38400),
        ("115200", termios.B115200),
        ("9600", termios.B9600),
        ("19200", termios.B19200),
        ("57600", termios.B57600),
    ]

    print("\n[*] Starting scan across ALL baud rates...")
    print(">>> NOW TURN POWER / TOGGLE SWITCH ON! <<<\n")

    for baud_name, baud_const in baud_rates:
        configure_port(fd, baud_const)
        print(f"[*] Testing {baud_name} Baud...")
        start = time.time()

        while time.time() - start < 3.0:
            # Send 0x02 probe
            try:
                os.write(fd, b'\x02')
            except Exception:
                pass

            # If 9600 Baud, also blast 21-byte wakeup burst
            if baud_name == "9600":
                try:
                    os.write(fd, wakeup_wire)
                except Exception:
                    pass

            r, _, _ = select.select([fd], [], [], 0.05)
            if r:
                try:
                    data = os.read(fd, 128)
                    if data:
                        print(f"\n[★★★ BYTES RECEIVED AT {baud_name} BAUD! ★★★]")
                        print(f"Length: {len(data)} bytes")
                        print(f"Hex:    {data.hex()}")
                        os.close(fd)
                        return
                except Exception:
                    pass

            time.sleep(0.02)

    os.close(fd)
    print("\n[-] Scan complete. If zero bytes were received across all bauds,")
    print("    TX/RX pins are almost certainly swapped, or ground is floating.")

if __name__ == '__main__':
    run()
