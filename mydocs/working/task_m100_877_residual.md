# Task #877 잔존 문제 분석 — 별도 task 권장

**작성일**: 2026-05-14
**관련 task**: #877 (sample16.hwp 정합)

## 잔존 문제 3건

본 task 의 fix 22 commits 누적으로 sample16.hwp 의 핵심 시각 정합 (페이지 수, 외곽선, 글머리, 다이어그램 표시) 모두 해소. 그러나 추가 세부 시각 차이 3건 잔존 — 본 task 범위 (HWP3 파서) 외 또는 광범위 렌더러 영역.

### 1. HWP3 페이지 외곽선 위치 — 본문 영역 vs page border 크기 불일치

**증상**: sample16 페이지 2 (목차) 의 목차 항목 끝 페이지 번호들 (1, 3, 5, 6, ...) 이 페이지 외곽선 박스 **밖에** 표시됨. 한컴 viewer (HWP5 변환본) 에서는 외곽선 안에 표시.

**원인 추정**:
- 본 task 의 page border IR 변환 (c8ba53b) 에서 `border_margin*=355 hunit (5mm)` 을 `spacing_*` 으로 설정
- 그러나 renderer ([renderer/layout.rs:748-776](../../src/renderer/layout.rs#L748-L776)) 의 `attr & 0x01` 가드:
  - `paper_based = false` → body_area 기준 (= 페이지 여백 안)
  - 그러면 외곽선 = body_area + spacing
  - 그러나 paragraph 텍스트 (목차 우측 페이지 번호 영역) 가 body_area 의 width 를 초과하는 듯
- 또는 한컴 viewer 의 page border 좌표가 paper_based 인데 rhwp 는 body_based 라 크기 다름

**영역**: rhwp renderer / layout (page border 좌표 기준).

**별도 task 권장**.

### 2. HWP3 paragraph 394 다이어그램 중복 emit (SVG 3 image)

**증상**: sample16 페이지 18 (한컴 16쪽) 의 paragraph 394 [1] 그림 (WMF, bin_id=3) 이 SVG 에 **3개 `<image>` 로 emit**됨 (모두 동일 href). 시각상 2~3개 다이어그램 중복.

**dump 결과**:
- paragraph 394: ls_count=3, controls=3 ([0] 표 + [1] 그림 + [2] 표)
- text: "￼￼  ￼" (3 picture markers)

**시도한 fix (효과 없음)**:
- treat_as_char picture wrap=Square → TopAndBottom 정합 (9e9d1bf / rebase 후 SHA): wrap 변경 후에도 image 3개 그대로
- → wrap 영향 아님. paragraph 의 multi-line + picture-per-line 처리 영역 bug

**영역**: rhwp 렌더러의 paragraph multi-line picture 처리 (typeset.rs / picture_footnote.rs / svg.rs).

**별도 task 권장**.

### 3. HWP5 변환본 페이지 수 inflate

**증상**: `samples/hwp3-sample16-hwp5.hwp` (한컴 HWP3 → HWP5 변환본) 를 rhwp 가 **98 페이지**로 인식. 한컴 viewer 표시 = **62 페이지**.

**관찰**:
- rhwp HWP3 sample16: 64 페이지 (한컴 정합)
- rhwp HWP5 변환본: 98 페이지 (한컴 62 페이지)
- HWP5 변환본은 외곽선 / 그림 / 글머리 등 시각 정합도 sample16 HWP3 보다 더 정확

**원인**: rhwp 의 **HWP5 파서 또는 pagination 영역**. 본 task #877 (HWP3 파서) 범위 외부.

**별도 task 권장**.

## 종합

본 task #877 의 핵심 목표 (HWP3 sample16 WASM panic 차단 + paragraph alignment + 핵심 시각 정합) 는 22 commits 로 완전 달성. 잔존 3건은 모두 본 task 범위 외 영역 (rhwp 렌더러 / HWP5 파서).

각각 별도 issue 등록 후 task 진행 권장:
1. HWP3 페이지 외곽선 좌표 기준 정합 (renderer)
2. paragraph multi-line picture SVG image 중복 emit (renderer)
3. HWP5 변환본 페이지 수 inflate (HWP5 파서 또는 pagination)
