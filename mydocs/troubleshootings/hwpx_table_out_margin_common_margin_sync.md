---
kind: reference
status: historical
canonical: mydocs/troubleshootings/README.md
last_verified: 2026-07-16
---

# Issue #1772 조사 보고서 — HWPX/HWP5 파스 경로별 saved-vpos 신뢰 차이의 근본 원인

## 요약

hwpdocs 코퍼스 5,600건 중 ~94건(OVER)의 최대 단일 원인이던 "11.36px 이동 군집"(50건+)의
근본 원인을 확정했다. **saved-vpos 신뢰 로직 차이가 아니라, HWPX 파서가 표의 outMargin 을
`table.common.margin`(CommonObjAttr)에 동기화하지 않는 IR 계약 위반**이다. 이슈 제목의
가설(신뢰 경로 차이)은 기각한다.

## 인과 사슬 (재현: 결재문서 36381023, seoul_071)

1. **HWPX 파서** (`src/parser/hwpx/section.rs` parse_table): `<hp:tbl><hp:outMargin>` 을
   `table.outer_margin_*` 에만 기록하고 `table.common.margin` 은 0 으로 남긴다.
2. **HWPX→HWP 어댑터** (`src/document_core/converters/hwpx_to_hwp.rs:1456`
   `materialize_table_outer_margin`): 직렬화 시점에 `common.margin ← outer_margin_*` 를
   명시 동기화한다. 따라서 HWP5 재파스본은 `common.margin.bottom = 852`(3mm)를 갖는다.
   — 이 함수의 존재 자체가 "두 필드는 동기 상태여야 한다"는 IR 계약의 방증이다.
3. **레이아웃** (`src/renderer/layout/shape_layout.rs:3247` `calc_shape_bottom_y`):
   쪽 고정(vert=Page) 자리차지(TopAndBottom) 표의 본문 예약 하단을
   `shape_y + shape_h + common.margin.bottom` 으로 계산한다 — `outer_margin_*` 이 아니라
   **`common.margin`** 을 참조.
4. 결과: HWPX 직파스 문서에서만 표 아래 여백(3mm=852HU=11.36px)이 무시되어 본문 첫 줄이
   표 하단에 붙는다(y=295.4). HWP5 경로는 306.7 — **한컴 저장 lineseg(pi=0 vpos=17478
   → 75.6+233.0≈306.7)와 일치하므로 HWP5 경로가 정답, HWPX 파서가 결함**.

## 검증 근거

- 플래그 실험: `is_hwpx_source` 강제 플립(RHWP_FORCE_HWPX_SOURCE) → 렌더 불변.
  신뢰 로직 차이 가설 기각.
- 데이터 실험: 셀 name 속성 제거 변형 HWPX → 불변. 필드 소실 가설 기각.
- 계측: 본문 pi=0 첫 줄 text_y 가 layout 진입 시점에 이미 295.4/306.7 로 분기 —
  상류는 `calculate_shape_reserved_heights` → `calc_shape_bottom_y` 의 예약 하단.
- **동기화 실험(확증)**: HWPX 파서에 `common.margin ← outer_margin_*` 동기화를 임시 적용
  (RHWP_1772_SYNC_MARGIN) → seoul_071 본문 첫 줄 306.7 로 이동, HWP5 경로와 완전 일치.
- 코퍼스 300건: OVER 13→10, PASS 285→288. 11.36px 군집 5건 중 3건 완전 해소(PASS),
  2건은 11.36→9.25px 로 감소 (잔여는 아래 별개 결함 b).

## 파생 결함 (별개 수정 필요)

**(b) `cell.apply_inner_margin` 파스 불일치**: HWPX 파스 = false, HWP5 재파스 = true.
seoul_071 두 번째 표 높이 244.9px(A) vs 254.2px(B) — 9.3px 차이의 원인. 동기화 실험 후
잔여 OVER(9.25px)가 이것. HWPX 파서의 셀 플래그 파싱 또는 HWP 직렬화의 셀 속성 기본값
중 어느 쪽이 한컴 정합인지 후속 판정 필요.

**(c)** admrul_1043(통일부 2828615)의 PAGE_MISMATCH 도 예약 높이 차이가 페이지 경계를
넘는 케이스로 추정 — (a) 수정 후 재검 필요.

## 수정 방향 제안 (승인 요청)

HWPX 파서 parse_table 종료 시점에 `table.common.margin ← table.outer_margin_*` 동기화.
(대안: 로드 후 normalize 단계 `commands/document.rs` 의 `normalize_hwpx_paragraphs` 옆에
문서 전체 표 순회 동기화 — 파서 대칭성 관점에서는 파서 내 동기화가 자연스러움.)

**주의(이중 적용 검토)**: `outer_margin_*` 을 직접 참조하는 레이아웃 경로(TAC 표 등)와
`common.margin` 참조 경로가 공존한다. 동기화 후 같은 여백이 두 번 더해지는 경로가 없는지
전수 확인 필요 — 코퍼스 검증 결과 **300건: 개선 5(완전 해소 3)·악화 0 / 2,500건: 개선 55
(OVER 90→55, PASS 2369→2404)·악화 0**. 이중 적용 회귀는 관측되지 않았다.

## 재현 명령

```
rhwp export-render-tree <36381023.hwpx> -p 0   # 본문 첫 TextLine y=295.4 (결함)
rhwp convert <36381023.hwpx> out.hwp && rhwp export-render-tree out.hwp -p 0   # 306.7 (정답)
rhwp dump <파일> -s 0 -p 0                      # ls[0] vpos=17478 (한컴 저장 위치)
```
