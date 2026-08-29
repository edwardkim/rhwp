#!/usr/bin/env python
"""한컴이 그림 회전·뒤집기를 저장할 때 SHAPE_COMPONENT `flip` 워드를 어떻게 두는가.

## 왜 필요한가

rhwp 의 그림 setter 는 회전이 바뀌면 `rotate_image = true` 와 `flip |= 0x0008_0000`
(bit19) 을 **각도와 무관하게** 세운다. 회전을 0 으로 되돌려도 그대로 남는다. 이것이
결함인지 한컴 관례인지는 스펙에 없다 — 폐쇄 포맷이라 **한글이 실제로 뭘 쓰는지**로만
갈린다. 이 도구가 그 한 가지를 잰다.

판정하려는 것:
  (A) bit19 가 "지금 회전돼 있음" 을 뜻하는가, 아니면 다른 뜻인가.
  (B) `rotate_image` 와 bit19 가 같이 움직이는가. rhwp 파스본에서는 어긋난다
      (한컴 34° 회전 그림: `rotate_image=false` 인데 bit19 는 켜져 있다).

## 모드

  --survey PATH...            COM 불필요. `rhwp dump` 로 한컴 저장본의 표를 만든다.
                              bit19 가 회전 0 에서도 나타나는지 전수로 본다.
  --resave PATH...            한글로 열어 **그대로** 저장한 뒤 대조한다. 한글이 저장 시
                              정규화하는지 본다(원본이 한컴 산출이면 보존이 기대값).
  --set-rotation DEG PATH...  한글로 열어 첫 그림 회전을 DEG 로 바꿔 저장한 뒤 대조한다.
                              DEG=0 이 결정적 실험이다.
  --child MODE SRC DST [DEG]  내부용. 문서 1건 = 자식 프로세스 1개.

## 사용

  python tools/hangul_rotation_oracle/oracle.py --survey samples/*.hwp
  python tools/hangul_rotation_oracle/oracle.py --resave samples/ta-pic-001-r.hwp
  python tools/hangul_rotation_oracle/oracle.py --set-rotation 0 samples/ta-pic-001-r.hwp

`--exe` 로 rhwp 실행 파일을 지정한다(기본 `target/release/rhwp.exe`). 출처가 분명한
빌드를 쓸 것 — 오래된 exe 는 유령 회귀를 만든다.

## 함정 (tools/hwp_oracle_pdf.ps1 에서 확인된 것과 같다)

- `SetMessageBoxMode(0x00020000)` 없이는 대화상자가 사람을 기다리며 멈춘다.
- 시작 전 잔여 `Hwp.exe` 를 정리한다. 떠 있는 인스턴스에 붙으면 그 창의 상태
  (수정됨 문서 등)를 물려받아 저장 확인 대화상자가 뜬다.
- `Open(path, "", "")` — 형식을 "HWP" 로 못박으면 .hwpx 가 빈 문서로 조용히 열린다.
- `FilePathCheckerModule` 이 등록돼 있지 않으면 파일 접근 확인 대화상자가 뜬다.
  등록은 DLL 설치 + regsvr32 + 레지스트리가 필요해 이 도구는 하지 않는다.
- 한글 COM 은 실패 뒤 인스턴스가 오염된다 — 문서 1건당 자식 프로세스로 격리한다.
- 이 장비에는 한글 2022 와 2024 가 함께 깔려 있을 수 있다. ProgID 가 어느 쪽에 붙는지는
  등록 상태에 달렸으므로 **매 실행 버전을 기록한다**. 기록하지 않으면 서로 다른 오라클의
  결과를 같은 표에 섞게 된다.
- 여러 판정을 동시에 돌리지 말 것. 서로의 `Hwp.exe` 를 죽여 무응답 오판을 만든다.
"""
from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
import tempfile

sys.stdout.reconfigure(encoding="utf-8", errors="replace")

ROTATE_IMAGE_BIT = 0x0008_0000

# `rhwp dump` 의 변환 줄. mydocs/manual/dump_command.md 의 형식과 함께 움직인다.
TRANSFORM = re.compile(
    r"변환: 뒤집기=\((\w+),(\w+)\), 회전=(-?\d+), flip=0x([0-9a-fA-F]+), rotateImage=(\w+)"
)


class Transform:
    """그림 하나의 변환 상태 — 판정 단위."""

    __slots__ = ("horz_flip", "vert_flip", "angle", "flip", "rotate_image")

    def __init__(self, horz_flip, vert_flip, angle, flip, rotate_image):
        self.horz_flip = horz_flip
        self.vert_flip = vert_flip
        self.angle = angle
        self.flip = flip
        self.rotate_image = rotate_image

    @property
    def bit19(self) -> bool:
        return bool(self.flip & ROTATE_IMAGE_BIT)

    @property
    def rotated(self) -> bool:
        return self.angle % 360 != 0

    def __eq__(self, other) -> bool:
        return isinstance(other, Transform) and self.key() == other.key()

    def key(self):
        return (
            self.horz_flip,
            self.vert_flip,
            self.angle,
            self.flip,
            self.rotate_image,
        )

    def __str__(self) -> str:
        return (
            f"각도={self.angle:>4} flip=0x{self.flip:08x} bit19={int(self.bit19)}"
            f" rotateImage={int(self.rotate_image)} 뒤집기=({int(self.horz_flip)},{int(self.vert_flip)})"
        )


def transforms(exe: str, path: str) -> list[Transform]:
    """`rhwp dump` 에서 변환 줄을 모두 읽는다."""
    out = subprocess.run(
        [exe, "dump", path],
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=300,
    )
    found = []
    for h, v, angle, flip, ri in TRANSFORM.findall(out.stdout):
        found.append(
            Transform(h == "true", v == "true", int(angle), int(flip, 16), ri == "true")
        )
    return found


# ---------------------------------------------------------------- 한글 COM


def hangul_version(com) -> str:
    try:
        return str(com.Version)
    except Exception:  # noqa: BLE001
        return "unknown"


def kill_hangul() -> None:
    for image in ("Hwp.exe", "HwpFrame.exe"):
        subprocess.run(
            ["taskkill", "/F", "/IM", image],
            capture_output=True,
            text=True,
        )


def child(mode: str, src: str, dst: str, degrees: int | None) -> int:
    """한글로 src 를 열어 (선택적으로 회전을 바꾸고) dst 로 저장한다.

    자식 프로세스 하나가 문서 하나를 맡는다 — COM 오염 격리.
    """
    import win32com.client

    kill_hangul()

    com = win32com.client.Dispatch("HWPFrame.HwpObject")
    try:
        com.SetMessageBoxMode(0x00020000)
        try:
            com.RegisterModule("FilePathCheckDLL", "FilePathCheckerModule")
        except Exception:  # noqa: BLE001
            print("WARN\tFilePathCheckerModule 미등록 — 접근 확인 대화상자가 뜰 수 있다")
        try:
            com.XHwpWindows.Item(0).Visible = False
        except Exception:  # noqa: BLE001
            pass

        print(f"VERSION\t{hangul_version(com)}")

        if not com.Open(os.path.abspath(src), "", ""):
            print("RESULT\tOPEN_FAIL")
            return 1

        if mode == "set-rotation":
            if not select_first_shape(com):
                print("RESULT\tNO_SHAPE")
                return 1
            if not set_rotation(com, int(degrees or 0)):
                print("RESULT\tROTATE_FAIL")
                return 1

        if not com.SaveAs(os.path.abspath(dst), "HWP", ""):
            print("RESULT\tSAVE_FAIL")
            return 1
        print(f"RESULT\t{'OK' if os.path.exists(dst) else 'MISSING'}")
        return 0
    finally:
        try:
            com.Clear(1)  # hwpDiscard — 저장 확인창이 종료를 막는 것을 피한다
        except Exception:  # noqa: BLE001
            pass
        try:
            com.Quit()
        except Exception:  # noqa: BLE001
            pass
        kill_hangul()


def select_first_shape(com) -> bool:
    """문서의 첫 그림/도형을 선택 상태로 만든다.

    커서를 밀며 `FindCtrl` 하는 방식은 **표 셀 안 개체를 놓친다**(실측: 셀 안 회전 그림
    표본에서 NO_SHAPE). `HeadCtrl` → `Next` 체인은 셀 내부까지 전부 훑으므로 그쪽을 쓰고,
    찾은 컨트롤의 앵커로 커서를 옮긴 뒤 선택한다.
    """
    try:
        ctrl = com.HeadCtrl
    except Exception as exc:  # noqa: BLE001
        print(f"WARN\tHeadCtrl 접근 실패: {exc}")
        return False

    while ctrl is not None:
        ctrl_id = ""
        try:
            ctrl_id = str(ctrl.CtrlID).strip()
        except Exception:  # noqa: BLE001
            pass
        # gso = 그리기 개체(그림 포함). 한컴 컨트롤 ID 4바이트 코드.
        if ctrl_id == "gso":
            try:
                com.SetPosBySet(ctrl.GetAnchorPos(0))
                com.Run("SelectCtrlFront")
                return True
            except Exception as exc:  # noqa: BLE001
                print(f"WARN\t개체 선택 실패(id={ctrl_id}): {exc}")
                return False
        try:
            ctrl = ctrl.Next
        except Exception:  # noqa: BLE001
            return False
    print("WARN\t문서에 gso 컨트롤이 없다")
    return False


# 개체 속성 액션·회전 필드 이름은 한글 버전마다 다르다. 하나를 못박으면 다른 버전에서
# 조용히 실패하고, 조용한 실패는 "회전을 바꿨다" 는 거짓 전제로 표를 만든다. 후보를 훑고
# **어느 조합이 들었는지 기록**한다.
ROTATE_ACTIONS = ("ShapeObjDialog", "ShapeObjectDialog", "ShapeObjPropertyDialog")
ROTATE_FIELDS = ("RotateAngle", "RotAngle", "Rotation")


def set_rotation(com, degrees: int) -> bool:
    """선택된 개체의 회전각을 degrees 로 바꾼다. 성공한 조합을 stdout 에 남긴다."""
    for action in ROTATE_ACTIONS:
        try:
            pset = com.CreateSet("ShapeObject")
        except Exception as exc:  # noqa: BLE001
            print(f"WARN\tCreateSet(ShapeObject) 실패: {exc}")
            return False
        try:
            com.HAction.GetDefault(action, pset)
        except Exception as exc:  # noqa: BLE001
            print(f"WARN\tGetDefault({action}) 실패: {exc}")
            continue
        for field in ROTATE_FIELDS:
            try:
                pset.SetItem(field, degrees)
            except Exception:  # noqa: BLE001
                continue
            try:
                if com.HAction.Execute(action, pset):
                    print(f"ACTION\t{action}/{field}")
                    return True
            except Exception as exc:  # noqa: BLE001
                print(f"WARN\tExecute({action}/{field}) 예외: {exc}")
    print(
        "WARN\t회전 액션 후보를 모두 시도했으나 듣지 않았다 — "
        f"actions={ROTATE_ACTIONS} fields={ROTATE_FIELDS}"
    )
    return False


def run_child(mode: str, src: str, dst: str, degrees, timeout: int) -> tuple[str, str]:
    """자식 프로세스로 한글 작업 1건. 반환: (결과, 한글버전)."""
    argv = [sys.executable, __file__, "--child", mode, src, dst]
    if degrees is not None:
        argv.append(str(degrees))
    try:
        proc = subprocess.run(
            argv, capture_output=True, text=True, encoding="utf-8",
            errors="replace", timeout=timeout,
        )
    except subprocess.TimeoutExpired:
        kill_hangul()
        return "TIMEOUT", "unknown"
    result, version = "NO_RESULT", "unknown"
    for line in (proc.stdout or "").splitlines():
        if line.startswith("RESULT\t"):
            result = line.split("\t", 1)[1].strip()
        elif line.startswith("VERSION\t"):
            version = line.split("\t", 1)[1].strip()
        elif line.startswith(("WARN\t", "ACTION\t")):
            print("  " + line)
    return result, version


# ---------------------------------------------------------------- 모드


def survey(exe: str, paths: list[str], detail: bool) -> int:
    """COM 없이 한컴 저장본의 `flip` 워드를 전수 조사한다.

    특정 비트를 가정하지 않는다 — **32비트 전부**에 대해 "회전됨" 과의 상관을 세서,
    회전을 따라 움직이는 비트가 어느 것인지 데이터가 말하게 한다. 처음에 bit19 를
    회전 표식으로 짚었다가 실측에서 뒤집힌 전례가 있다.
    """
    if detail:
        print("파일\t개체#\t각도\tflip\trotateImage\t뒤집기")
    rows: list[tuple[str, int, Transform]] = []
    for path in paths:
        for i, t in enumerate(transforms(exe, path)):
            rows.append((os.path.basename(path), i, t))
            if detail:
                print(
                    f"{os.path.basename(path)}\t{i}\t{t.angle}\t0x{t.flip:08x}"
                    f"\t{int(t.rotate_image)}\t({int(t.horz_flip)},{int(t.vert_flip)})"
                )

    rotated = [t for _, _, t in rows if t.rotated]
    flat = [t for _, _, t in rows if not t.rotated]
    print()
    print(f"=== 표본: 개체 {len(rows)}건 (회전!=0 {len(rotated)}건, 회전==0 {len(flat)}건) ===")
    if not rotated or not flat:
        print("판정 불가 — 회전 개체와 비회전 개체가 모두 있어야 상관을 볼 수 있다.")
        return 0

    print()
    print("비트\t회전&켜짐\t회전&꺼짐\t평평&켜짐\t평평&꺼짐\t판정")
    verdict_bits: list[int] = []
    for bit in range(32):
        mask = 1 << bit
        r_on = sum(1 for t in rotated if t.flip & mask)
        r_off = len(rotated) - r_on
        f_on = sum(1 for t in flat if t.flip & mask)
        f_off = len(flat) - f_on
        if r_on == 0 and f_on == 0:
            continue  # 어디에도 안 나타나는 비트는 생략
        if r_on == len(rotated) and f_on == 0:
            verdict = "회전과 정확히 일치"
            verdict_bits.append(bit)
        elif r_on == 0 and f_on == len(flat):
            verdict = "비회전과 정확히 일치(반대 방향)"
        elif f_on == 0:
            verdict = "회전에서만 나타남(부분)"
        elif r_on == 0:
            verdict = "비회전에서만 나타남(부분)"
        else:
            verdict = "무관"
        print(
            f"bit{bit}(0x{mask:08x})\t{r_on}\t{r_off}\t{f_on}\t{f_off}\t{verdict}"
        )

    print()
    if verdict_bits:
        names = ", ".join(f"bit{b}(0x{1 << b:08x})" for b in verdict_bits)
        print(f"판정: 회전 상태와 정확히 일치하는 비트 — {names}")
    else:
        print("판정: 회전 상태와 정확히 일치하는 비트가 없다.")
    print("      rhwp 가 회전 편집에서 세우는 비트가 위 목록에 없다면, 그 비트는 회전")
    print("      표식이 아니므로 회전을 근거로 세우거나 지워선 안 된다.")
    return 0


def compare(exe: str, mode: str, paths: list[str], degrees, timeout: int) -> int:
    """한글 저장 전/후를 대조한다."""
    label = "그대로 저장" if mode == "resave" else f"회전={degrees} 적용 후 저장"
    print(f"=== 한글 오라클: {label} ===")
    print("파일\t그림#\t한글전\t한글후\t판정")
    for path in paths:
        before = transforms(exe, path)
        with tempfile.TemporaryDirectory() as td:
            dst = os.path.join(td, "hangul_saved.hwp")
            result, version = run_child(mode, path, dst, degrees, timeout)
            print(f"# 한글 버전: {version}, 결과: {result}")
            if result != "OK":
                print(f"{os.path.basename(path)}\t-\t-\t-\t{result}")
                continue
            after = transforms(exe, dst)

        if len(before) != len(after):
            print(
                f"{os.path.basename(path)}\t-\t{len(before)}개\t{len(after)}개\t그림수불일치"
            )
            continue
        for i, (b, a) in enumerate(zip(before, after)):
            print(f"{os.path.basename(path)}\t{i}\t{b}\t{a}\t{verdict_of(b, a)}")
    return 0


def verdict_of(before: Transform, after: Transform) -> str:
    """한글 저장 전/후 차이를 사람이 읽을 판정으로. 어느 비트가 움직였는지 밝힌다."""
    if before == after:
        return "무변화"
    parts: list[str] = []
    if before.angle != after.angle:
        parts.append(f"각도 {before.angle}->{after.angle}")
    changed = before.flip ^ after.flip
    if changed:
        bits = ", ".join(
            f"bit{b}({'꺼짐->켜짐' if after.flip & (1 << b) else '켜짐->꺼짐'})"
            for b in range(32)
            if changed & (1 << b)
        )
        parts.append(f"flip {bits}")
    if before.rotate_image != after.rotate_image:
        parts.append(
            f"rotateImage {int(before.rotate_image)}->{int(after.rotate_image)}"
        )
    return "변화: " + "; ".join(parts) if parts else "변화"


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--exe", default=os.path.join("target", "release", "rhwp.exe"))
    ap.add_argument("--timeout", type=int, default=180)
    ap.add_argument("--survey", nargs="*", metavar="PATH")
    ap.add_argument(
        "--list",
        dest="lst",
        help="경로 목록 파일(한 줄에 하나). 표본이 많으면 커맨드라인 길이 한계에 걸린다",
    )
    ap.add_argument("--detail", action="store_true", help="--survey 에서 개체별 줄도 낸다")
    ap.add_argument("--resave", nargs="+", metavar="PATH")
    ap.add_argument("--set-rotation", nargs="+", metavar=("DEG", "PATH"))
    ap.add_argument("--child", nargs="+", metavar="ARG")
    a = ap.parse_args()

    if a.child:
        mode, src, dst = a.child[0], a.child[1], a.child[2]
        deg = int(a.child[3]) if len(a.child) > 3 else None
        return child(mode, src, dst, deg)

    if not os.path.exists(a.exe):
        print(f"rhwp 실행 파일이 없다: {a.exe}", file=sys.stderr)
        print("cargo build --release --bin rhwp 로 만들고 --exe 로 지정한다.", file=sys.stderr)
        return 2

    from_list: list[str] = []
    if a.lst:
        with open(a.lst, encoding="utf-8") as handle:
            from_list = [ln.strip() for ln in handle if ln.strip()]

    if a.survey is not None:
        return survey(a.exe, list(a.survey) + from_list, a.detail)
    if a.resave:
        return compare(a.exe, "resave", a.resave, None, a.timeout)
    if a.set_rotation:
        return compare(
            a.exe, "set-rotation", a.set_rotation[1:], int(a.set_rotation[0]), a.timeout
        )
    ap.error("--survey / --resave / --set-rotation 중 하나가 필요하다")
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
