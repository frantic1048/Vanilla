#!/usr/bin/env python3

import sys
import time

# API: https://api.kde.org/frameworks/kcodecs/html/classKEncodingProber.html
from PyKF5.KCodecs import KEncodingProber

# https://doc.qt.io/qt-5/qbytearray.html
from PyQt5.QtCore import QByteArray

prober_types = [
    KEncodingProber.Unicode,
    KEncodingProber.ChineseSimplified,
    KEncodingProber.Japanese,
    KEncodingProber.Universal,
]


prober_list = [KEncodingProber(prober_type) for prober_type in prober_types]

input_list = sys.argv[1:]


def usage():
    print("Usage:")
    print("\tencodingprober.py text_file1 [text_file2 ...]")


def assert_input_not_empty():
    if len(input_list) < 1:
        print("Insufficient args, aborted")
        usage()
        exit(1)


assert_input_not_empty()

for text_file in input_list:
    with open(text_file, "rb") as f:
        raw_data = f.read(2048)
        probe_records = {}
        for prober in prober_list:
            prober.reset()

            if prober.feed(QByteArray(raw_data)) == KEncodingProber.FoundIt:
                probe_records[
                    str(prober.encoding(), encoding="ascii")
                ] = prober.confidence()

        probe_result_list = sorted(
            list(probe_records.items()), key=lambda x: x[1], reverse=True
        )

        # FIXME: why utf8 show 0.0 confidence on utf8 file
        print(text_file)
        for [encoding, confidence] in probe_result_list:
            print(encoding, confidence)

