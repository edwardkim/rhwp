# HWP3 paragraph border_fill 분석 — Task #877 Stage 4 jang진단

**작성일**: 2026-05-14
**관련 task**: #877 sample16.hwp WASM 로드 정합

## 문제 정의

sample16.hwp 의 페이지 16 (한컴 viewer 기준) 본문 영역에 회색 점선 외곽선 박스가 표시되어야 하나, rhwp-studio 에서 일부 paragraph 의 외곽선이 누락됨. HWP5 변환본 (`hwp3-sample16-hwp5.hwp`) 에서는 정상 표시.

## HWP3 vs HWP5 spec 비교

### HWP5 spec (한글문서파일형식 5.0 §표 43 문단 모양)

paragraph 모양 record 에 다음 필드 존재:
- `테두리/배경 모양 ID (BorderFill ID)` — UINT16
- `문단 테두리 왼쪽/오른쪽/위쪽/아래쪽 간격` — INT16 ×4

paragraph 의 외곽선 박스를 명시적 ID 로 참조 → renderer 가 paragraph 영역에 border 그림.

### HWP3 spec (한글문서파일구조 3.0 §5 문단 모양)

paragraph 모양 record:
- offset 180: `음영 비율` — byte (%)
- offset 181: `문단 테두리` — byte (0=없음, 1=있음)
- offset 182: `선 연결` — byte
- offset 183: `margin_top` — u16

HWP5 와 달리:
- **`BorderFill ID` 명시적 필드 없음** — 단일 boolean (border=0/1)
- 색상 / 점선 종류 / 두께 정보 없음

## sample16 실측 데이터

### paragraph 89 (본문 글머리 paragraph, 한컴 viewer 16쪽 영역)

**HWP3 sample16.hwp**:
- `ParaShape.border = 0` (테두리 없음)
- `shade_ratio = 0`
- raw HWP3 자체에 외곽선 정보 부재

**HWP5 변환본 hwp3-sample16-hwp5.hwp**:
- `border_fill_id = 1` (테두리/배경 ID = 1)
- border_spacing left/right/top/bottom = 0

### paragraph 5 (표지 RFP 박스)

paragraph 5 의 외곽선 박스는 **paragraph 자체의 border 가 아닌 별도 picture (ch=11 → ShapeObject::Rectangle drawing object)**:
- HWP3 raw line_color=0x000000 + width=84 HU + line_style=0x0000
- Stage 3 v2 fix: line_style=0x0000 + width>0 → 0x0001 (Solid LineType) 보강
- Stage 4 fix: ref_pos=0 (Text) → horz/vert_rel_to=Para (paragraph inline)

즉 paragraph 5 처럼 **명시적 picture 로 외곽선 박스를 그리는 경우는 rhwp 가 정상 처리** (Stage 3+4 fix 후).

### paragraph 393 (본문 영역 점선 박스)

paragraph 393 도 picture (Rectangle drawing object) — paragraph border 가 아닌 별도 그림:
- raw line_color=0x000000 + width=56 HU + style=0x0002 (Dash 점선)
- Stage 4 점선 가시성 fix: width < 1.0 px → 1.0 px 보강

## 한컴 HWP5 변환기 휴리스틱 추정

HWP5 변환본의 1058 paragraph 전부 `border_fill_id > 0`. 분석:

| 케이스 | HWP5 변환본 처리 |
|--------|----------------|
| 일반 paragraph | `border_fill_id = 1` (default, line_type=0 "선 없음") |
| 본문 영역 paragraph 그룹 | 같은 `border_fill_id` 부여하여 시각 박스 형성 |

즉 한컴 변환기는 paragraph margins/indent 패턴 + 인접 paragraph 그룹화 분석하여 **자동으로 border_fill_id 를 부여**. 이는 **HWP3 raw 정보에서 직접 도출 불가능한 변환기 휴리스틱**.

## rhwp 현 상태 분석 결과

본 task 의 fix 들로 sample16 의 외곽선 박스가 다음과 같이 처리됨:

| 시각 외곽선 | rhwp 처리 | 상태 |
|----------|----------|------|
| 표지 RFP 박스 (paragraph 5 picture) | drawing object Rectangle, Stage 3 v2 fix | ✅ |
| 16쪽 본문 영역 점선 박스 (paragraph 393 picture) | drawing object Rectangle, Stage 4 점선 가시성 보강 | ✅ |
| 다이어그램 외곽선 (paragraph 394 표) | 표 (Table) | ✅ |

**즉 sample16 의 실제 외곽선 박스들은 raw HWP3 의 picture/drawing object 로 표현**되며, Stage 3+4 의 fix 들로 모두 표시됨.

HWP5 변환본의 paragraph border_fill_id 는 한컴 변환기가 default 부여한 추가 정보 (대부분 line_type=0 "선 없음") 이며, 실제 시각 외곽선은 별도 picture 객체로 그려짐.

## 결론

1. **HWP3 paragraph 의 raw border 필드 (offset 181)** 는 binary 0/1. sample16 의 paragraph 89/91 = 0 → 외곽선 없음 (정합).

2. **실제 시각 외곽선 박스**는 HWP3 raw 에서 별도 picture (ch=11 → ShapeObject::Rectangle drawing object) 로 표현. Stage 3+4 fix 로 정상 표시.

3. **한컴 HWP5 변환본의 paragraph.border_fill_id** 는 변환기가 모든 paragraph 에 부여한 default 정보. 시각 외곽선과 직접 대응 안 됨.

4. **추가 휴리스틱 부재 정당함**: paragraph margins 패턴 → border_fill 자동 부여 같은 휴리스틱은 sample 별 차이 크고 HWP3 spec 외 영역이라 회귀 위험 큼. **HWP3 의 raw picture/drawing object 처리가 정상화되면 시각 외곽선은 자연스럽게 표시**됨.

## Task #877 잔여 시각 차이의 실체

사용자 screenshot 비교 결과 잔여 시각 차이의 원인:

| 시각 차이 | 실체 | 본 task 해결 |
|----------|------|------------|
| 표지 RFP 박스 누락 | paragraph 5 picture 의 ref_pos=0 위치 기준 누락 | ✅ Stage 4 |
| 16쪽 점선 박스 가시성 | width=56 HU 점선 가시성 부족 | ✅ Stage 4 |
| paragraph 89 글머리 누락 | PUA U+F03C5 font fallback 부재 | ✅ Stage 4 (○ 매핑) |
| paragraph 91 ◦ 글머리 누락 | HWP3 raw 정보 부재 (한컴 변환기 휴리스틱) | ✅ Stage 4 (margins 패턴 휴리스틱) |
| 다이어그램 미표시 | WMF magic detection 누락 | ✅ Stage 4 |

**paragraph border_fill 자체의 자동 부여 휴리스틱은 불필요** — 위 fix 들이 시각 외곽선의 실체 (picture/drawing object) 를 모두 정상화.

## 변경 없음

본 분석 결과 paragraph border_fill 자동 부여 휴리스틱 도입 불필요. 이미 적용된 fix 들이 충분.

향후 시각 차이 발견 시 paragraph border_fill 가 아닌 **HWP3 picture/drawing object 의 처리** 또는 **decode_hwp3_extra PUA 매핑** 영역에서 분석할 것.
