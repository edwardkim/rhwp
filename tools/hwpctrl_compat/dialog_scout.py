""""안 끝남"으로 분류된 액션이 정말 **대화상자** 때문인지 UI Automation 으로 확정한다.

## 왜 있나

`classify_remaining.py` 는 시한초과로 "안 끝남"을 세는데, 그 안엔 진짜 대화상자(창을 몰면
결정적으로 끝낼 수 있어 언젠가 열릴 후보)와 다른 이유로 안 끝나는 것이 섞여 있다. COM
`Run(액션)` 은 대화상자를 띄우면 **닫힐 때까지 블록**하므로, 액션을 스레드에서 걸고 주
스레드에서 뜬 모달 창의 **제목**을 잡아 닫는다. 제목이 잡히면 대화상자, 시한 안에 창이 안
뜨고 스레드도 안 끝나면 "창 없이 안 끝남"이다.

**이것은 분류 도구다** — 원장을 올리지 않는다. 대화상자를 기본값으로 몰아 결과를 재는 것은
다음 단계(저장본 차분)이고, 이 스카우트는 "어느 액션이 어떤 창을 띄우는가"만 확정한다.

## 재현성 경고

오라클 프로세스에 입력을 넣는 순간 결과가 실행 환경(창 관리자·포커스·타이밍)의 함수가 된다.
그래서 이 도구의 산출물은 **분류 근거**(창 제목)일 뿐, 게이트 정답지가 아니다. 창은
**이름/제어형**으로 찾는다 — 좌표 클릭은 해상도·DPI 에 깨진다.

## 쓰임

    python tools/hwpctrl_compat/dialog_scout.py --context object ShapeObjDialog
    python tools/hwpctrl_compat/dialog_scout.py --context cellblock TablePropertyDialog

동시에 다른 오라클을 돌리지 말 것. 실행 전 `Get-Process Hwp | Stop-Process -Force`.
"""

from __future__ import annotations

import argparse
import io
import json
import sys
import threading
import time
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
SAMPLE_SHAPE = "samples/mix-shape-01.hwp"
SAMPLE_TABLE = "samples/21868765_별표2_보건소_분장사무.hwp"


def _open(com, rel: str) -> None:
    """3-인자 Open + EditMode 확인 — 읽기 전용으로 열리면 편집이 조용히 무시된다(§4.24)."""
    com.Open(str(REPO / rel), "", "")
    if com.EditMode != 1:
        raise SystemExit(
            f"읽기 전용으로 열렸다(EditMode {com.EditMode}) — 남은 Hwp 를 죽이고 다시 하라"
        )


def build_context(com, context: str) -> None:
    """액션이 뜻을 가지려면 필요한 상태를 만든다 — 대화상자는 맥락을 타는 것이 많다(§4.73)."""
    if context == "object":
        _open(com, SAMPLE_SHAPE)
        com.SetPos(0, 0, 0)
        com.Run("SelectCtrlFront")
        desc = getattr(com.CurSelectedCtrl, "UserDesc", None) if com.CurSelectedCtrl else None
        print(f"[scout] 고른 개체: {desc}", file=sys.stderr)
        if not desc:
            raise SystemExit("개체 선택 실패 — 맥락이 안 섰다")
    elif context == "cellblock":
        _open(com, SAMPLE_TABLE)
        com.SetPos(2, 0, 0)
        com.Run("TableCellBlock")
    elif context == "caret":
        _open(com, SAMPLE_TABLE)
        com.SetPos(2, 0, 0)
    else:
        raise SystemExit(f"모르는 맥락: {context}")


def find_and_close_dialog(pid: int, deadline: float) -> str | None:
    """그 프로세스가 띄운 모달 창을 찾아 제목을 돌려주고 닫는다. 없으면 None."""
    from pywinauto import Desktop

    while time.time() < deadline:
        try:
            for win in Desktop(backend="win32").windows():
                try:
                    if win.process_id() != pid:
                        continue
                except Exception:  # noqa: BLE001
                    continue
                title = win.window_text()
                # 문서 창(제목이 파일명이거나 빈 것)은 건너뛴다 — 모달만 본다.
                if not title or title.endswith(".hwp") or "한글" in title and "-" in title:
                    continue
                if not win.is_visible():
                    continue
                # 이름으로 닫는다 — 취소/닫기 버튼, 없으면 ESC.
                closed = False
                for name in ("취소", "닫기", "Cancel", "Close"):
                    try:
                        win.child_window(title=name, control_type="Button").click()
                        closed = True
                        break
                    except Exception:  # noqa: BLE001
                        continue
                if not closed:
                    try:
                        win.type_keys("{ESC}")
                    except Exception:  # noqa: BLE001
                        pass
                return title
        except Exception:  # noqa: BLE001
            pass
        time.sleep(0.3)
    return None


def scout(action: str, context: str, timeout: float) -> dict:
    import win32process  # noqa: F401  (pywinauto 가 끌어온다)
    from pyhwpx import Hwp

    hwp = Hwp(new=True, visible=True)  # 창을 몰려면 보여야 한다
    com = hwp.hwp
    result = {"action": action, "context": context, "dialogTitle": None,
              "threadFinished": None, "error": None}
    try:
        build_context(com, context)
        try:
            pid = com.XHwpWindows.Item(0).WindowHandle  # 핸들에서 pid 유도 실패 시 아래로
        except Exception:  # noqa: BLE001
            pid = None
        import win32gui
        import win32process as w32p
        if pid is not None:
            try:
                _, pid = w32p.GetWindowThreadProcessId(pid)
            except Exception:  # noqa: BLE001
                pid = None
        if pid is None:
            # 최상위 창을 훑어 한글 프로세스 pid 를 찾는다.
            found = {}

            def enum(hwnd, _):
                text = win32gui.GetWindowText(hwnd)
                if text.endswith(".hwp") or "HwpObject" in text:
                    _, p = w32p.GetWindowThreadProcessId(hwnd)
                    found["pid"] = p

            win32gui.EnumWindows(enum, None)
            pid = found.get("pid")

        # **액션은 주 스레드에서 건다**(COM STA). 대화상자가 뜨면 `Run` 이 블록하므로, 창을
        # 찾아 닫는 감시자는 **자기 COM 초기화를 가진 별도 스레드**에 둔다.
        deadline = time.time() + timeout
        watcher_result: dict = {"title": None}

        def watch():
            import pythoncom

            pythoncom.CoInitialize()
            try:
                watcher_result["title"] = find_and_close_dialog(pid, deadline) if pid else None
            finally:
                pythoncom.CoUninitialize()

        watcher = threading.Thread(target=watch, daemon=True)
        watcher.start()

        fired = threading.Event()

        def fire_flag():
            # `Run` 이 반환하면(대화상자가 닫혔거나 애초에 창이 없었으면) 표시한다.
            fired.set()

        try:
            com.Run(action)
            fire_flag()
        except Exception as exc:  # noqa: BLE001
            result["error"] = f"{type(exc).__name__}: {exc}"
            fire_flag()

        watcher.join(timeout=max(1.0, deadline - time.time()))
        result["dialogTitle"] = watcher_result["title"]
        # 창을 닫아 `Run` 이 반환했으면 대화상자였다. 창 없이 곧장 반환했으면 대화상자가 아니다.
        result["threadFinished"] = fired.is_set()
    finally:
        try:
            com.Quit()
        except Exception:  # noqa: BLE001
            pass
    return result


def main() -> int:
    sys.stdout.reconfigure(encoding="utf-8")
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("action", help="확인할 액션 이름 (예: ShapeObjDialog)")
    ap.add_argument("--context", default="caret", choices=["object", "cellblock", "caret"])
    ap.add_argument("--timeout", type=float, default=8.0, help="창이 뜨길 기다리는 초")
    args = ap.parse_args()

    res = scout(args.action, args.context, args.timeout)
    print(json.dumps(res, ensure_ascii=False, indent=2))
    if res["dialogTitle"]:
        print(f'→ 대화상자 확정: "{res["dialogTitle"]}"')
        return 0
    if res["threadFinished"]:
        print("→ 창 없이 끝났다 — 대화상자가 아니다(맥락을 의심하라)")
        return 2
    print("→ 창도 못 찾고 스레드도 안 끝났다 — 창 없이 안 끝남")
    return 3


if __name__ == "__main__":
    raise SystemExit(main())
