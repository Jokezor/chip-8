# Save this file as generate_rom.py
rom_data = [
    0x60, 0x0A,  # 600A - Set V0 = 10
    0x61, 0x05,  # 6105 - Set V1 = 5
    0xA2, 0x00,  # A200 - Set I = 0x200
    0x70, 0x01,  # 7001 - Add 1 to V0
    0x00, 0xE0   # 00E0 - Clear screen
]

with open("test_rom.ch8", "wb") as f:
    f.write(bytearray(rom_data))
