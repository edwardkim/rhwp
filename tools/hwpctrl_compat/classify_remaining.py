"""남은 원장 항목을 **왜 못 올렸는지**로 갈래 지어 도달 가능한 상한을 낸다.

원장 484 가 도달 가능한 상한이 아니다 — 아예 답을 안 주고 안 끝나는 액션이 있고, 이 빌드에
없는 API 도 있다. 그 선을 숫자로 그어 두면 다음 사람이 "남은 것을 다 할 수 있다"는 잘못된
기대로 시간을 태우지 않는다.

    python tools/hwpctrl_compat/classify_remaining.py

## 이 도구는 **입력이 있어야** 확정값을 낸다

갈래 중 "안 끝남"·"대화상자" 는 Windows 에서 액션을 하나씩 걸어 본 **스윕 산출물**로만
가린다. 그 파일들은 `output/` 아래에 생기고 Git 에 담기지 않는다. 없으면 그 항목들이 조용히
"아직 안 잼" 으로 새어 **상한이 부풀어 오른다** — 같은 head 에서 계획서·README 는 312 인데
스윕 없는 기계에서 돌리면 366 이 나왔다(#4274 리뷰의 세 번째 지적).

그래서 입력을 먼저 점검하고, 없으면 그 숫자를 **확정값이라고 부르지 않는다.**
"""

from __future__ import annotations

import io
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]
LEDGER = REPO / "npm" / "hwpctrl-ocx" / "spec" / "api_ledger.json"
SWEEPS = [
    REPO / "output" / "poc" / "hwpctrl" / "sweep_actions.tsv",
    REPO / "output" / "poc" / "hwpctrl" / "sweep_shapeobj.tsv",
    REPO / "output" / "poc" / "hwpctrl" / "sweep_table.tsv",
    REPO / "output" / "poc" / "hwpctrl" / "sweep_cellblock.tsv",
]

# 이 COM 개체에 **아예 없는** 것들(AttributeError 실측).
ABSENT = {
    "HwpCtrl.method.GetTableCellAddr", "HwpCtrl.method.GetViewStatus",
    "HwpCtrl.property.ScrollPosInfo", "HwpCtrl.property.ReadOnlyMode",
    "HwpCtrl.method.GetCtrlHorizontalOffset", "HwpCtrl.method.GetCtrlVerticalOffset",
    "HwpCtrl.method.GetTextBySet", "HwpCtrl.method.SaveDocument",
    "HwpCtrl.method.MoveToFieldEx", "HwpCtrl.method.OpenDocument",
    "ParameterSet.method.GetInterSection", "ParameterSet.DrawLayout",
    # 이벤트 계열과 대화상자 띄우기 — 전부 `AttributeError`(계획서 §4.65).
    "HwpCtrl.method.AddEventListener", "HwpCtrl.event.OnMouseLButtonDown",
    "HwpCtrl.event.OnMouseLButtonUp", "HwpCtrl.event.OnScroll",
    "Action.method.PopupDialog", "HwpCtrl.method.InsertDocument",
    # `UI 전용` 으로 세던 것들인데 재 보니 **아예 없다**(계획서 §4.69). 딱지가 틀렸었다.
    "HwpCtrl.method.ShowCaret", "HwpCtrl.method.ShowRibbon", "HwpCtrl.method.ShowStatusBar",
    "HwpCtrl.method.ShowToolBar", "HwpCtrl.method.ShowHorizontalScroll",
    "HwpCtrl.method.ShowVerticalScroll", "HwpCtrl.method.SetToolBar",
    "HwpCtrl.method.IsSpellCheckCompleted", "HwpCtrl.method.CreatePageImageEx",
}

# 창·기계에 달려 관측이 성립하지 않는 것들 — **실측으로** 이유를 붙였다(계획서 §4.69).
UI_ONLY = {
    # 물리 마우스 자리를 준다(`MousePos` 셋의 `X`·`Y` 가 1743332·1539150 따위) — 부를 때마다
    # 다르고 이 층에는 마우스가 없다.
    "HwpCtrl.method.GetMousePos",
    # 화면 크기가 정한다: 쪽 맞춤 **37%**, 폭 맞춤 **139%**(창 크기·DPI 에 달렸다).
    # `ViewZoomNormal` 만 100 으로 상수인데, 셋을 가르려면 나머지 둘이 필요하다.
    "Action.ViewZoomFitPage", "Action.ViewZoomFitWidth", "Action.ViewZoomNormal",
    # 덮어쓰기 토글은 `KeyIndicator` 의 `over` 를 1↔0 으로 뒤집는 것으로만 보이는데,
    # 그 API 는 `pos` 때문에 통째로 못 올린다(§4.67).
    "Action.ToggleOverwrite",
    "HwpCtrl.method.PrintDocument",
}

# **글자 폭(advance)이 한글과 같아야** 잴 수 있는 것들 — 조판 정밀도가 아니라 글꼴 계측이다
# (계획서 §4.67). `KeyIndicator` 의 `pos` 와 캐럿 들여쓰기가 캐럿의 **x** 에서 나온다.
GLYPH_METRICS = {
    "HwpCtrl.method.KeyIndicator",
    "Action.ParagraphShapeIndentAtCaret",
}

# 조판(쪽·줄)이 맞아야 잴 수 있는 것들.
LAYOUT = {
        "Action.MoveLineUp", "Action.MoveLineDown",
    "Action.MoveUp", "Action.MoveDown",     "Action.MoveViewBegin", "Action.MoveViewEnd",
    "Action.MoveViewUp", "Action.MoveViewDown", "Action.MoveScrollUp", "Action.MoveScrollDown",
    "Action.MoveScrollNext", "Action.MoveScrollPrev",         "Action.MoveSelLineUp", "Action.MoveSelLineDown", "Action.MoveSelUp", "Action.MoveSelDown",
    "Action.MoveSelViewUp",
    "Action.MoveSelViewDown",
}

# **관측창이 없는** 것들 — 맥락은 섰는데 어느 API 도 결과를 안 비춘다(계획서 §4.58).
#
# 무동작으로 뭉뚱그리지 않고 따로 세는 이유: 무동작은 "맥락을 더 찾으면 될지 모른다"이고
# 이것은 "찾아도 볼 창이 없다"라 성격이 다르다.
#
# **이 갈래의 전제가 절반쯤 무너졌다.** "파일에는 적히지만 이 게이트가 대조하는 것은 API
# 반환이라"고 적어 두고는 그 파일을 열어 보지 않았다. 열어 보니 **적혀 있고 읽을 수도
# 있다** — 액션 앞뒤로 저장한 두 벌을 `rhwp ir-sweep` 으로 견주면 `common.z_order` 가 1↔2 로
# 뒤바뀐 것, `render_sx` 가 1.0 → −1.0 이 된 것, 표의 `col_count` 가 3 → 4 가 된 것,
# 지워진 칸의 글이 사라진 것이 그대로 나온다. 액션 없이 두 번 저장하면 차이는 **0** 이라
# 잡음 바닥도 없다.
#
# 그래서 아래 `SAVE_OBSERVABLE` 로 옮긴 것들은 **막힌 것이 아니라 아직 구현이 없는 것**이다.
# 상한에서 빼지 않는다. 남은 `NO_WINDOW` 는 저장본으로도 안 보이거나(한글이 조용히 아무 일도
# 안 하는 것) 오라클을 죽여 정답지를 못 만드는 것들이다.
#
# `ir-diff` 가 아니라 `ir-sweep` 인 이유: `ir-diff` 의 비교 목록은 손으로 쌓은 것이라
# `z_order` 도 변환 행렬도 아예 안 본다. 같은 파일 쌍에 "동일" 이라 답했다.
SAVE_OBSERVABLE = {
    # z-순서 — `CTRL_HEADER` 에 적히고 앞뒤 저장본에서 1↔2 교환이 보인다.
    "Action.ShapeObjBringToFront", "Action.ShapeObjSendToBack",
    "Action.ShapeObjBringForward", "Action.ShapeObjSendBack",
    "Action.ShapeObjBringInFrontOfText", "Action.ShapeObjCtrlSendBehindText",
    # 뒤집기 — `SHAPE_COMPONENT` 의 변환 행렬(`render_sx` 1.0 → −1.0)과 `horz_flip`.
    "Action.ShapeObjHorzFlip", "Action.ShapeObjVertFlip",
    "Action.ShapeObjHorzFlipOrgState", "Action.ShapeObjVertFlipOrgState",
    # 표 칸 조절 — `CellShape` 는 안 움직여도 저장본은 움직인다(`col_count` 3 → 4,
    # `cell_grid` 재구성, `segment_width`, 표 레코드 attr).
    "Action.TableResizeCellDown", "Action.TableResizeCellLeft",
    "Action.TableResizeCellRight", "Action.TableResizeCellUp",
    "Action.TableResizeDown", "Action.TableResizeLeft",
    "Action.TableResizeRight", "Action.TableResizeUp",
    "Action.TableResizeExDown", "Action.TableResizeExLeft",
    "Action.TableResizeExRight", "Action.TableResizeExUp",
    "Action.TableResizeLineDown", "Action.TableResizeLineLeft",
    "Action.TableResizeLineRight", "Action.TableResizeLineUp",
    # 칸 지우기 — "지워진 자취가 어디에도 안 남는다"고 적었는데 **틀렸다**. 칸의 글이
    # 지워진 것이 저장본에 그대로 있다("부 서 명" → "").
    "Action.TableDeleteCell",
}

NO_WINDOW = {
    # 걸면 **한글이 죽는다**(COM 서버가 사라져 그 뒤 호출이 전부 RPC 오류). 정답지를 못 만든다.
    "Action.TableStringToTable", "Action.CellBorder", "Action.CellBorderFill",
    # 칸 나눔 — 유일한 창인 표 셋(`CellShape` 은 이름과 달리 **표**의 셋이다)이 한 박자 늦게
    # 답하고 같은 표를 연달아 읽어도 값이 다르다(31679 → 33171). 정답지가 안 선다.
    "Action.TableDistributeCellWidth", "Action.TableDistributeCellHeight",
    # 캐럿이 누름틀 안인데도 안 지워진다.
    "Action.DeleteField",
    # 찾기·바꾸기·맞춤법 대화상자 — COM 으로 걸면 **조용히 아무 일도 안 한다**(계획서 §4.73).
    # 몇 초에 끝나고, 한글을 보이게 띄워 창을 세어도 **문서 창 하나뿐**이라 대화상자가 안 뜬다.
    "Action.FindDlg", "Action.ReplaceDlg", "Action.SpellingCheck",
    # 쪽 배경 그림 — **성공하는 길을 못 찾았다**(계획서 §4.72). 인자는 둘 이상이어야 하는데
    # (하나면 `필수 매개 변수`), 열넷 가지 인자 꼴·칸 블록·표 고름·jpg/png, 그리고 **한글이
    # 스스로 만든 bmp** 까지 전부 `false` 다. 없는 파일도 같은 `false` 라 실패 이유조차 안 준다.
    "HwpCtrl.method.InsertBackgroundPicture",
    # 액션 바꿔치기 — `true` 를 주는데 바꾼 대로 안 돈다(`MoveDocEnd` 를 `MoveDocBegin` 으로
    # 바꿔도 문서 끝으로 간다). 반환 말고는 볼 것이 없다.
    "HwpCtrl.method.ReplaceAction",
    # 개요 문자열 — 문서 셋에서 전부 빈 문자열이라 양성 사례를 못 찾았다.
    "HwpCtrl.method.GetHeadingString",
    # 캐럿 자리에 글을 넣는다는데 문자열도 셋도 조용히 아무 일도 안 한다.
    "HwpCtrl.method.Insert",
}

# 세로 이동 — **숨은 x 닻**이 막는다(계획서 §4.63). `SetPos` 는 세로 이동이 딛는 x 를 안 바꾸고
# (시작 자리를 뭘로 주든 답이 같다) 그 x 는 문서를 연 뒤 **첫 자리 지정**이 정한다. API 로는
# 읽지도 정하지도 못하니 조판 정밀도만 올려서는 안 열린다. 닻을 맞춰도 x → 글자 번호가 폰트
# advance 로 한 칸씩 갈린다.
HIDDEN_STATE = {
    # 찾기·치환 — 찾을 말이 **대화상자에 남은 상태**라 API 로는 못 정한다. 상태가 없으면
    # `AllReplace` 는 예외로 죽고 `BackwardFind` 는 조용히 아무 일도 안 한다(실측).
    "Action.AllReplace", "Action.BackwardFind",
    "Action.MoveUp", "Action.MoveDown", "Action.MoveLineUp", "Action.MoveLineDown",
    "Action.MoveSelUp", "Action.MoveSelDown", "Action.MoveSelLineUp", "Action.MoveSelLineDown",
}

# **맥락을 API 로 만들 수 없는** 것들 — 맞추기는 개체를 여럿 골라야 먹는데, `SelectAll` 은
# 개체 선택을 버리고(모드 4→1) `SelectCtrl` 은 COM 에 아예 없다. 여럿 고르기는 마우스의 일이다.
UNBUILDABLE_CONTEXT = {
    # 화면·스크롤 이동 — OCX 의 **뷰포트**가 정하는 값이다. 이 층에는 보이는 창이 없어
    # 만들 수 있는 맥락이 아니다(계획서 §4.62).
    "Action.MoveViewBegin", "Action.MoveViewEnd", "Action.MoveViewUp", "Action.MoveViewDown",
    "Action.MoveScrollUp", "Action.MoveScrollDown", "Action.MoveScrollNext", "Action.MoveScrollPrev",
    "Action.MoveSelViewUp", "Action.MoveSelViewDown",
    "Action.ShapeObjAlignLeft", "Action.ShapeObjAlignRight", "Action.ShapeObjAlignTop",
    "Action.ShapeObjAlignBottom", "Action.ShapeObjAlignCenter", "Action.ShapeObjAlignMiddle",
    "Action.ShapeObjAlignWidth", "Action.ShapeObjAlignHeight", "Action.ShapeObjAlignSize",
    "Action.ShapeObjAlignHorzSpacing", "Action.ShapeObjAlignVertSpacing",
}

# 머신(설치 글꼴)에 달린 것들.
MACHINE = {
    "Action.CharShapeNextFaceName", "Action.CharShapePrevFaceName",
}

# **막힌 것이 아니라 아직 자료가 없는 것들** — 세는 자리를 옮긴다(계획서 §4.68).
#
# 음·양력 넷을 오래 "머신 의존"으로 세고 있었는데 그 딱지가 틀렸다. 재 보니 **결정적인 표
# 조회**다 — 같은 날짜에 늘 같은 답이 나오고 범위도 뚜렷하다(음력 1841~2043). 머신 상태가
# 아니라 rhwp 에 **음력 표가 없을 뿐**이다. 구조적으로 막힌 것으로 세면 상한이 거짓이 된다.
#
# 다만 표를 **오라클에서 뽑아 만들면 검증이 순환**이 된다(한글로 만든 표를 한글로 검증). 표는
# 한국천문연구원 같은 독립된 출처에서 와야 하고, 그때 이 넷이 열린다.
#
# **열렸다.** 한국천문연구원 자료를 공공데이터포털 API 로 받아 표를 만들어 넷을 구현했다.
# 다만 그 표는 한글의 표와 35일(1.71%)이 어긋나므로 오라클이 판정자가 될 수 없다 —
# 원장에서 `substituted` 다. 그래서 이 갈래는 이제 비어 있다.
DATA_MISSING: set[str] = set()


HANG_DIALOGS = REPO / "output" / "poc" / "hwpctrl" / "hang_dialogs.tsv"


def missing_inputs() -> list[str]:
    """갈래를 가르는 데 필요한데 지금 없는 산출물. 있으면 빈 목록이다."""
    absent = [str(path.relative_to(REPO)) for path in SWEEPS if not path.exists()]
    if not HANG_DIALOGS.exists():
        absent.append(str(HANG_DIALOGS.relative_to(REPO)))
    return absent


def dialog_titles() -> dict[str, str]:
    """대화상자를 띄우는 액션과 그 **제목** — 창을 열거해 받은 실측이다.

    창 클래스로는 문서 창과 대화상자를 못 가른다(둘 다 `HwndWrapper[Hwp.exe;;<GUID>]`).
    제목으로 갈라야 한다.
    """
    if not HANG_DIALOGS.exists():
        return {}
    out: dict[str, str] = {}
    for line in io.open(HANG_DIALOGS, encoding="utf-8").read().splitlines()[1:]:
        parts = line.split("\t")
        if len(parts) >= 3 and parts[2].strip():
            out[parts[0]] = parts[2].split(" | ")[0]
    return out


DIALOG_TITLES = dialog_titles()


def sweep_kinds() -> dict[str, str]:
    """스윕이 낸 갈래. **안 건 것을 `HANG` 으로 채우지 않는다.**

    예전에는 금지 목록의 이름을 전부 `HANG` 으로 채웠는데, 그 목록에는 재 보지도 않고 짐작으로
    넣은 이름이 섞여 있었다. 그래서 39 개가 "안 끝남"으로 세어졌고 하나씩 걸어 보니 전부
    멀쩡히 끝났다(계획서 §4.70). 이제 **실측으로 확인된 것만** 그렇게 센다.
    """
    from sweep_actions import CONFIRMED_HANG, KILLS_HANGUL

    kinds: dict[str, str] = {name: "HANG" for name in CONFIRMED_HANG}
    kinds.update({name: "KILLS" for name in KILLS_HANGUL})
    for path in SWEEPS:
        if not path.exists():
            continue
        for line in io.open(path, encoding="utf-8").read().splitlines()[1:]:
            parts = line.split("\t")
            if len(parts) < 2:
                continue
            name, kind = parts[0], parts[1]
            # 맥락을 붙여 살아난 것이 있으면 그 결과를 우선한다.
            if kinds.get(name) in ("CHANGED", "MOVED"):
                continue
            kinds[name] = kind
    return kinds


def main() -> int:
    sys.stdout.reconfigure(encoding="utf-8")
    doc = json.loads(LEDGER.read_text(encoding="utf-8"))

    def walk(node):
        if isinstance(node, dict):
            if "id" in node and "status" in node:
                yield node
            for value in node.values():
                yield from walk(value)
        elif isinstance(node, list):
            for value in node:
                yield from walk(value)

    items = list(walk(doc))
    done = [i for i in items if i.get("status") in ("verified", "substituted")]
    rest = [i for i in items if i not in done]
    kinds = sweep_kinds()

    buckets: dict[str, list[str]] = {}
    for item in rest:
        ident = item["id"]
        action = ident.split(".", 1)[1] if ident.startswith("Action.") else None
        if ident in ABSENT:
            key = "없는 API"
        elif ident in UI_ONLY:
            key = "UI 전용(관측 불가)"
        elif ident in MACHINE:
            key = "머신 의존"
        elif ident in SAVE_OBSERVABLE:
            key = "저장본으로 관측 가능(막힌 것 아님)"
        elif ident in NO_WINDOW:
            key = "관측창 없음"
        elif ident in UNBUILDABLE_CONTEXT:
            key = "맥락을 못 만듦"
        elif ident in HIDDEN_STATE:
            key = "숨은 상태"
        elif ident in GLYPH_METRICS:
            key = "글자 폭 정밀도"
        elif ident in DATA_MISSING:
            key = "자료 없음(막힌 것 아님)"
        elif ident in LAYOUT:
            key = "조판 의존"
        elif action and action in DIALOG_TITLES:
            key = f"대화상자({DIALOG_TITLES[action]})" if False else "대화상자"
        elif action and kinds.get(action) == "HANG":
            key = "안 끝남(대화상자 없음)"
        elif action and kinds.get(action) in ("CHANGED", "MOVED"):
            key = "관측됨 — 다음 후보"
        elif action and kinds.get(action) == "NOOP":
            key = "무동작(맥락 더 필요)"
        else:
            key = "아직 안 잼"
        buckets.setdefault(key, []).append(ident)

    print(f"원장 {len(items)} · 완료 {len(done)} · 남은 {len(rest)}\n")
    blocked = 0
    for key in sorted(buckets, key=lambda k: -len(buckets[k])):
        names = buckets[key]
        # 구조적으로 막힌 갈래 — 이 하니스로는 영원히 못 올린다.
        if key in (
            "없는 API",
            "UI 전용(관측 불가)",
            "머신 의존",
            "대화상자",
            "안 끝남(대화상자 없음)",
            "관측창 없음",
            "맥락을 못 만듦",
            "숨은 상태",
            "글자 폭 정밀도",
        ):
            blocked += len(names)
        print(f"  {key:<22} {len(names):>4}")
        if key in ("관측됨 — 다음 후보", "아직 안 잼"):
            for j in range(0, len(names), 3):
                print("      " + "  ".join(f"{n:<34}" for n in names[j : j + 3]))
    reachable = len(items) - blocked
    absent = missing_inputs()
    if absent:
        # 스윕이 없으면 "안 끝남"·"대화상자" 를 못 가려 그 항목들이 "아직 안 잼" 으로 샌다.
        # 그 상태로 나온 수는 상한이 아니라 **상한의 위쪽 한계**다. 확정값이라고 부르지 않는다.
        print("\n입력 없음 — 아래 수는 확정값이 아니다. Windows 에서 스윕을 돌린 뒤 다시 세라.")
        for path in absent:
            print(f"  없음: {path}")
        print(f"막힌 것 {blocked} 이상 → 상한은 **{reachable}/{len(items)} 이하**(확정 아님)")
        print(f"지금 {len(done)}")
        return 1
    print(f"\n구조적으로 막힌 것 {blocked} → **도달 가능한 상한 {reachable}/{len(items)}**")
    print(f"지금 {len(done)} — 상한까지 {reachable - len(done)} 남음")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
