"""안 끝나는 액션(`HANG`)마다 **한글을 보이게 띄우고 창 목록을 뽑는다.**

왜 필요한가: 멈춘 액션을 "대화상자"라고 적었다가 틀렸다 — 화면을 찍어 보니 대화상자가 없었다.
69개를 눈으로 다 보는 대신 창 목록을 기계로 뽑으면 전수 판정이 되고, 대화상자가 있으면
**제목까지** 나와 어느 것인지 바로 안다. 화면도 함께 남겨 필요할 때 눈으로 확인한다.

    python tools/hwpctrl_compat/sweep_hang_windows.py [--limit N] [--shots]

출력 TSV: 이름, 창 개수, 보이는 창 제목들.
"""

from __future__ import annotations

import argparse
import io
import json
import subprocess
import sys
import tempfile
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]
SWEEPS = [
    REPO / "output" / "poc" / "hwpctrl" / "sweep_actions.tsv",
    REPO / "output" / "poc" / "hwpctrl" / "sweep_shapeobj.tsv",
    REPO / "output" / "poc" / "hwpctrl" / "sweep_table.tsv",
]
SAMPLE = "samples/para-001.hwp"

# 대화상자는 **클래스가 아니라 제목**으로 가른다.
#
# 한글의 창 클래스는 `HwndWrapper[Hwp.exe;;<GUID>]` 라 이름으로는 문서 창과 대화상자를 못
# 가른다. 대신 제목이 다르다 — 문서 창은 "… - 한글", 대화상자는 "편집 용지" 처럼 제 이름을
# 단다. 아래는 한글이 늘 달고 있는 **살림용 창**들이고, 이것 말고 제목이 있는 창이 뜨면
# 대화상자다.
HOUSEKEEPING_TITLES = {
    "Hidden Window", "DDE Server Window", "WISPTIS", "SystemResourceNotifyWindow",
    "MediaContextNotificationWindow", "GDI+ Window (Hwp.exe)", "CiceroUIWndFrame",
    "MSCTFIME UI", "Default IME", "",
}


def hang_actions() -> list[str]:
    names: list[str] = []
    for path in SWEEPS:
        if not path.exists():
            continue
        for line in io.open(path, encoding="utf-8").read().splitlines()[1:]:
            parts = line.split("\t")
            if len(parts) >= 2 and parts[1] == "HANG" and parts[0] not in names:
                names.append(parts[0])
    from sweep_actions import FORBIDDEN

    for name in sorted(FORBIDDEN):
        if name not in names:
            names.append(name)
    return names


def windows() -> list[str]:
    proc = subprocess.run(
        ["powershell", "-ExecutionPolicy", "Bypass", "-File", str(HERE / "hang_windows.ps1")],
        capture_output=True, check=False,
    )
    text = proc.stdout.decode("utf-8", "replace")
    return [line for line in text.splitlines() if line.strip()]


def main() -> int:
    sys.stdout.reconfigure(encoding="utf-8")
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--out", type=Path,
                    default=REPO / "output" / "poc" / "hwpctrl" / "hang_windows.tsv")
    ap.add_argument("--limit", type=int, default=0)
    ap.add_argument("--wait", type=float, default=18.0, help="액션을 건 뒤 창을 볼 때까지 기다리는 초")
    ap.add_argument("--shots", action="store_true", help="화면도 PNG 로 남긴다")
    ap.add_argument("--names", help="이 이름들만(쉼표로 구분)")
    args = ap.parse_args()

    # `--names` 는 **그 이름을 그대로** 본다. 예전에는 `HANG` 으로 알려진 것들만 걸러
    # 봤는데, 그러면 "안 끝나는 줄 알았지만 아니었던" 이름의 창을 볼 수가 없다 — 갈래를
    # 다시 매기려면 바로 그 이름들을 봐야 한다.
    if args.names:
        names = [n.strip() for n in args.names.split(",") if n.strip()]
    else:
        names = hang_actions()
    if args.limit:
        names = names[: args.limit]
    tmp = Path(tempfile.mkdtemp(prefix="hangwin-"))
    shots = REPO / "output" / "poc" / "hwpctrl" / "hang_shots"
    if args.shots:
        shots.mkdir(parents=True, exist_ok=True)
    args.out.parent.mkdir(parents=True, exist_ok=True)

    rows = []
    print(f"안 끝나는 액션 {len(names)}개의 창을 본다 (액션 뒤 {args.wait}초 대기)")
    for i, action in enumerate(names, 1):
        scenario = {
            "id": f"hangwin-{action}",
            "title": action,
            "ledger": [],
            "open": SAMPLE,
            "calls": [["SetPos", [0, 0, 20]], ["Run", [action]], ["GetPos", []]],
        }
        path = tmp / f"hangwin-{action}.json"
        with io.open(path, "w", encoding="utf-8", newline="\n") as fh:
            json.dump(scenario, fh, ensure_ascii=False)
        subprocess.run(["taskkill", "/F", "/IM", "Hwp.exe"], capture_output=True, check=False)
        env = {"HWPCTRL_VISIBLE": "1"}
        proc = subprocess.Popen(
            [sys.executable, str(HERE / "runner_ocx.py"), str(path), "--out", str(tmp),
             "--expect-version", "12"],
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
            env={**dict(__import__("os").environ), **env},
        )
        time.sleep(args.wait)
        rows_now = windows()
        visible = [r.split("\t") for r in rows_now]
        # 제목이 있고 살림용도 아니고 문서 창(" - 한글")도 아니면 대화상자다.
        extra = [
            v for v in visible
            if len(v) >= 3
            and v[2].strip() not in HOUSEKEEPING_TITLES
            and not v[2].endswith(" - 한글")
        ]
        titles = " | ".join(v[2] for v in extra) if extra else ""
        if args.shots:
            subprocess.run(
                ["powershell", "-ExecutionPolicy", "Bypass", "-File", str(HERE / "screenshot.ps1"),
                 "-Out", str(shots / f"{action}.png")],
                capture_output=True, check=False,
            )
        proc.kill()
        subprocess.run(["taskkill", "/F", "/IM", "Hwp.exe"], capture_output=True, check=False)
        rows.append((action, str(len(visible)), titles))
        print(f"  [{i}/{len(names)}] {action}: 창 {len(visible)} {titles[:60]}")

    with io.open(args.out, "w", encoding="utf-8", newline="\n") as fh:
        fh.write("action\twindows\textra_visible\n")
        for row in rows:
            fh.write("\t".join(row) + "\n")
    with_dialog = [r for r in rows if r[2]]
    print(f"\n창이 더 뜬 액션 {len(with_dialog)} / {len(rows)}")
    for row in with_dialog:
        print(f"   {row[0]:<26} {row[2][:80]}")
    print(f"→ {args.out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
