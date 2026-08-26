#!/usr/bin/env python3
"""[#5249] `secd` CTRL_HEADER 확장 tail 길이와 저장 버전의 상관을 코퍼스에서 재현한다.

HWPX→HWP 어댑터가 tail 을 바탕쪽 유무로 가르던 근거를 검증하려면, 한컴 저작 HWP5 가
실제로 무엇을 기준으로 10 byte(secd 38)와 19 byte(secd 47)를 가르는지가 필요하다.
이 스크립트는 `samples/**/*.hwp` 를 전수로 열어 구역마다 다음을 싣는다.

    파일 · FileHeader 버전 · secd 레코드 크기 · tail 바이트 · 바탕쪽 수 · 출처

출처는 `RhwpHwpxOrigin` 스트림 유무로 가른다 — rhwp 자신이 변환해 만든 산출물은
한컴 계약의 증거가 아니라 **현재 동작의 기록**이므로 집계에서 분리해야 한다.

의존성 0 (Python 3 표준 라이브러리) — CFB 리더를 직접 들고 있다.

    python scripts/secd_tail_survey.py                        # samples 전수 요약
    python scripts/secd_tail_survey.py --root samples --json  # 구역별 원시 행
"""

from __future__ import annotations

import argparse
import json
import os
import struct
import sys
import zlib
from collections import Counter

CFB_SIGNATURE = b"\xd0\xcf\x11\xe0\xa1\xb1\x1a\xe1"
FREESECT = 0xFFFFFFFF
ENDOFCHAIN = 0xFFFFFFFE

HWPTAG_BEGIN = 0x010
HWPTAG_PARA_HEADER = HWPTAG_BEGIN + 50
HWPTAG_CTRL_HEADER = HWPTAG_BEGIN + 55
HWPTAG_LIST_HEADER = HWPTAG_BEGIN + 56

# ctrl_id 는 big-endian 으로 조립해 u32 LE 로 실린다 (parser/tags.rs 의 ctrl_id).
CTRL_SECTION_DEF = (ord("s") << 24) | (ord("e") << 16) | (ord("c") << 8) | ord("d")

# secd 고정 필드(ctrl_id 4 + 24 byte) 뒤가 tail 이다.
SECD_FIXED = 4 + 24


class Cfb:
    """HWP5 를 읽는 데 필요한 만큼의 최소 CFB 리더."""

    def __init__(self, blob):
        if blob[:8] != CFB_SIGNATURE:
            raise ValueError("CFB 시그니처 없음")
        self.blob = blob
        self.sector_size = 1 << struct.unpack_from("<H", blob, 0x1E)[0]
        self.mini_sector_size = 1 << struct.unpack_from("<H", blob, 0x20)[0]
        self.first_dir = struct.unpack_from("<I", blob, 0x30)[0]
        self.mini_cutoff = struct.unpack_from("<I", blob, 0x38)[0]
        self.first_mini_fat = struct.unpack_from("<I", blob, 0x3C)[0]
        self.first_difat = struct.unpack_from("<I", blob, 0x44)[0]
        self.difat_count = struct.unpack_from("<I", blob, 0x48)[0]
        self.fat = self._read_fat()
        self.dir_entries = self._read_directory()
        self.mini_fat = self._read_chain_as_u32(self.first_mini_fat)
        root = self.dir_entries[0]
        self.mini_stream = self._read_chain(root["start"], root["size"])

    def _sector(self, index):
        off = 512 + index * self.sector_size
        return self.blob[off : off + self.sector_size]

    def _read_fat(self):
        sectors = list(struct.unpack_from("<109I", self.blob, 0x4C))
        # DIFAT 확장 — 큰 파일에서만 쓰인다.
        next_difat = self.first_difat
        for _ in range(self.difat_count):
            if next_difat in (ENDOFCHAIN, FREESECT):
                break
            data = self._sector(next_difat)
            per = self.sector_size // 4 - 1
            sectors.extend(struct.unpack_from("<" + str(per) + "I", data, 0))
            next_difat = struct.unpack_from("<I", data, per * 4)[0]
        fat = []
        for sector in sectors:
            if sector in (FREESECT, ENDOFCHAIN):
                continue
            count = self.sector_size // 4
            fat.extend(struct.unpack_from("<" + str(count) + "I", self._sector(sector), 0))
        return fat

    def _read_chain_as_u32(self, start):
        raw = self._read_chain(start, None)
        if not raw:
            return []
        return list(struct.unpack_from("<" + str(len(raw) // 4) + "I", raw, 0))

    def _read_chain(self, start, size):
        out = bytearray()
        sector = start
        steps = 0
        while sector not in (ENDOFCHAIN, FREESECT) and sector < len(self.fat):
            out += self._sector(sector)
            sector = self.fat[sector]
            steps += 1
            if steps > len(self.fat):  # 순환 방어
                break
        return bytes(out[:size]) if size is not None else bytes(out)

    def _read_mini_chain(self, start, size):
        out = bytearray()
        sector = start
        steps = 0
        while sector not in (ENDOFCHAIN, FREESECT) and sector < len(self.mini_fat):
            off = sector * self.mini_sector_size
            out += self.mini_stream[off : off + self.mini_sector_size]
            sector = self.mini_fat[sector]
            steps += 1
            if steps > len(self.mini_fat):
                break
        return bytes(out[:size])

    def _read_directory(self):
        raw = self._read_chain(self.first_dir, None)
        entries = []
        for off in range(0, len(raw), 128):
            chunk = raw[off : off + 128]
            if len(chunk) < 128:
                break
            name_len = struct.unpack_from("<H", chunk, 0x40)[0]
            name = chunk[: max(name_len - 2, 0)].decode("utf-16-le", "replace")
            entries.append(
                {
                    "name": name,
                    "type": chunk[0x42],
                    "start": struct.unpack_from("<I", chunk, 0x74)[0],
                    "size": struct.unpack_from("<Q", chunk, 0x78)[0],
                }
            )
        return entries

    def names(self):
        return [entry["name"] for entry in self.dir_entries]

    def read(self, name):
        for entry in self.dir_entries:
            if entry["name"] == name and entry["type"] == 2:
                if entry["size"] < self.mini_cutoff:
                    return self._read_mini_chain(entry["start"], entry["size"])
                return self._read_chain(entry["start"], entry["size"])
        raise KeyError(name)


def records(buf):
    """HWP5 레코드 헤더를 풀어 (tag, level, data) 를 낸다."""
    off = 0
    while off + 4 <= len(buf):
        header = struct.unpack_from("<I", buf, off)[0]
        off += 4
        tag = header & 0x3FF
        level = (header >> 10) & 0x3FF
        size = (header >> 20) & 0xFFF
        if size == 0xFFF:
            if off + 4 > len(buf):
                return
            size = struct.unpack_from("<I", buf, off)[0]
            off += 4
        yield tag, level, buf[off : off + size]
        off += size


def inflate(raw, compressed):
    if not compressed:
        return raw
    for wbits in (-15, 15):
        try:
            return zlib.decompress(raw, wbits)
        except zlib.error:
            continue
    return b""


def master_page_counts(body):
    """구역마다 바탕쪽 수 — secd 자식 구간의 최소 level LIST_HEADER 수.

    `parser/body_text.rs` 의 parse_master_pages_from_raw 와 같은 창·같은 기준이다.
    """
    windows = []
    current = None
    current_level = 0
    for tag, level, data in records(body):
        if tag == HWPTAG_CTRL_HEADER and len(data) >= 4:
            ctrl = struct.unpack_from("<I", data, 0)[0]
            if ctrl == CTRL_SECTION_DEF:
                current = []
                current_level = level
                windows.append(current)
                continue
            if current is not None and level <= current_level:
                current = None
        if tag == HWPTAG_PARA_HEADER and current is not None and level <= current_level:
            current = None
        if tag == HWPTAG_LIST_HEADER and current is not None:
            current.append(level)
    return [levels.count(min(levels)) if levels else 0 for levels in windows]


def survey_file(path):
    with open(path, "rb") as handle:
        blob = handle.read()
    cfb = Cfb(blob)
    header = cfb.read("FileHeader")
    version = "{}.{}.{}.{}".format(header[35], header[34], header[33], header[32])
    flags = struct.unpack_from("<I", header, 36)[0]
    compressed = bool(flags & 1)
    origin = "rhwp" if "RhwpHwpxOrigin" in cfb.names() else "hancom"

    rows = []
    section_names = sorted(
        (name for name in cfb.names() if name.startswith("Section")),
        key=lambda name: int(name[len("Section") :] or 0),
    )
    for name in section_names:
        body = inflate(cfb.read(name), compressed)
        counts = master_page_counts(body)
        index = 0
        for tag, _level, data in records(body):
            if tag != HWPTAG_CTRL_HEADER or len(data) < 4:
                continue
            if struct.unpack_from("<I", data, 0)[0] != CTRL_SECTION_DEF:
                continue
            rows.append(
                {
                    "file": path.replace(os.sep, "/"),
                    "section": name,
                    "version": version,
                    "secdSize": len(data),
                    "tail": data[SECD_FIXED:].hex(),
                    "masterPages": counts[index] if index < len(counts) else 0,
                    "origin": origin,
                }
            )
            index += 1
    return rows


def version_tuple(text):
    return tuple(int(part) for part in text.split("."))


def main():
    parser = argparse.ArgumentParser(description="secd tail 조사 (#5249)")
    parser.add_argument("--root", default="samples", help="탐색 루트 (기본: samples)")
    parser.add_argument("--json", action="store_true", help="구역별 원시 행을 JSON 으로")
    args = parser.parse_args()

    rows = []
    unreadable = 0
    for dirpath, _dirs, files in os.walk(args.root):
        for name in files:
            if not name.lower().endswith(".hwp"):
                continue
            try:
                rows.extend(survey_file(os.path.join(dirpath, name)))
            except Exception:
                unreadable += 1  # HWP3·암호 문서 등 — CFB 가 아니거나 열 수 없다

    if args.json:
        json.dump(rows, sys.stdout, ensure_ascii=False)
        print()
        return 0

    hancom = [row for row in rows if row["origin"] == "hancom"]
    print("구역 {} (한컴 저작 {} · rhwp 산출 {})".format(len(rows), len(hancom), len(rows) - len(hancom)))
    print("CFB 로 열 수 없는 파일 {} (HWP3·암호 문서 등)".format(unreadable))

    print("\n한컴 저작: 저장 버전대 × secd 크기")
    table = Counter(
        (
            "< 5.0.4.0" if version_tuple(row["version"]) < (5, 0, 4, 0) else ">= 5.0.4.0",
            row["secdSize"],
        )
        for row in hancom
    )
    for (band, size), count in sorted(table.items()):
        print("  {:10s} secd {:3d} byte : {}".format(band, size, count))

    modern = [row for row in hancom if version_tuple(row["version"]) >= (5, 0, 4, 0)]
    legacy = [row for row in hancom if version_tuple(row["version"]) < (5, 0, 4, 0)]
    print("\n바탕쪽 게이트 반증 (한컴 저작)")
    print(
        "  바탕쪽 0 인데 47 byte :",
        sum(1 for row in modern if row["masterPages"] == 0 and row["secdSize"] == 47),
    )
    print(
        "  바탕쪽 있는데 38 byte :",
        sum(1 for row in legacy if row["masterPages"] > 0 and row["secdSize"] == 38),
    )

    print("\n관측된 저장 버전")
    versions = Counter(row["version"] for row in hancom)
    for version, count in sorted(versions.items(), key=lambda item: version_tuple(item[0])):
        sizes = sorted({row["secdSize"] for row in hancom if row["version"] == version})
        print("  {:10s} {:4d}구역  secd {}".format(version, count, sizes))
    return 0


if __name__ == "__main__":
    sys.exit(main())
