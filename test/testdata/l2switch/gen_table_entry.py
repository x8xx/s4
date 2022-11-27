import os

dir = os.path.dirname(__file__)
with open(dir + '/init_table_entry.s4bin', 'wb') as f:
    bin = bytearray([])
    # entry_count
    bin.append(48)
    bin.append(0)
    bin.append(0)
    bin.append(0)
    for i in range(48):
        # table_id
        bin.append(0)
        # entry buf size
        bin.append(13)

        # value count
        bin.append(1)
        # value len
        bin.append(6)
        # dst mac address
        bin.append(0)
        bin.append(0)
        bin.append(0)
        bin.append(0)
        bin.append(0)
        bin.append(i)
        # va;ue prefix mask
        bin.append(0xff)
        # priority
        bin.append(1)
        # action_id
        bin.append(1)
        # action data len
        bin.append(1)
        # action data
        bin.append(i)

    f.write(bin)
