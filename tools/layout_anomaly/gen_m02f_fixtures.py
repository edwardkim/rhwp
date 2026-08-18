#!/usr/bin/env python3
"""M02-f layout-anomaly 판정 픽스처 생성기.

devel `scan_page` 규칙을 파이썬으로 재현해 합성 트리·verdict 행렬·
사람/JSON 성적표를 만든다. 레이아웃 엔진은 건드리지 않는다.
"""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "tests" / "fixtures" / "layout_anomaly_m02f"

CHECKABLE = {
    "Table",
    "Image",
    "TextBox",
    "Equation",
    "Group",
    "Form",
    "Placeholder",
    "RawSvg",
    "Line",
    "Rect",
    "Ellipse",
    "Path",
    "TextLine",
}
CONTENT_TYPES = CHECKABLE - {"TextLine"}
EXCLUSIVE_WRAP = {"Square", "Tight", "TopAndBottom"}
KO_TYPE = {
    "Table": "표",
    "Image": "그림",
    "TextBox": "글상자",
    "Equation": "수식",
    "Group": "묶음",
    "Form": "양식",
    "Placeholder": "자리표시",
    "RawSvg": "RawSvg",
    "Line": "선",
    "Rect": "사각형",
    "Ellipse": "타원",
    "Path": "경로",
    "TextLine": "문단 줄",
    "TextRun": "텍스트 런",
    "Cell": "셀",
    "Column": "단",
}
KO_DIR = {"left": "왼쪽", "top": "위", "right": "오른쪽", "bottom": "아래"}


def node(
    t,
    x,
    y,
    w,
    h,
    *,
    children=None,
    wrap=None,
    master=False,
    visible=True,
    editor_only=False,
    text=None,
    char_overlap=False,
    column=None,
):
    return {
        "t": t,
        "x": float(x),
        "y": float(y),
        "w": float(w),
        "h": float(h),
        "wrap": wrap,
        "master": bool(master),
        "visible": bool(visible),
        "editorOnly": bool(editor_only),
        "text": text,
        "charOverlap": bool(char_overlap),
        "column": column,
        "children": list(children or []),
    }


def box(x, y, w, h):
    return {"x": float(x), "y": float(y), "w": float(w), "h": float(h)}


def visible_text(s):
    return any(not ch.isspace() for ch in (s or ""))


def intersection(a, b):
    x0 = max(a["x"], b["x"])
    y0 = max(a["y"], b["y"])
    x1 = min(a["x"] + a["w"], b["x"] + b["w"])
    y1 = min(a["y"] + a["h"], b["y"] + b["h"])
    if x1 > x0 and y1 > y0:
        return x1 - x0, y1 - y0
    return None


def overflow_amounts(bbox, boundary):
    ol = max(boundary["x"] - bbox["x"], 0.0)
    ot = max(boundary["y"] - bbox["y"], 0.0)
    oright = max(bbox["x"] + bbox["w"] - (boundary["x"] + boundary["w"]), 0.0)
    ob = max(bbox["y"] + bbox["h"] - (boundary["y"] + boundary["h"]), 0.0)
    return ol, ot, oright, ob


def off_canvas_amounts(bbox, page):
    ol = max(page["x"] - bbox["x"], 0.0)
    ot = max(max(page["y"] - bbox["y"], 0.0), max(-bbox["y"], 0.0))
    oright = max(bbox["x"] + bbox["w"] - (page["x"] + page["w"]), 0.0)
    ob = max(bbox["y"] + bbox["h"] - (page["y"] + page["h"]), 0.0)
    return ol, ot, oright, ob


def is_overlap_candidate(n):
    if n["t"] == "TextLine":
        return any(
            c["t"] == "TextRun" and visible_text(c.get("text")) for c in n["children"]
        )
    if n["t"] == "Table":
        return True
    if n.get("master"):
        return False
    return n.get("wrap") in EXCLUSIVE_WRAP


def is_text_overlap_candidate(n):
    if n["t"] != "TextRun":
        return False
    text = n.get("text") or ""
    return (
        not n.get("charOverlap")
        and visible_text(text)
        and n["w"] > 0.0
        and n["h"] > 0.0
    )


def walk(
    n,
    path,
    column,
    suppress,
    off_sup,
    body,
    page,
    opts,
    overflow,
    off_canvas,
    flow,
    text,
    has_content,
):
    if not n.get("visible", True) or n.get("editorOnly"):
        return
    if n["t"] == "TextRun" and visible_text(n.get("text")):
        has_content[0] = True
    if n["t"] in CONTENT_TYPES:
        has_content[0] = True
    if n["t"] == "Column":
        column = n.get("column")
    if is_text_overlap_candidate(n):
        text.append({"path": path, "t": "TextRun", "box": box(n["x"], n["y"], n["w"], n["h"]), "column": column})
    next_sup = suppress
    next_off = off_sup
    if n["t"] in CHECKABLE:
        label = n["t"]
        allowed = opts["types"] is None or label in opts["types"]
        if not off_sup:
            ol, ot, oright, ob = off_canvas_amounts(n, page)
            mx = max(ol, ot, oright, ob)
            if mx > opts["overflowTol"]:
                off_canvas.append(
                    {
                        "path": path,
                        "t": label,
                        "overLeft": ol,
                        "overTop": ot,
                        "overRight": oright,
                        "overBottom": ob,
                        "maxOver": mx,
                    }
                )
            next_off = True
        if not suppress and allowed:
            ol, ot, oright, ob = overflow_amounts(n, body)
            mx = max(ol, ot, oright, ob)
            if mx > opts["overflowTol"]:
                overflow.append(
                    {
                        "path": path,
                        "t": label,
                        "overLeft": ol,
                        "overTop": ot,
                        "overRight": oright,
                        "overBottom": ob,
                        "maxOver": mx,
                    }
                )
            if is_overlap_candidate(n):
                flow.append(
                    {
                        "path": path,
                        "t": label,
                        "box": box(n["x"], n["y"], n["w"], n["h"]),
                        "column": column,
                    }
                )
            next_sup = True
    for i, child in enumerate(n.get("children") or []):
        walk(
            child,
            f"{path}/{child['t']}{i}",
            column,
            next_sup,
            next_off,
            body,
            page,
            opts,
            overflow,
            off_canvas,
            flow,
            text,
            has_content,
        )


def find_overlaps(cands, tol):
    out = []
    for i, a in enumerate(cands):
        for b in cands[i + 1 :]:
            if a["column"] != b["column"]:
                continue
            hit = intersection(a["box"], b["box"])
            if hit and hit[0] > tol and hit[1] > tol:
                out.append(
                    {
                        "pathA": a["path"],
                        "typeA": a["t"],
                        "pathB": b["path"],
                        "typeB": b["t"],
                        "overlapW": hit[0],
                        "overlapH": hit[1],
                    }
                )
    return out


def scan(case):
    page = case["page"]
    body = case["body"]
    opts = case["opts"]
    overflow, off_canvas, flow, text = [], [], [], []
    has_content = [False]
    wrapper = {"t": "Body", "visible": True, "editorOnly": False, "children": case["nodes"]}
    for i, child in enumerate(case["nodes"]):
        walk(
            child,
            f"Page/Body/{child['t']}{i}",
            None,
            False,
            False,
            body,
            page,
            opts,
            overflow,
            off_canvas,
            flow,
            text,
            has_content,
        )
    overlap = find_overlaps(flow, opts["overlapTol"])
    text_overlap = find_overlaps(text, opts["overlapTol"])
    page_i = case["pageIndex"]
    page_n = case["pageCount"]
    empty = page_n >= 3 and 0 < page_i < page_n - 1 and not has_content[0]
    signal = bool(overflow or off_canvas or overlap or text_overlap)
    return {
        "overflow": overflow,
        "offCanvas": off_canvas,
        "overlap": overlap,
        "textOverlap": text_overlap,
        "empty": empty,
        "signal": signal,
        "hasContent": has_content[0],
    }


def report_lines(page_i, scanned):
    lines = []
    for o in scanned["overflow"]:
        lines.append(
            f"  [OVERFLOW] page {page_i:>3}  {o['maxOver']:>7.2f}px  {o['path']} ({o['t']})"
        )
    for o in scanned["offCanvas"]:
        lines.append(
            f"  [OFF-CANVAS] page {page_i:>3}  {o['maxOver']:>7.2f}px  {o['path']} ({o['t']})"
        )
    for o in scanned["overlap"]:
        lines.append(
            f"  [OVERLAP]  page {page_i:>3}  {o['overlapW']:.2f}x{o['overlapH']:.2f}px  "
            f"{o['pathA']} ({o['typeA']}) x {o['pathB']} ({o['typeB']})"
        )
    for o in scanned["textOverlap"]:
        lines.append(
            f"  [TEXT-OVERLAP] page {page_i:>3}  {o['overlapW']:.2f}x{o['overlapH']:.2f}px  "
            f"{o['pathA']} ({o['typeA']}) x {o['pathB']} ({o['typeB']})"
        )
    if scanned["empty"]:
        lines.append(
            f"  [EMPTY_PAGE?] page {page_i:>3}  콘텐츠 없음 (가능성 신호 — 의도된 빈 쪽일 수 있음)"
        )
    return lines


def attach_expect(case):
    scanned = scan(case)
    first_ov = scanned["overflow"][0] if scanned["overflow"] else None
    case["expect"] = {
        "overflow": len(scanned["overflow"]),
        "offCanvas": len(scanned["offCanvas"]),
        "overlap": len(scanned["overlap"]),
        "textOverlap": len(scanned["textOverlap"]),
        "empty": scanned["empty"],
        "signal": scanned["signal"],
        "overflowTypes": [o["t"] for o in scanned["overflow"]],
        "offCanvasTypes": [o["t"] for o in scanned["offCanvas"]],
        "overflowPaths": [o["path"] for o in scanned["overflow"]],
        "offCanvasPaths": [o["path"] for o in scanned["offCanvas"]],
        "overlapPairs": [
            [o["pathA"], o["typeA"], o["pathB"], o["typeB"], o["overlapW"], o["overlapH"]]
            for o in scanned["overlap"]
        ],
        "textOverlapPairs": [
            [o["pathA"], o["typeA"], o["pathB"], o["typeB"], o["overlapW"], o["overlapH"]]
            for o in scanned["textOverlap"]
        ],
        "overLeft": first_ov["overLeft"] if first_ov else 0.0,
        "overTop": first_ov["overTop"] if first_ov else 0.0,
        "overRight": first_ov["overRight"] if first_ov else 0.0,
        "overBottom": first_ov["overBottom"] if first_ov else 0.0,
        "maxOver": first_ov["maxOver"] if first_ov else 0.0,
        "reportLines": report_lines(case["pageIndex"], scanned),
        "status": "ANOMALY" if scanned["signal"] else "CLEAN",
    }
    return case


def case(
    cid,
    family,
    hypothesis,
    nodes,
    *,
    page=None,
    body=None,
    page_index=0,
    page_count=3,
    overflow_tol=1.0,
    overlap_tol=2.0,
    types=None,
):
    rec = {
        "id": cid,
        "family": family,
        "hypothesis": hypothesis,
        "page": page or box(0, 0, 200, 300),
        "body": body or box(10, 20, 180, 260),
        "pageIndex": page_index,
        "pageCount": page_count,
        "opts": {
            "overflowTol": float(overflow_tol),
            "overlapTol": float(overlap_tol),
            "types": types,
        },
        "nodes": nodes,
    }
    return attach_expect(rec)


def compact_node(n):
    out = {"t": n["t"], "x": n["x"], "y": n["y"], "w": n["w"], "h": n["h"]}
    if n.get("wrap") is not None:
        out["wrap"] = n["wrap"]
    if n.get("master"):
        out["master"] = True
    if n.get("visible", True) is False:
        out["visible"] = False
    if n.get("editorOnly"):
        out["editorOnly"] = True
    if n.get("text") is not None:
        out["text"] = n["text"]
    if n.get("charOverlap"):
        out["charOverlap"] = True
    if n.get("column") is not None:
        out["column"] = n["column"]
    kids = [compact_node(c) for c in n.get("children") or []]
    if kids:
        out["children"] = kids
    return out


def compact_case(c):
    rec = {
        "id": c["id"],
        "family": c["family"],
        "hypothesis": c["hypothesis"],
        "page": c["page"],
        "body": c["body"],
        "pageIndex": c["pageIndex"],
        "pageCount": c["pageCount"],
        "opts": c["opts"],
        "nodes": [compact_node(n) for n in c["nodes"]],
        "expect": c["expect"],
    }
    return rec


def dump_json(path: Path, payload):
    path.parent.mkdir(parents=True, exist_ok=True)
    text = json.dumps(payload, ensure_ascii=False, indent=2)
    path.write_text(text + "\n", encoding="utf-8", newline="\n")


def dump_tsv(path: Path, header, rows):
    path.parent.mkdir(parents=True, exist_ok=True)
    lines = ["\t".join(header)]
    for row in rows:
        lines.append("\t".join(str(c) for c in row))
    path.write_text("\n".join(lines) + "\n", encoding="utf-8", newline="\n")


def text_line(x, y, w, h, runs):
    return node("TextLine", x, y, w, h, children=list(runs))


def run_at(text, x, y, w, h, *, char_overlap=False):
    return node("TextRun", x, y, w, h, text=text, char_overlap=char_overlap)


def gen_overflow_simple():
    cases = []
    types = [
        "Table",
        "Image",
        "TextBox",
        "Rect",
        "TextLine",
    ]
    # 본문 (10,20)-(190,280), 페이지 (0,0)-(200,300)
    # 기준 상자 안 원소: x=20,y=40,w=80,h=40
    # 허용치 안(0.5)·경계(1.0)·확정(2.5)·큰 넘침(16)
    base = (20.0, 40.0, 80.0, 40.0)
    overs = [0.5, 1.0, 2.5, 16.0]
    tols = [1.0]
    for t in types:
        for direction, apply in {
            "left": lambda ox, oy, ow, oh, over: (ox - over, oy, ow + over, oh),
            "top": lambda ox, oy, ow, oh, over: (ox, oy - over, ow, oh + over),
            "right": lambda ox, oy, ow, oh, over: (ox, oy, ow + over, oh),
            "bottom": lambda ox, oy, ow, oh, over: (ox, oy, ow, oh + over),
        }.items():
            for over in overs:
                for tol in tols:
                    x, y, w, h = apply(*base, over)
                    children = []
                    if t == "TextLine":
                        children = [run_at("본문", x, y, min(w, 40.0), min(h, 12.0))]
                    cid = f"ovf-{t.lower()}-{direction}-over{over:g}-tol{tol:g}"
                    hyp = (
                        f"{KO_TYPE[t]}가 본문 {KO_DIR[direction]}을 {over:g}px 넘친다. "
                        f"overflow 허용치 {tol:g}px. 페이지 상자와 비교하면 "
                        f"{'off-canvas 후보' if direction in ('left', 'top') or over >= 10 else '쪽 안에 잔류'}."
                    )
                    cases.append(
                        case(
                            cid,
                            "overflow",
                            hyp,
                            [node(t, x, y, w, h, children=children)],
                            overflow_tol=tol,
                        )
                    )
    return cases


def gen_overflow_nested_suppress():
    cases = []
    for over in [2.5, 8.0, 40.0]:
        for t in ["Table", "TextBox", "Group"]:
            inner = text_line(
                12.0,
                22.0,
                180.0 + over,
                12.0,
                [run_at("내부줄", 12.0, 22.0, 40.0, 12.0)],
            )
            cell = node("Cell", 10.0, 20.0, 180.0 + over, 40.0, children=[inner])
            outer = node(t, 10.0, 20.0, 180.0 + over, 40.0, children=[cell])
            cases.append(
                case(
                    f"suppress-{t.lower()}-right-{over:g}",
                    "suppress",
                    f"{KO_TYPE[t]}가 본문 오른쪽을 {over:g}px 넘칠 때 자손 TextLine 은 "
                    "suppress 되어 중복 overflow 를 내지 않는다.",
                    [outer],
                )
            )
    # --types TextLine 은 표 안으로 내려가 줄 overflow 만 본다
    for over in [2.5, 8.0]:
        inner = text_line(
            10.0,
            20.0,
            180.0 + over,
            12.0,
            [run_at("필터줄", 10.0, 20.0, 40.0, 12.0)],
        )
        cell = node("Cell", 10.0, 20.0, 180.0 + over, 40.0, children=[inner])
        table = node("Table", 10.0, 20.0, 180.0 + over, 40.0, children=[cell])
        cases.append(
            case(
                f"types-textline-inside-table-{over:g}",
                "types",
                f"--types TextLine 은 표 컨테이너를 건너뛰고 내부 줄의 {over:g}px 오른쪽 "
                "넘침만 overflow 로 센다. off-canvas 는 타입 필터와 무관하게 표에서 먼저 닫힌다.",
                [table],
                types=["TextLine"],
            )
        )
        cases.append(
            case(
                f"types-image-hides-table-{over:g}",
                "types",
                f"--types Image 는 표 overflow {over:g}px 를 버린다. 확정 신호는 "
                "off-canvas 가 있을 때만 남는다.",
                [table],
                types=["Image"],
            )
        )
    return cases


def gen_overlap():
    cases = []
    # 두 표 겹침 폭·높이 스윕
    for dw, dh in [
        (0.0, 20.0),
        (1.0, 20.0),
        (2.0, 20.0),
        (2.25, 20.0),
        (5.0, 20.0),
        (20.0, 20.0),
        (20.0, 1.0),
        (20.0, 2.0),
        (20.0, 2.25),
        (20.0, 0.5),
    ]:
        a = node("Table", 20.0, 40.0, 40.0, 40.0)
        b = node("Table", 20.0 + 40.0 - dw, 40.0 + 40.0 - dh, 40.0, 40.0)
        cases.append(
            case(
                f"ovl-table-table-w{dw:g}-h{dh:g}",
                "overlap",
                f"두 표의 교차가 {dw:g}x{dh:g}px. 기본 overlap 허용치 2px 는 "
                "폭·높이 둘 다 넘어야 확정이다.",
                [a, b],
            )
        )
    # wrap 모드
    wraps = ["Square", "Tight", "TopAndBottom", "BehindText", None]
    for wa in wraps:
        for wb in wraps:
            a = node("Rect", 20.0, 40.0, 40.0, 40.0, wrap=wa)
            b = node("Rect", 30.0, 50.0, 40.0, 40.0, wrap=wb)
            wa_s = wa or "none"
            wb_s = wb or "none"
            cases.append(
                case(
                    f"ovl-wrap-{wa_s.lower()}-{wb_s.lower()}",
                    "overlap",
                    f"도형 wrap={wa_s} 와 wrap={wb_s} 가 30x30 겹친다. "
                    "Square/Tight/TopAndBottom 만 overlap 후보다. "
                    "BehindText/InFrontOfText/Through/없음 은 겹치라고 있는 배치다.",
                    [a, b],
                )
            )
    # 바탕쪽
    cases.append(
        case(
            "ovl-master-square-ignored",
            "overlap",
            "바탕쪽 유래 Square 도형은 본문 Square 와 겹쳐도 overlap 후보가 아니다.",
            [
                node("Rect", 20.0, 40.0, 40.0, 40.0, wrap="Square", master=True),
                node("Rect", 30.0, 50.0, 40.0, 40.0, wrap="Square"),
            ],
        )
    )
    # 다른 단
    for same in (True, False):
        col_a = 0
        col_b = 0 if same else 1
        a = node(
            "Column",
            10.0,
            20.0,
            80.0,
            200.0,
            column=col_a,
            children=[node("Table", 12.0, 30.0, 50.0, 50.0)],
        )
        b = node(
            "Column",
            90.0 if not same else 20.0,
            20.0,
            80.0,
            200.0,
            column=col_b,
            children=[node("Table", 20.0, 40.0, 50.0, 50.0)],
        )
        cases.append(
            case(
                f"ovl-column-same-{str(same).lower()}",
                "overlap",
                "같은 단 안의 표만 짝짓는다. 다른 단은 x 가 나뉘어 있어도 후보에서 뺀다."
                if not same
                else "같은 단(column=0) 두 표가 겹치면 overlap 1건.",
                [a, b],
            )
        )
    # 운반용 빈 줄
    for text in ["", " ", "\t", "본문", " 가"]:
        slug = "empty" if not visible_text(text) else "visible"
        uniq = text.encode("unicode_escape").decode("ascii") or "empty"
        cases.append(
            case(
                f"ovl-carrier-{slug}-{uniq}",
                "overlap",
                "표와 같은 자리에 찍힌 운반용 TextLine 은 보이는 글자가 없으면 overlap 이 아니다. "
                f"text={text!r}.",
                [
                    node("Table", 20.0, 40.0, 80.0, 40.0),
                    text_line(20.0, 40.0, 80.0, 40.0, [run_at(text, 20.0, 40.0, 80.0, 40.0)]),
                ],
            )
        )
    # 두 줄 겹침
    for dy in [0.0, 2.0, 4.0, 10.0, 12.0]:
        a = text_line(20.0, 40.0, 80.0, 12.0, [run_at("갑", 20.0, 40.0, 80.0, 12.0)])
        b = text_line(20.0, 40.0 + 12.0 - dy, 80.0, 12.0, [run_at("을", 20.0, 40.0 + 12.0 - dy, 80.0, 12.0)])
        cases.append(
            case(
                f"ovl-textline-dy{dy:g}",
                "overlap",
                f"보이는 글자가 있는 두 문단 줄이 세로로 {dy:g}px 겹친다. "
                "줄 overlap 과 런 text-overlap 이 같이 열린다.",
                [a, b],
            )
        )
    return cases


def gen_text_overlap():
    cases = []
    # 같은 줄 런 교차
    for dx in [0.0, 2.0, 4.0, 10.0, 20.0, 30.0]:
        a = run_at("왼", 20.0, 40.0, 30.0, 12.0)
        b = run_at("오", 20.0 + 30.0 - dx, 40.0, 30.0, 12.0)
        cases.append(
            case(
                f"txov-same-line-dx{dx:g}",
                "text-overlap",
                f"한 줄 안 두 런이 {dx:g}px 겹친다. 맞닿음(0)은 교차가 아니고, "
                "허용치 2px 를 넘는 폭·높이만 text-overlap 이다.",
                [text_line(20.0, 40.0, 80.0, 12.0, [a, b])],
            )
        )
    # 표 안 두 줄
    for dy in [0.0, 2.0, 4.0, 8.0]:
        la = text_line(20.0, 40.0, 40.0, 12.0, [run_at("갑", 20.0, 40.0, 40.0, 12.0)])
        lb = text_line(30.0, 42.0 + dy, 40.0, 12.0, [run_at("을", 30.0, 42.0 + dy, 40.0, 12.0)])
        cell = node("Cell", 20.0, 40.0, 80.0, 40.0, children=[la, lb])
        table = node("Table", 20.0, 40.0, 80.0, 40.0, children=[cell])
        cases.append(
            case(
                f"txov-in-table-dy{dy:g}",
                "text-overlap",
                f"표 하나 안의 두 런이 세로 어긋남 {dy:g}px. 표-표 overlap 은 없고 "
                "글자끼리만 text-overlap 으로 남는다.",
                [table],
            )
        )
    # 글자겹침 컨트롤
    for flag in (True, False):
        cases.append(
            case(
                f"txov-char-overlap-{str(flag).lower()}",
                "text-overlap",
                "한컴 글자겹침 컨트롤 런은 의도된 겹침이라 후보에서 뺀다."
                if flag
                else "글자겹침 플래그가 없으면 같은 자리 두 런은 text-overlap 이다.",
                [
                    text_line(
                        20.0,
                        40.0,
                        40.0,
                        12.0,
                        [
                            run_at("가", 20.0, 40.0, 20.0, 12.0, char_overlap=flag),
                            run_at("나", 22.0, 40.0, 20.0, 12.0),
                        ],
                    )
                ],
            )
        )
    # 영면적
    for w, h, label in [(0.0, 12.0, "zero-w"), (20.0, 0.0, "zero-h"), (20.0, 12.0, "area")]:
        cases.append(
            case(
                f"txov-{label}",
                "text-overlap",
                "면적이 없는 런 bbox 는 text-overlap 후보가 아니다.",
                [
                    text_line(
                        20.0,
                        40.0,
                        40.0,
                        12.0,
                        [
                            run_at("가", 20.0, 40.0, w, h),
                            run_at("나", 22.0, 40.0, 20.0, 12.0),
                        ],
                    )
                ],
            )
        )
    # 공백만
    for text in ["", " ", "\n", "\t\t", "가"]:
        slug = "ws" if not visible_text(text) else "glyph"
        uniq = text.encode("unicode_escape").decode("ascii") or "empty"
        cases.append(
            case(
                f"txov-text-{slug}-{uniq}",
                "text-overlap",
                f"런 텍스트 {text!r} 의 가시 글자 여부가 후보를 가른다.",
                [
                    text_line(
                        20.0,
                        40.0,
                        40.0,
                        12.0,
                        [
                            run_at(text, 20.0, 40.0, 20.0, 12.0),
                            run_at("나", 22.0, 40.0, 20.0, 12.0),
                        ],
                    )
                ],
            )
        )
    return cases


def gen_off_canvas():
    cases = []
    types = ["Table", "Image", "TextBox", "Rect"]
    for t in types:
        for y in [-0.5, -1.0, -8.0, -80.0]:
            cases.append(
                case(
                    f"offy-{t.lower()}-y{y:g}",
                    "off-canvas",
                    f"{KO_TYPE[t]} y={y:g}. y<0 는 페이지 상자 y 와 무관하게 쪽 위 소실로 본다. "
                    "허용치 1px 이하면 침묵.",
                    [node(t, 20.0, y, 80.0, 40.0)],
                )
            )
        for extra in [0.5, 1.25, 50.0]:
            cases.append(
                case(
                    f"offw-{t.lower()}-right{extra:g}",
                    "off-canvas",
                    f"{KO_TYPE[t]} 가 페이지 폭을 {extra:g}px 넘긴다. 본문도 같이 넘치면 "
                    "overflow 와 off-canvas 가 동시에 열린다.",
                    [node(t, 10.0, 40.0, 190.0 + extra, 40.0)],
                )
            )
            cases.append(
                case(
                    f"offh-{t.lower()}-bottom{extra:g}",
                    "off-canvas",
                    f"{KO_TYPE[t]} 가 페이지 높이를 {extra:g}px 넘긴다.",
                    [node(t, 20.0, 200.0, 80.0, 100.0 + extra)],
                )
            )
    # 본문만 넘치고 쪽 안
    cases.append(
        case(
            "off-body-only-not-page",
            "off-canvas",
            "표가 본문 오른쪽만 넘치고 페이지 상자 안에 남으면 overflow 만 있고 off-canvas 는 없다.",
            [node("Table", 10.0, 40.0, 185.0, 40.0)],
        )
    )
    # 중첩 표 이중 보고 금지
    inner = text_line(-80.0, -80.0, 30.0, 10.0, [run_at("x", -80.0, -80.0, 30.0, 10.0)])
    cell = node("Cell", -80.0, -80.0, 30.0, 10.0, children=[inner])
    table = node("Table", -80.0, -80.0, 80.0, 120.0, children=[cell])
    cases.append(
        case(
            "off-nested-table-once",
            "off-canvas",
            "음수 y 표 안의 줄은 off-canvas suppress 로 표 한 번만 보고한다.",
            [table],
        )
    )
    return cases


def gen_empty_page():
    cases = []
    for page_n in range(1, 6):
        for page_i in range(page_n):
            for content in (False, True):
                nodes = []
                if content:
                    nodes = [
                        text_line(
                            20.0,
                            40.0,
                            40.0,
                            12.0,
                            [run_at("본문", 20.0, 40.0, 40.0, 12.0)],
                        )
                    ]
                mid = page_n >= 3 and 0 < page_i < page_n - 1
                hyp = (
                    f"page={page_i}/{page_n}, content={content}. "
                    "empty_page 는 쪽 수≥3 이고 첫·마지막이 아니며 콘텐츠가 없을 때만 가능성 신호. "
                    "has_signal 에는 들어가지 않는다."
                    if not content
                    else f"page={page_i}/{page_n} 에 보이는 글자가 있으면 empty_page 가 아니다."
                )
                if not mid and not content:
                    hyp += " 표지·뒷면 빈 쪽은 침묵."
                cases.append(
                    case(
                        f"empty-p{page_i}-of{page_n}-{'text' if content else 'blank'}",
                        "empty-page",
                        hyp,
                        nodes,
                        page_index=page_i,
                        page_count=page_n,
                    )
                )
    # 이미지만 있는 중간 쪽은 빈 쪽이 아님
    cases.append(
        case(
            "empty-middle-image-is-content",
            "empty-page",
            "중간 쪽에 그림만 있어도 has_content 이라 empty_page 가 아니다.",
            [node("Image", 20.0, 40.0, 40.0, 40.0)],
            page_index=1,
            page_count=4,
        )
    )
    return cases


def gen_visibility():
    cases = []
    for visible in (True, False):
        for editor in (False, True):
            cases.append(
                case(
                    f"vis-table-vis{str(visible).lower()}-ed{str(editor).lower()}",
                    "visibility",
                    "visible=false 이거나 editor_only=true 이면 그 서브트리는 걷지 않는다.",
                    [
                        node(
                            "Table",
                            0.0,
                            0.0,
                            250.0,
                            80.0,
                            visible=visible,
                            editor_only=editor,
                        )
                    ],
                )
            )
    return cases


def gen_combined():
    cases = []
    # overflow + text-overlap
    la = text_line(10.0, 30.0, 200.0, 12.0, [run_at("왼", 10.0, 30.0, 40.0, 12.0)])
    lb = text_line(20.0, 32.0, 40.0, 12.0, [run_at("오", 20.0, 32.0, 40.0, 12.0)])
    table = node("Table", 10.0, 30.0, 200.0, 40.0, children=[
        node("Cell", 10.0, 30.0, 200.0, 40.0, children=[la, lb])
    ])
    cases.append(
        case(
            "combo-table-overflow-and-text-overlap",
            "combined",
            "표가 본문·페이지를 넘치고 내부 런이 겹치면 overflow+off-canvas+text-overlap 이 같이 열린다. "
            "표 하나라 일반 overlap 은 없다.",
            [table],
        )
    )
    # 두 표 + 음수 y
    cases.append(
        case(
            "combo-two-tables-negative-y",
            "combined",
            "음수 y 로 겹친 두 표는 off-canvas 2 + overlap 1.",
            [
                node("Table", 10.0, -40.0, 80.0, 80.0),
                node("Table", 30.0, -20.0, 80.0, 80.0),
            ],
        )
    )
    # Square 도형 + 본문 줄
    cases.append(
        case(
            "combo-square-over-textline",
            "combined",
            "Square wrap 도형이 보이는 문단 줄과 겹치면 일반 overlap. 런-도형은 text-overlap 이 아니다.",
            [
                text_line(20.0, 40.0, 80.0, 12.0, [run_at("본문", 20.0, 40.0, 80.0, 12.0)]),
                node("Rect", 20.0, 40.0, 40.0, 40.0, wrap="Square"),
            ],
        )
    )
    return cases


def gen_tolerance_grid():
    cases = []
    for ov_tol in [0.5, 1.0, 2.0]:
        for over in [0.5, 1.0, 2.0, 8.0]:
            cases.append(
                case(
                    f"tol-ovf-over{over:g}-tol{ov_tol:g}",
                    "tolerance",
                    f"오른쪽 넘침 {over:g}px, overflow 허용치 {ov_tol:g}px. "
                    "판정은 초과량이 허용치보다 클 때만.",
                    [node("Table", 10.0, 40.0, 180.0 + over, 40.0)],
                    overflow_tol=ov_tol,
                )
            )
    for ovl_tol in [1.0, 2.0, 4.0]:
        for dw in [1.0, 2.0, 3.0, 8.0]:
            cases.append(
                case(
                    f"tol-ovl-w{dw:g}-tol{ovl_tol:g}",
                    "tolerance",
                    f"두 표 교차 {dw:g}x20px, overlap 허용치 {ovl_tol:g}px.",
                    [
                        node("Table", 20.0, 40.0, 40.0, 40.0),
                        node("Table", 20.0 + 40.0 - dw, 50.0, 40.0, 40.0),
                    ],
                    overlap_tol=ovl_tol,
                )
            )
    return cases


FAMILIES = {
    "overflow_simple": gen_overflow_simple,
    "overflow_nested": gen_overflow_nested_suppress,
    "overlap": gen_overlap,
    "text_overlap": gen_text_overlap,
    "off_canvas": gen_off_canvas,
    "empty_page": gen_empty_page,
    "visibility": gen_visibility,
    "combined": gen_combined,
    "tolerance": gen_tolerance_grid,
}


def matrix_rows(cases):
    rows = []
    for c in cases:
        e = c["expect"]
        rows.append(
            [
                c["id"],
                c["family"],
                c["opts"]["overflowTol"],
                c["opts"]["overlapTol"],
                ",".join(c["opts"]["types"] or []),
                c["pageIndex"],
                c["pageCount"],
                e["overflow"],
                e["offCanvas"],
                e["overlap"],
                e["textOverlap"],
                int(e["empty"]),
                int(e["signal"]),
                e["status"],
                "|".join(e["overflowTypes"]),
                "|".join(e["offCanvasTypes"]),
                f"{e['maxOver']:.4f}",
                c["hypothesis"],
            ]
        )
    return rows


MATRIX_HEADER = [
    "id",
    "family",
    "overflowTol",
    "overlapTol",
    "types",
    "pageIndex",
    "pageCount",
    "overflow",
    "offCanvas",
    "overlap",
    "textOverlap",
    "empty",
    "signal",
    "status",
    "overflowTypes",
    "offCanvasTypes",
    "maxOver",
    "hypothesis",
]


def write_transcripts(all_cases):
    tdir = OUT / "transcripts"
    tdir.mkdir(parents=True, exist_ok=True)
    picks = []
    wanted = [
        "ovf-table-right-over2.5-tol1",
        "ovf-table-right-over0.5-tol1",
        "offy-table-y-80",
        "off-body-only-not-page",
        "ovl-table-table-w20-h20",
        "txov-same-line-dx10",
        "txov-in-table-dy0",
        "txov-char-overlap-true",
        "empty-p1-of3-blank",
        "empty-p0-of3-blank",
        "combo-table-overflow-and-text-overlap",
        "suppress-table-right-8",
        "types-textline-inside-table-2.5",
        "types-image-hides-table-2.5",
        "ovl-wrap-behindtext-square",
        "ovl-carrier-empty-empty",
        "vis-table-visfalse-edfalse",
        "tol-ovf-over1-tol1",
        "ovl-column-same-false",
        "combo-two-tables-negative-y",
    ]
    by_id = {c["id"]: c for c in all_cases}
    for wid in wanted:
        if wid in by_id:
            picks.append(by_id[wid])
    # 대표 케이스마다 사람 성적표 + JSON 봉투
    for c in picks:
        e = c["expect"]
        human = [
            f"# transcript {c['id']}",
            f"# family={c['family']}",
            f"# hypothesis={c['hypothesis']}",
            f"쪽 수: {c['pageCount']}  overflow: {e['overflow']}  off-canvas: {e['offCanvas']}  "
            f"overlap: {e['overlap']}  text-overlap: {e['textOverlap']}  empty_page(가능성): {int(e['empty'])}",
        ]
        if not e["reportLines"]:
            human.append(f"이상 신호 없음: {c['id']}.synth")
        human.extend(e["reportLines"])
        human.append(f"status: {e['status']}")
        (tdir / f"{c['id']}.human.txt").write_text(
            "\n".join(human) + "\n", encoding="utf-8", newline="\n"
        )
        envelope = {
            "schemaVersion": "1.0",
            "mode": "single",
            "source": f"{c['id']}.synth",
            "pageCount": c["pageCount"],
            "pageFilter": None,
            "overflowTolerancePx": c["opts"]["overflowTol"],
            "overlapTolerancePx": c["opts"]["overlapTol"],
            "types": c["opts"]["types"],
            "strict": False,
            "overflowCount": e["overflow"],
            "offCanvasCount": e["offCanvas"],
            "overlapCount": e["overlap"],
            "textOverlapCount": e["textOverlap"],
            "emptyPageCount": int(e["empty"]),
            "hasSignal": e["signal"],
            "pages": [
                {
                    "page": c["pageIndex"],
                    "overflow": [
                        {
                            "path": p,
                            "nodeType": t,
                        }
                        for p, t in zip(e["overflowPaths"], e["overflowTypes"])
                    ],
                    "offCanvas": [
                        {
                            "path": p,
                            "nodeType": t,
                        }
                        for p, t in zip(e["offCanvasPaths"], e["offCanvasTypes"])
                    ],
                    "overlap": [
                        {
                            "pathA": pair[0],
                            "typeA": pair[1],
                            "pathB": pair[2],
                            "typeB": pair[3],
                            "overlapW": pair[4],
                            "overlapH": pair[5],
                        }
                        for pair in e["overlapPairs"]
                    ],
                    "textOverlap": [
                        {
                            "pathA": pair[0],
                            "typeA": pair[1],
                            "pathB": pair[2],
                            "typeB": pair[3],
                            "overlapW": pair[4],
                            "overlapH": pair[5],
                        }
                        for pair in e["textOverlapPairs"]
                    ],
                    "emptyPage": e["empty"],
                }
            ]
            if (e["overflow"] or e["offCanvas"] or e["overlap"] or e["textOverlap"] or e["empty"])
            else [],
            "hypothesis": c["hypothesis"],
            "fixtureId": c["id"],
        }
        dump_json(tdir / f"{c['id']}.envelope.json", envelope)

    # 배치 성적표 — 픽 순서를 정렬 키로 고정
    batch_rows = sorted(picks, key=lambda c: c["id"])
    ndjson_lines = []
    human_batch = ["=== layout-anomaly 요약 ==="]
    clean = anomaly = 0
    for c in batch_rows:
        e = c["expect"]
        if e["signal"]:
            anomaly += 1
            status = "ANOMALY"
        else:
            clean += 1
            status = "CLEAN"
        rec = {
            "schemaVersion": "1.0",
            "mode": "batch",
            "source": f"{c['id']}.synth",
            "overflowCount": e["overflow"],
            "offCanvasCount": e["offCanvas"],
            "overlapCount": e["overlap"],
            "textOverlapCount": e["textOverlap"],
            "emptyPageCount": int(e["empty"]),
            "hasSignal": e["signal"],
            "status": status,
            "types": c["opts"]["types"],
        }
        ndjson_lines.append(json.dumps(rec, ensure_ascii=False))
        human_batch.append(
            f"[{status:>15}] overflow={e['overflow']:<4} overlap={e['overlap']:<4} "
            f"empty={int(e['empty']):<3} {c['id']}.synth"
        )
    human_batch = [
        "",
        "=== layout-anomaly 요약 ===",
        f"  총 파일         : {len(batch_rows)}",
        f"  CLEAN           : {clean}",
        f"  ANOMALY         : {anomaly}",
        f"  LOAD_FAIL       : 0",
        "",
    ] + human_batch[1:]
    (tdir / "batch_catalog.ndjson").write_text(
        "\n".join(ndjson_lines) + "\n", encoding="utf-8", newline="\n"
    )
    (tdir / "batch_catalog.human.txt").write_text(
        "\n".join(human_batch) + "\n", encoding="utf-8", newline="\n"
    )

    # exit 행렬 성적표
    exit_rows = [
        ("single-clean-default", 0, "기본은 신호가 없어도 0"),
        ("single-signal-default", 0, "기본은 overflow 가 있어도 0"),
        ("single-signal-strict", 3, "--strict + 확정 신호는 3"),
        ("single-empty-strict", 0, "empty_page 만으로는 --strict 가 3 을 내지 않는다"),
        ("single-unknown-flag", 2, "알 수 없는 옵션은 2, stdout 0바이트"),
        ("single-bad-types", 2, "--types NotAType 은 2"),
        ("batch-missing-folder", 2, "--batch 폴더 없음은 2"),
        ("batch-load-fail", 1, "로드 실패는 1 이 --strict 의 3보다 우선"),
        ("batch-signal-strict", 3, "배치 --strict + 확정 신호는 3"),
        ("batch-clean-strict", 0, "깨끗한 배치는 --strict 여도 0"),
        ("off-canvas-strict", 3, "off-canvas 단독도 --strict 확정"),
        ("text-overlap-strict", 3, "text-overlap 단독도 --strict 확정"),
    ]
    lines = ["# layout-anomaly exit 계약 성적표", "scenario\texit\tnote"]
    lines += [f"{a}\t{b}\t{c}" for a, b, c in exit_rows]
    (tdir / "exit_contract.tsv").write_text(
        "\n".join(lines) + "\n", encoding="utf-8", newline="\n"
    )


def write_readme(counts):
    lines = [
        "# layout-anomaly M02-f 판정 픽스처",
        "",
        "이 폴더는 `rhwp layout-anomaly` 의 판정·성적표를 합성 렌더 트리로 고정한다.",
        "레이아웃 엔진·canvaskit_policy·serializer·pdf·equation 은 바꾸지 않는다.",
        "",
        "## 가족",
        "",
    ]
    for name, n in counts.items():
        lines.append(f"- `{name}`: {n}건")
    lines += [
        "",
        "## 읽는 법",
        "",
        "- `trees/*.json` — 페이지/본문 상자 + 노드 트리 + `expect` 판정.",
        "- `matrices/*.tsv` — 같은 건의 한 줄 행렬. 배치 리포트·회귀 표용.",
        "- `transcripts/*` — 사람 성적표·JSON 봉투·배치 NDJSON 표본.",
        "",
        "테스트는 `tests/cases/layout_anomaly_m02f_fatten.rs` 가 트리를 재조립해",
        "`scan_page` 실측과 `expect` 를 대조한다.",
        "",
    ]
    (OUT / "README.md").write_text("\n".join(lines), encoding="utf-8", newline="\n")


def main():
    if OUT.exists():
        for p in OUT.rglob("*"):
            if p.is_file():
                p.unlink()
    (OUT / "trees").mkdir(parents=True, exist_ok=True)
    (OUT / "matrices").mkdir(parents=True, exist_ok=True)
    (OUT / "transcripts").mkdir(parents=True, exist_ok=True)

    all_cases = []
    counts = {}
    for name, fn in FAMILIES.items():
        cases = fn()
        ids = [c["id"] for c in cases]
        if len(ids) != len(set(ids)):
            raise SystemExit(f"duplicate id in {name}")
        dump_json(OUT / "trees" / f"{name}.json", [compact_case(c) for c in cases])
        dump_tsv(OUT / "matrices" / f"{name}.tsv", MATRIX_HEADER, matrix_rows(cases))
        counts[name] = len(cases)
        all_cases.extend(cases)

    dump_tsv(OUT / "matrices" / "all_verdicts.tsv", MATRIX_HEADER, matrix_rows(all_cases))
    dump_json(
        OUT / "catalog.json",
        {
            "issue": 5459,
            "title": "M02-f layout-anomaly 판정 픽스처",
            "caseCount": len(all_cases),
            "families": counts,
        },
    )
    write_transcripts(all_cases)
    write_readme(counts)
    print(f"wrote {len(all_cases)} cases under {OUT}")
    for name, n in counts.items():
        print(f"  {name}: {n}")


if __name__ == "__main__":
    main()
