# #2015 Stage 4 findings — HWPX 인라인 표 pi=50 별표 셀 valign 무력화 (발원지 ②)

- 이슈: #2015 / 브랜치: `fix/2015-saved-bounds-rowbreak-overflow`
- 범위: 소스 무수정. 발원지 ②(p4 상단 인라인 표 pi=50 세로 드리프트) 근본원인 확정 + 수정 판단.

## 1. 재특성화 — 피치가 아니라 셀 내용 세로정렬

시각 오라클(PDF=PyMuPDF) 좌표 대조:

- 별표 줄 **피치는 rhwp/PDF 동일**(24.3/30.9px 교대) → 피치 버그 아님.
- 별표 **블록 크기 동일**(span ≈ 269px).
- 차이는 **박스 내부 "제목→별표 시작" 간격**:
  - rhwp(HWPX): 제목 359.3 → 별표 382.6 (간격 23.3px)
  - rhwp(HWP): 제목 359.9 → 별표 413.6 (간격 53.7px)
  - 한컴 PDF: 제목 360 → 별표 420.5 (간격 60px)
- 박스 위 본문·제목·표 bbox(355.2, h=383.6)는 HWPX/HWP **동일**하고 ~1px 정합.

즉 **같은 박스 안에서 별표(row 2 셀[5]) 내용이 HWPX 에서만 37px 위로 붙는다.**

## 2. 근본원인 — HWPX 합성 셀 내용높이 과대계산 → valign=Center 무력화

`table_layout.rs` 셀 세로정렬(2688~):
```
VerticalAlign::Center => mechanical_offset = (inner_height − total_content_height).max(0)/2
```

`RHWP_VALIGN_DBG` 계측 (pi=50 별표 셀[5], valign=Center):

| | inner_height | total_content_height | mech_off |
|---|---|---|---|
| HWPX | 354.3 | **362.8** | **0.0** (내용>셀 → 클램프) |
| HWP  | 353.2 | **293.3** | **29.9** (센터링) |

- 셀 inner_height 는 동일(≈354). **total_content_height 만 HWPX 가 69.5px 과대**(362.8 vs 293.3).
- 렌더 별표 span 은 둘 다 269px 인데 **계산 content-height 만 다르다** → HWPX 합성
  lineSeg/문단 높이 과대계산.
- 과대계산분이 셀 높이를 초과 → `mech_off` 가 0 으로 클램프 → 별표가 셀 상단에 붙어 37px 위.
- IR(`dump`)은 HWPX/HWP 동일(컨테이너 lh=28769, 셀 999/1998/26771). 유일 차이는 lineSeg
  tag synthetic 비트(0x80000000)와 ls(720 vs 1200). → **렌더 경로의 HWPX 합성 content-height 문제.**

## 3. 영향·심각도

- **오버플로우/페이지수/본문 흐름 영향 없음**: 박스 아래 본문은 재정렬되어 정합(“(사회기여)”
  rhwp 765.7 vs PDF 768, 2.3px). 순수 **박스 내부 별표 위치 37px** 국소 시각 오차.
- 발원지 ①(오버플로우, 실제 조판 결함)과 성격이 다르다 — ②는 미관 수준.

## 4. 수정 판단 — 후속 스코프 권장

- 수정 locus: HWPX 합성 lineSeg/문단 content-height 계산(composer 계열). 이 값은 **센터링뿐
  아니라 행높이·오버플로우 검출에도 쓰여** 모든 HWPX 표 다중줄 셀에 영향 → **광범위 회귀 표면**.
- 이득은 미관(별표 박스 센터링), 위험은 전 HWPX 표 조판. 리스크-이득 비대칭.
- 프로젝트 방법론상 레이아웃 변경은 시각 판정 최종 게이트 + 다수 HWPX 샘플 회귀 필수.
- 따라서 **발원지 ②는 별도 스코프 타스크로 분리** 권장: HWPX 합성 content-height 를 HWP 저장
  경로와 정합(362.8→~293)시키되, HWPX 코퍼스 전수 시각/구조 회귀로 검증.
- 본 이슈 #2015 는 발원지 ①(오버플로우) 확정 수정으로 마무리하고, ②는 findings 로 인계.

## 5. 재현

```bash
# 별표 셀 valign 클램프 계측(디버그 필요 시 table_layout.rs Center 분기에 일시 print)
# 좌표 대조:
python scripts/visual_oracle_native.py --hwp <hwpx> --pdf <pdf> --pages 4 --out output/poc/oracle
# rhwp render tree(HWPX vs HWP) 별표 첫 줄 y: 382.6 vs 413.6 (한컴 420.5)
```
