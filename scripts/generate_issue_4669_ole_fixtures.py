#!/usr/bin/env python3
"""Generate #4669 OLE shape-component fixtures and envelope transcripts.

Each case is a distinct HWPX `<hp:ole>` save contract (id vs instid,
curSz 0 sentinel, offset wraparound, flip, rotation, renderingInfo,
lineShape). Re-run to regenerate the corpus; do not hand-edit outputs.
"""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "tests" / "fixtures" / "issue_4669_ole_shape_component"
XML_DIR = OUT / "xml"
ENV_DIR = OUT / "envelopes"

NS = """xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section"
        xmlns:hc="http://www.hancom.co.kr/hwpml/2011/core\""""

# 한셀OLE.hwpx 실측 정답값 (#4669 이슈 본문·원본 section0.xml).
HANSEL = {
    "id": 2141242094,
    "instid": 1067500271,
    "zOrder": 1,
    "numberingType": "PICTURE",
    "textWrap": "SQUARE",
    "textFlow": "BOTH_SIDES",
    "lock": "0",
    "objectType": "EMBEDDED",
    "binaryItemIDRef": "ole1",
    "drawAspect": "CONTENT",
    "offset": (0, 0),
    "orgSz": (42001, 13501),
    "curSz": (29999, 4051),
    "flip": (0, 0),
    "rot": (0, 14999, 2025, 1),
    "trans": (1, 0, 0, 0, 1, 0),
    "sca": (0.714245, 0, 0, 0, 0.300052, 0),
    "rotm": (1, 0, 0, 0, 1, 0),
    "extent": (29999, 4051),
    "line": {"color": "#000000", "width": 0, "style": "NONE", "endCap": "ROUND"},
    "sz": (29999, 4051),
}


def wrap_u32(v: int) -> int:
    return v & 0xFFFFFFFF


def fmt_num(v: float | int) -> str:
    if isinstance(v, int) or float(v).is_integer():
        return str(int(v))
    s = f"{float(v):.6f}".rstrip("0").rstrip(".")
    return s


def matrix_xml(name: str, m: tuple) -> str:
    e = [fmt_num(x) for x in m]
    return (
        f'          <hc:{name} e1="{e[0]}" e2="{e[1]}" e3="{e[2]}" '
        f'e4="{e[3]}" e5="{e[4]}" e6="{e[5]}"/>'
    )


def ole_xml(c: dict) -> str:
    ox, oy = c["offset"]
    ow, oh = c["orgSz"]
    cw, ch = c["curSz"]
    fh, fv = c["flip"]
    ang, cx, cy, ri = c["rot"]
    ex, ey = c["extent"]
    sw, sh = c["sz"]
    line = c["line"]
    id_attr = "" if c.get("omit_id") else f' id="{c["id"]}"'
    instid_attr = "" if c.get("omit_instid") else f' instid="{c["instid"]}"'
    return f"""      <hp:ole{id_attr} zOrder="{c["zOrder"]}" numberingType="{c["numberingType"]}" textWrap="{c["textWrap"]}"
              textFlow="{c["textFlow"]}" lock="{c["lock"]}"{instid_attr} objectType="{c["objectType"]}"
              binaryItemIDRef="{c["binaryItemIDRef"]}" drawAspect="{c["drawAspect"]}">
        <hp:offset x="{wrap_u32(ox)}" y="{wrap_u32(oy)}"/>
        <hp:orgSz width="{ow}" height="{oh}"/>
        <hp:curSz width="{cw}" height="{ch}"/>
        <hp:flip horizontal="{fh}" vertical="{fv}"/>
        <hp:rotationInfo angle="{ang}" centerX="{cx}" centerY="{cy}" rotateimage="{ri}"/>
        <hp:renderingInfo>
{matrix_xml("transMatrix", c["trans"])}
{matrix_xml("scaMatrix", c["sca"])}
{matrix_xml("rotMatrix", c["rotm"])}
        </hp:renderingInfo>
        <hc:extent x="{ex}" y="{ey}"/>
        <hp:lineShape color="{line["color"]}" width="{line["width"]}" style="{line["style"]}" endCap="{line["endCap"]}"/>
        <hp:sz width="{sw}" widthRelTo="ABSOLUTE" height="{sh}" heightRelTo="ABSOLUTE" protect="0"/>
        <hp:pos treatAsChar="0" vertRelTo="PARA" horzRelTo="COLUMN" vertOffset="0" horzOffset="0"/>
        <hp:outMargin left="0" right="0" top="0" bottom="0"/>
      </hp:ole>"""


def section_xml(fid: str, family: str, contract: str, source: str, oles: list[str]) -> str:
    joined = "\n".join(oles)
    return f"""<?xml version="1.0" encoding="UTF-8"?>
<!--
  issue: 4669
  fixture: {fid}
  family: {family}
  contract: {contract}
  source: {source}
  save-must: hp:ole id + instid 분리, offset/orgSz/curSz/flip/rotationInfo/renderingInfo/lineShape 원문
  save-must-not: id="0" 재부여(원문이 0 이 아닐 때), curSz=0 을 orgSz 로 재유도
-->
<hs:sec {NS}>
  <hp:p id="0" paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
{joined}
      <hp:t/>
    </hp:run>
  </hp:p>
</hs:sec>
"""


def expected_id(c: dict) -> int | None:
    if c.get("omit_id"):
        return None
    return int(c["id"])


def expected_instid(c: dict) -> int:
    if c.get("omit_instid"):
        return int(c.get("id", 0))
    return int(c["instid"])


def signed32(v: int) -> int:
    v = int(v)
    if v >= 2**31:
        return v - 2**32
    if v < -2**31:
        return v + 2**32
    return v


def envelope(fid: str, family: str, contract: str, source: str, cases: list[dict]) -> dict:
    items = []
    for i, c in enumerate(cases):
        ox, oy = c["offset"]
        ow, oh = c["orgSz"]
        cw, ch = c["curSz"]
        eid = expected_id(c)
        einst = expected_instid(c)
        was_zw = cw == 0 and ow > 0
        was_zh = ch == 0 and oh > 0
        save_id = einst if eid is None else eid
        expect_xml = [
            f'id="{save_id}"',
            f'instid="{einst}"',
            f'<hp:offset x="{wrap_u32(ox)}" y="{wrap_u32(oy)}"/>',
            f'<hp:orgSz width="{ow}" height="{oh}"/>',
            f'<hp:curSz width="{cw}" height="{ch}"/>',
            f'<hp:flip horizontal="{c["flip"][0]}" vertical="{c["flip"][1]}"/>',
            f'angle="{c["rot"][0]}"',
            f'centerX="{c["rot"][1]}"',
            f'centerY="{c["rot"][2]}"',
            f'rotateimage="{c["rot"][3]}"',
            f'style="{c["line"]["style"]}"',
            f'width="{c["line"]["width"]}"',
        ]
        forbid_xml = []
        if was_zw and was_zh:
            forbid_xml.append(f'<hp:curSz width="{ow}" height="{oh}"/>')
        if ox != 0 or oy != 0:
            forbid_xml.append('<hp:offset x="0" y="0"/>')
        items.append(
            {
                "index": i,
                "id": None if c.get("omit_id") else c["id"],
                "instid": None if c.get("omit_instid") else c["instid"],
                "hwpx_ole_id": eid,
                "instance_id": einst,
                "save_id": save_id,
                "offset": [signed32(ox), signed32(oy)],
                "offset_u32": [wrap_u32(ox), wrap_u32(oy)],
                "orgSz": [ow, oh],
                "curSz": [cw, ch],
                "was_zero": [was_zw, was_zh],
                "flip": [c["flip"][0], c["flip"][1]],
                "rotation": list(c["rot"]),
                "trans": list(c["trans"]),
                "sca": list(c["sca"]),
                "line": dict(c["line"]),
                "expect_xml": expect_xml,
                "forbid_xml": forbid_xml,
                "forbid_id_zero": bool(eid not in (None, 0) and save_id != 0),
            }
        )
    return {
        "schema": "rhwp.issue4669.ole-shape-component.v1",
        "issue": 4669,
        "tracker": 5450,
        "fixture_id": fid,
        "family": family,
        "contract": contract,
        "source": source,
        "oles": items,
    }


def clone(base: dict, **kw) -> dict:
    out = dict(base)
    for k, v in kw.items():
        if k == "line" and isinstance(v, dict):
            line = dict(out["line"])
            line.update(v)
            out["line"] = line
        else:
            out[k] = v
    return out


def build_cases() -> list[tuple[str, str, str, str, list[dict]]]:
    h = HANSEL
    cases: list[tuple[str, str, str, str, list[dict]]] = []

    def add(fid, family, contract, source, oles):
        cases.append((fid, family, contract, source, oles))

    # --- family: hansel oracle -------------------------------------------------
    add(
        "000_hansel_oracle",
        "oracle",
        "samples/한셀OLE.hwpx 원본 hp:ole 의 id≠instid 와 shape-component 원문을 그대로 고정한다.",
        "samples/한셀OLE.hwpx Contents/section0.xml",
        [dict(h)],
    )

    # --- family: id / instid ---------------------------------------------------
    id_pairs = [
        ("001_id_ne_instid_hansel", h["id"], h["instid"], False, False, "한셀 실측 id≠instid"),
        ("002_id_eq_instid", 77, 77, False, False, "id 와 instid 가 같은 경우도 둘 다 보존"),
        ("003_id_zero_instid_nonzero", 0, h["instid"], False, False, "명시적 id=0 은 instid 로 덮지 않는다"),
        ("004_id_nonzero_instid_zero", 1117817146, 0, False, False, "명시적 instid=0 은 id 로 덮지 않는다 (#4099)"),
        ("005_id_absent_instid_present", h["id"], h["instid"], True, False, "id 부재 시 instid 가 id 를 겸한다"),
        ("006_id_present_instid_absent", 4242, 0, False, True, "instid 부재 시 id 가 instance_id 를 겸한다"),
        ("007_both_zero", 0, 0, False, False, "id=0 instid=0 둘 다 유효한 원문"),
        ("008_both_max_u32", 4294967295, 4294967295, False, False, "u32 최대값 id/instid"),
        ("009_id_max_instid_one", 4294967295, 1, False, False, "id 최대·instid 최소 비대칭"),
        ("010_id_one_instid_max", 1, 4294967295, False, False, "id 최소 비영·instid 최대"),
        ("011_id_hansel_instid_one", h["id"], 1, False, False, "한셀 id + 작은 instid"),
        ("012_id_small_instid_hansel", 9, h["instid"], False, False, "작은 id + 한셀 instid"),
        ("013_id_pow2_instid_pow2", 1 << 20, 1 << 30, False, False, "2의 거듭제곱 분리"),
        ("014_id_odd_instid_even", 2141242095, 1067500270, False, False, "홀/짝 분리"),
        ("015_id_chart_fallback_style", 1117817146, 0, False, False, "차트 fallback OLE 의 instid=0 관례"),
        ("016_id_near_i32_max", 2147483647, 2147483648, False, False, "i32 경계 근처"),
    ]
    for fid, i, inst, omit_id, omit_instid, why in id_pairs:
        add(
            fid,
            "id-instid",
            why,
            "synthetic / 한셀OLE id·instid 관례",
            [clone(h, id=i, instid=inst, omit_id=omit_id, omit_instid=omit_instid)],
        )

    # --- family: curSz / orgSz -------------------------------------------------
    size_rows = [
        ("021_cursz_zero_zero", (0, 0), (42001, 13501), "한컴 원산 관례 curSz=0 → was_zero 센티널"),
        ("022_cursz_zero_width_only", (0, 4051), (42001, 13501), "width 만 0 센티널"),
        ("023_cursz_zero_height_only", (29999, 0), (42001, 13501), "height 만 0 센티널"),
        ("024_cursz_eq_orgsz", (42001, 13501), (42001, 13501), "curSz = orgSz 는 재계산 없이 보존"),
        ("025_cursz_eq_extent", (29999, 4051), (42001, 13501), "한셀 실측 curSz = extent ≠ orgSz"),
        ("026_cursz_independent", (18000, 2400), (42001, 13501), "curSz 가 orgSz·extent 와 모두 다름"),
        ("027_cursz_one_by_one", (1, 1), (42001, 13501), "최소 양수 curSz"),
        ("028_orgsz_one_by_one", (1, 1), (1, 1), "최소 orgSz+curSz"),
        ("029_orgsz_zero_cursz_zero", (0, 0), (0, 0), "orgSz=0 이면 was_zero materialize 없음"),
        ("030_large_orgsz", (0, 0), (200000, 150000), "큰 orgSz + curSz=0"),
        ("031_square_orgsz", (8000, 8000), (16000, 16000), "정사각 원본"),
        ("032_wide_banner", (48000, 1200), (96000, 2400), "가로로 긴 OLE"),
        ("033_tall_banner", (1200, 48000), (2400, 96000), "세로로 긴 OLE"),
        ("034_cursz_gt_orgsz", (50000, 20000), (10000, 4000), "표시 크기가 원본보다 큼"),
        ("035_cursz_half_orgsz", (21000, 6750), (42000, 13500), "절반 스케일"),
        ("036_cursz_third_orgsz", (14000, 4500), (42000, 13500), "1/3 스케일"),
        ("037_orgsz_prime", (29989, 4049), (42013, 13513), "소수 크기 — 재유도 오탐 방지"),
        ("038_cursz_zero_7200_org", (0, 0), (7200, 7200), "7200 org + 0 cur — 기본값과 구분"),
        ("039_asymmetric_zero", (0, 1), (9000, 3000), "width 0 / height 1"),
        ("040_asymmetric_zero_h", (1, 0), (9000, 3000), "width 1 / height 0"),
        ("041_unit_hwpx", (283, 283), (566, 566), "1mm 근처 HWPUNIT"),
    ]
    for fid, cur, org, why in size_rows:
        add(
            fid,
            "cursz-orgsz",
            why,
            "synthetic / #2017 OLE 센티널",
            [clone(h, curSz=cur, orgSz=org, extent=cur if cur[0] and cur[1] else org, sz=cur if cur[0] and cur[1] else org)],
        )

    # --- family: offset --------------------------------------------------------
    off_rows = [
        ("045_offset_zero", (0, 0), "원점 offset"),
        ("046_offset_pos", (12, 34), "양수 offset (단위 시험 값)"),
        ("047_offset_hansel_zero", (0, 0), "한셀 실측 offset 0,0"),
        ("048_offset_wrap_y", (5250, -4332), "y 음수 wraparound — pic #4668 과 동형, OLE 축"),
        ("049_offset_wrap_x", (-2429, 100), "x 음수 wraparound"),
        ("050_offset_wrap_both", (-100, -200), "x/y 모두 wraparound"),
        ("051_offset_i32_min_y", (0, -2147483648), "y = i32::MIN"),
        ("052_offset_large_pos", (100000, 80000), "큰 양수 offset"),
        ("053_offset_unit", (1, -1), "최소 비영 ±1"),
        ("054_offset_page_out", (59528, -10000), "쪽 너비 밖 + 위쪽 음수"),
        ("055_offset_x_only", (7777, 0), "x 만 이동"),
        ("056_offset_y_only", (0, 8888), "y 만 이동"),
        ("057_offset_u32_max_as_neg1", (-1, 0), "x=-1 → 4294967295"),
        ("058_offset_trans_match", (5250, -4332), "offset 과 transMatrix 병진이 일치해야 하는 형태"),
    ]
    for fid, off, why in off_rows:
        trans = (1, 0, off[0], 0, 1, off[1])
        add(fid, "offset", why, "synthetic / #3544 unsigned wraparound", [clone(h, offset=off, trans=trans)])

    # --- family: flip ----------------------------------------------------------
    for i, (fh, fv) in enumerate([(0, 0), (1, 0), (0, 1), (1, 1)]):
        add(
            f"06{1+i}_flip_{fh}_{fv}",
            "flip",
            f"flip horizontal={fh} vertical={fv} 원문 보존",
            "synthetic / shape-component flip",
            [clone(h, flip=(fh, fv))],
        )
    # flip + nonzero offset / cursz 0
    add("065_flip_h_cursz0", "flip", "수평 flip + curSz=0 동시 보존", "synthetic", [clone(h, flip=(1, 0), curSz=(0, 0))])
    add("066_flip_v_offset_wrap", "flip", "수직 flip + offset wraparound", "synthetic", [clone(h, flip=(0, 1), offset=(0, -50), trans=(1, 0, 0, 0, 1, -50))])
    add("067_flip_both_id_ne", "flip", "양축 flip + id≠instid", "synthetic", [clone(h, flip=(1, 1))])
    add("068_flip_none_cursz0", "flip", "flip 없음 + curSz=0 (기본 flip 재유도와 구분)", "synthetic", [clone(h, flip=(0, 0), curSz=(0, 0))])

    # --- family: rotation ------------------------------------------------------
    rot_rows = [
        ("069_rot_0_hansel", (0, 14999, 2025, 1), "한셀 실측 rotationInfo"),
        ("070_rot_90", (90, 14999, 2025, 1), "90도"),
        ("071_rot_180", (180, 14999, 2025, 1), "180도"),
        ("072_rot_270", (270, 14999, 2025, 1), "270도"),
        ("073_rot_45", (45, 8000, 4000, 1), "45도 + 다른 중심"),
        ("074_rot_image_off", (0, 14999, 2025, 0), "rotateimage=0"),
        ("075_rot_image_on_nonzero", (15, 0, 0, 1), "작은 각 + 원점 중심"),
        ("076_rot_neg_center", (0, -100, -200, 1), "음수 회전 중심"),
        ("077_rot_large_center", (0, 100000, 80000, 1), "큰 회전 중심"),
        ("078_rot_359", (359, 1, 1, 0), "359도 rotateimage=0"),
        ("079_rot_1_deg", (1, 14999, 2025, 1), "1도"),
        ("080_rot_zero_center_zero_angle", (0, 0, 0, 0), "전부 0 — 기본값 재유도와 구분"),
    ]
    for fid, rot, why in rot_rows:
        add(fid, "rotation", why, "synthetic / rotationInfo", [clone(h, rot=rot)])

    # --- family: renderingInfo -------------------------------------------------
    rend_rows = [
        ("081_render_hansel_scale", HANSEL["trans"], HANSEL["sca"], HANSEL["rotm"], "한셀 실측 sca 0.714245×0.300052"),
        ("082_render_identity", (1, 0, 0, 0, 1, 0), (1, 0, 0, 0, 1, 0), (1, 0, 0, 0, 1, 0), "항등 행렬 3개"),
        ("083_render_scale_half", (1, 0, 0, 0, 1, 0), (0.5, 0, 0, 0, 0.5, 0), (1, 0, 0, 0, 1, 0), "균등 0.5 스케일"),
        ("084_render_scale_2", (1, 0, 0, 0, 1, 0), (2, 0, 0, 0, 2, 0), (1, 0, 0, 0, 1, 0), "균등 2배"),
        ("085_render_trans_only", (1, 0, 5250, 0, 1, -4332), (1, 0, 0, 0, 1, 0), (1, 0, 0, 0, 1, 0), "병진만 — offset 과 일치 형태"),
        ("086_render_rot90", (1, 0, 0, 0, 1, 0), (1, 0, 0, 0, 1, 0), (0, -1, 0, 1, 0, 0), "90도 rotMatrix"),
        ("087_render_shear", (1, 0.2, 0, 0.1, 1, 0), (1, 0, 0, 0, 1, 0), (1, 0, 0, 0, 1, 0), "전단 성분"),
        ("088_render_tiny_scale", (1, 0, 0, 0, 1, 0), (0.01, 0, 0, 0, 0.02, 0), (1, 0, 0, 0, 1, 0), "매우 작은 스케일"),
        ("089_render_neg_scale", (1, 0, 0, 0, 1, 0), (-1, 0, 0, 0, 1, 0), (1, 0, 0, 0, 1, 0), "음수 sx (거울)"),
        ("090_render_neg_sy", (1, 0, 0, 0, 1, 0), (1, 0, 0, 0, -1, 0), (1, 0, 0, 0, 1, 0), "음수 sy"),
        ("091_render_precise", (1, 0, 0, 0, 1, 0), (0.714245, 0, 0, 0, 0.300052, 0), (1, 0, 0, 0, 1, 0), "한셀 소수 정밀도 재현"),
        ("092_render_other_frac", (1, 0, 0, 0, 1, 0), (1.579917, 0, 0, 0, 0.333333, 0), (1, 0, 0, 0, 1, 0), "다른 소수 — f32 왕복"),
        ("093_render_tx_ty", (1, 0, 12, 0, 1, 34), (0.714245, 0, 0, 0, 0.300052, 0), (1, 0, 0, 0, 1, 0), "병진+한셀 스케일"),
        ("094_render_almost_id", (1, 0, 0, 0, 1, 0), (0.999999, 0, 0, 0, 1.000001, 0), (1, 0, 0, 0, 1, 0), "항등에 가까운 스케일"),
        ("095_render_zero_scale", (1, 0, 0, 0, 1, 0), (0, 0, 0, 0, 0, 0), (1, 0, 0, 0, 1, 0), "영 스케일 행렬"),
        ("096_render_combined", (1, 0, 100, 0, 1, -50), (0.5, 0, 0, 0, 0.25, 0), (0, -1, 0, 1, 0, 0), "병진+스케일+회전 동시"),
    ]
    for fid, tr, sc, rm, why in rend_rows:
        add(fid, "rendering", why, "synthetic / renderingInfo raw_rendering", [clone(h, trans=tr, sca=sc, rotm=rm)])

    # --- family: lineShape -----------------------------------------------------
    line_rows = [
        ("097_line_none_hansel", "#000000", 0, "NONE", "ROUND", "한셀 실측 선 없음"),
        ("098_line_solid_w5", "#000000", 5, "SOLID", "ROUND", "SOLID width=5 (단위 시험)"),
        ("099_line_solid_w1", "#FF0000", 1, "SOLID", "FLAT", "빨간 실선 FLAT"),
        ("100_line_dash", "#0000FF", 10, "DASH", "SQUARE", "파란 파선 SQUARE"),
        ("101_line_dot", "#00FF00", 8, "DOT", "ROUND", "점선"),
        ("102_line_dash_dot", "#123456", 3, "DASH_DOT", "ROUND", "1점 쇄선"),
        ("103_line_dash_dot_dot", "#ABCDEF", 4, "DASH_DOT_DOT", "FLAT", "2점 쇄선"),
        ("104_line_long_dash", "#010101", 12, "LONG_DASH", "ROUND", "긴 파선"),
        ("105_line_circle", "#800080", 6, "CIRCLE", "ROUND", "원 테두리 스타일"),
        ("106_line_double_slim", "#008080", 2, "DOUBLE_SLIM", "ROUND", "이중 가는 선"),
        ("107_line_slim_thick", "#808000", 7, "SLIM_THICK", "FLAT", "가늘+굵"),
        ("108_line_thick_slim", "#000080", 9, "THICK_SLIM", "SQUARE", "굵+가늘"),
        ("109_line_slim_thick_slim", "#808080", 11, "SLIM_THICK_SLIM", "ROUND", "가늘+굵+가늘"),
        ("110_line_white_solid", "#FFFFFF", 20, "SOLID", "ROUND", "흰 실선 굵게"),
        ("111_line_none_nonzero_w", "#000000", 15, "NONE", "ROUND", "style=NONE 이지만 width>0 — 재유도 금지"),
        ("112_line_solid_zero_w", "#000000", 0, "SOLID", "ROUND", "SOLID + width=0"),
    ]
    for fid, color, width, style, cap, why in line_rows:
        add(fid, "lineshape", why, "synthetic / lineShape", [clone(h, line={"color": color, "width": width, "style": style, "endCap": cap})])

    # --- family: hansel mutants (한 필드만 변경) --------------------------------
    mutants = [
        ("113_mut_cursz0", dict(curSz=(0, 0)), "한셀 원문에 curSz=0 만 넣은 재현"),
        ("114_mut_offset_wrap", dict(offset=(5250, -4332), trans=(1, 0, 5250, 0, 1, -4332)), "한셀 + wraparound offset"),
        ("115_mut_flip_h", dict(flip=(1, 0)), "한셀 + 수평 flip"),
        ("116_mut_id_zero", dict(id=0), "한셀 instid 유지 + id=0"),
        ("117_mut_instid_zero", dict(instid=0), "한셀 id 유지 + instid=0"),
        ("118_mut_line_solid", dict(line={"color": "#000000", "width": 5, "style": "SOLID", "endCap": "ROUND"}), "한셀 + SOLID 테두리"),
        ("119_mut_rot90", dict(rot=(90, 14999, 2025, 1)), "한셀 + 90도"),
        ("120_mut_sca_id", dict(sca=(1, 0, 0, 0, 1, 0)), "한셀 sca 를 항등으로"),
        ("121_mut_lock", dict(lock="1"), "한셀 + lock=1"),
        ("122_mut_icon", dict(drawAspect="ICON"), "한셀 + drawAspect=ICON"),
        ("123_mut_thumb", dict(drawAspect="THUMBNAIL"), "한셀 + THUMBNAIL"),
        ("124_mut_docprint", dict(drawAspect="DOCPRINT"), "한셀 + DOCPRINT"),
        ("125_mut_wrap_tight", dict(textWrap="TIGHT"), "한셀 + TIGHT"),
        ("126_mut_wrap_topbottom", dict(textWrap="TOP_AND_BOTTOM"), "한셀 + TOP_AND_BOTTOM"),
        ("127_mut_front", dict(textWrap="IN_FRONT_OF_TEXT"), "한셀 + IN_FRONT_OF_TEXT"),
        ("128_mut_behind", dict(textWrap="BEHIND_TEXT"), "한셀 + BEHIND_TEXT"),
        ("129_mut_all_zero_comp", dict(offset=(0, 0), curSz=(0, 0), flip=(0, 0), rot=(0, 0, 0, 0)), "한셀 id 유지 + 자식 0"),
    ]
    for fid, kw, why in mutants:
        add(fid, "hansel-mutant", why, "samples/한셀OLE.hwpx 1필드 변이", [clone(h, **kw)])

    # --- family: attributes around ole (still shape-component save) ------------
    attr_rows = [
        ("133_aspect_icon_cursz0", dict(drawAspect="ICON", curSz=(0, 0)), "ICON + curSz=0"),
        ("134_lock_and_wrap_offset", dict(lock="1", offset=(-10, 20), trans=(1, 0, -10, 0, 1, 20)), "lock + wraparound x"),
        ("135_through_wrap", dict(textWrap="THROUGH"), "THROUGH 배치"),
        ("136_largest_flow", dict(textFlow="LARGEST_ONLY"), "LARGEST_ONLY"),
        ("137_right_flow", dict(textFlow="RIGHT_ONLY"), "RIGHT_ONLY"),
        ("138_picture_numbering_cursz0", dict(numberingType="PICTURE", curSz=(0, 0)), "PICTURE numbering + curSz=0"),
    ]
    for fid, kw, why in attr_rows:
        add(fid, "ole-attr", why, "synthetic / hp:ole 속성 + 자식 동시", [clone(h, **kw)])

    # --- family: multi-ole -----------------------------------------------------
    a = clone(h, id=1001, instid=2001, offset=(10, 20), trans=(1, 0, 10, 0, 1, 20))
    b = clone(h, id=1002, instid=2002, curSz=(0, 0), flip=(1, 0))
    c = clone(h, id=0, instid=3003, line={"color": "#FF0000", "width": 3, "style": "SOLID", "endCap": "FLAT"})
    d = clone(h, id=4004, instid=0, offset=(-5, -6), trans=(1, 0, -5, 0, 1, -6))
    add("143_multi_two_oles", "multi-ole", "한 문단에 OLE 2개 — 각각의 id/자식을 독립 보존", "synthetic", [a, b])
    add("144_multi_three_oles", "multi-ole", "OLE 3개 (양수 offset / curSz0+flip / id=0)", "synthetic", [a, b, c])
    add("145_multi_four_oles", "multi-ole", "OLE 4개 (instid=0 wraparound 포함)", "synthetic", [a, b, c, d])
    e = clone(h, id=5005, instid=5005, rot=(90, 1, 2, 0))
    add("146_multi_id_eq_and_ne", "multi-ole", "id=instid 인 OLE 와 id≠instid 인 OLE 혼재", "synthetic", [a, e])
    f = clone(h, id=6006, instid=7007, sca=(0.5, 0, 0, 0, 0.25, 0))
    add("147_multi_scale_pair", "multi-ole", "서로 다른 scaMatrix 를 가진 OLE 쌍", "synthetic", [clone(h), f])
    add("148_multi_cursz0_pair", "multi-ole", "둘 다 curSz=0 이지만 orgSz 가 다름", "synthetic", [clone(h, curSz=(0, 0), orgSz=(1000, 2000), id=11, instid=21), clone(h, curSz=(0, 0), orgSz=(3000, 4000), id=12, instid=22)])

    # --- family: combination stress -------------------------------------------
    add(
        "149_combo_all_nonzero",
        "combo",
        "id≠instid + wraparound offset + curSz 독립 + flip + 90도 + 한셀 sca + SOLID",
        "synthetic combination",
        [clone(h, offset=(-2429, -4332), trans=(1, 0, -2429, 0, 1, -4332), curSz=(18000, 2400), flip=(1, 1), rot=(90, 9000, 1200, 1), line={"color": "#112233", "width": 4, "style": "SOLID", "endCap": "FLAT"})],
    )
    add(
        "150_combo_all_zeroish",
        "combo",
        "명시적 0 값 묶음 — 기본값 재유도와 구분돼야 한다",
        "synthetic combination",
        [clone(h, id=0, instid=0, offset=(0, 0), curSz=(0, 0), orgSz=(7200, 7200), flip=(0, 0), rot=(0, 0, 0, 0), sca=(1, 0, 0, 0, 1, 0), line={"color": "#000000", "width": 0, "style": "NONE", "endCap": "ROUND"})],
    )
    add(
        "151_combo_cursz0_wrap_idne",
        "combo",
        "이슈 본문 재현: id≠instid + curSz=0 + 원문 offset",
        "issue #4669 reproduction",
        [clone(h, curSz=(0, 0), offset=(12, 34), trans=(1, 0, 12, 0, 1, 34))],
    )
    add(
        "152_combo_issue_body_xml",
        "combo",
        "이슈 본문 축약 XML 과 동일 값 (id 2141242094 / instid 1067500271 / curSz 0 / offset 12,34)",
        "issue #4669 body",
        [clone(h, offset=(12, 34), trans=(1, 0, 12, 0, 1, 34), orgSz=(42001, 13501), curSz=(0, 0), flip=(1, 0), rot=(0, 14999, 2025, 1))],
    )

    return cases


def main() -> None:
    if OUT.exists():
        for p in XML_DIR.glob("*.xml"):
            p.unlink()
        for p in ENV_DIR.glob("*.json"):
            p.unlink()
    XML_DIR.mkdir(parents=True, exist_ok=True)
    ENV_DIR.mkdir(parents=True, exist_ok=True)

    catalog = []
    cases = build_cases()
    seen = set()
    for fid, family, contract, source, oles in cases:
        if fid in seen:
            raise SystemExit(f"duplicate fixture id: {fid}")
        seen.add(fid)
        xml = section_xml(fid, family, contract, source, [ole_xml(c) for c in oles])
        env = envelope(fid, family, contract, source, oles)
        (XML_DIR / f"{fid}.xml").write_text(xml, encoding="utf-8", newline="\n")
        (ENV_DIR / f"{fid}.json").write_text(
            json.dumps(env, ensure_ascii=False, indent=2) + "\n",
            encoding="utf-8",
            newline="\n",
        )
        c0 = oles[0]
        catalog.append(
            "\t".join(
                [
                    fid,
                    family,
                    str(len(oles)),
                    str(c0.get("id", "")),
                    str(c0.get("instid", "")),
                    f"{c0['curSz'][0]}x{c0['curSz'][1]}",
                    f"{c0['offset'][0]},{c0['offset'][1]}",
                    contract.replace("\t", " "),
                ]
            )
        )

    header = "id\tfamily\tole_count\tid_attr\tinstid\tcursz\toffset\tcontract\n"
    (OUT / "catalog.tsv").write_text(header + "\n".join(catalog) + "\n", encoding="utf-8", newline="\n")
    readme = f"""# issue #4669 OLE shape-component fixture corpus

M05-9 / #5450. HWPX 저장이 `hp:ole` 의 shape-component 자식과 `id` 를
보존하는지 고정하는 코퍼스다. pic offset(#4668)·쪽수(#3737)·char_shapes 는
다루지 않는다.

- `xml/` : 픽스처 섹션 XML ({len(cases)}개)
- `envelopes/` : 파싱→저장 기대 봉투 전사 ({len(cases)}개)
- `catalog.tsv` : 색인

생성: `python scripts/generate_issue_4669_ole_fixtures.py`

시험: `tests/cases/issue_4669_ole_shape_component.rs`
"""
    (OUT / "README.md").write_text(readme, encoding="utf-8", newline="\n")
    print(f"wrote {len(cases)} fixtures → {OUT}")


if __name__ == "__main__":
    main()
