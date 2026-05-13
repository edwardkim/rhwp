# Task #864 Stage A 본질 정밀 진단

**작성일**: 2026-05-13
**브랜치**: `local/task864`

## A.1 WMF binary record dump

`RHWP_DEBUG_WMF=1 ./target/release/rhwp export-svg samples/hwp3-sample14.hwp -p 1` 실행 결과:

- 총 WMF record 수: **822** (scan + play 합산; 실제 record 411 / WMF binary 약 100 record/file × 4 binary)
- 페이지 2 에 WMF binary **2 개** 사용 (record #12, #17 의 STRETCHDIB → 각각 BMP image)

### WMF #1 (그래프 영역) — Window: org=(329, 1536), ext=(4231, 1189)

| 순서 | record | 좌표 | 비고 |
|---|---|---|---|
| #7 | SETWINDOWORG | (329, 1536) | logical origin |
| #8 | SETWINDOWEXT | (4231, 1189) | logical extent |
| #12 | STRETCHDIB | x_dst=329 y_dst=1536 w=2119 h=768 | BMP image #1 |
| #17 | STRETCHDIB | x_dst=2544 y_dst=1536 w=2016 h=768 | BMP image #2 |
| #26~ | POLYGON | y range 1920~2505 | 축/눈금 |
| #74 | POLYGON | (1487,2013) (1440,1920) (1394,2013) | 화살표 |
| #82 | RECTANGLE | x=1111 y=1008 w=1528 h=174 (transformed: y=1008?) | 캡션 outline |
| #96~ | EXTTEXTOUT | y=2665 | 축 텍스트 |

### WMF binary 의 record 종류 (function code 빈도)

| function | 의미 | 횟수 |
|---|---|---|
| 0x0324 | META_POLYGON | 108 |
| 0x012D | META_SELECTOBJECT | 104 |
| 0x0102 | META_SETBKMODE | 104 |
| 0x012E | META_SETROP2 | 100 |
| 0x0626 | META_ESCAPE | 72 |
| 0x0A32 | META_EXTTEXTOUT | 50 |
| 0x0F43 | **META_STRETCHDIB** (image!) | 8 |
| 0x041B | META_RECTANGLE | 4 |
| 0x020C | META_SETWINDOWEXT | 4 |
| 0x020B | META_SETWINDOWORG | 4 |

**중요 부재 record**:
- META_SETMAPMODE (0x0103) — 없음 → default **MM_TEXT** 적용
- META_SETVIEWPORTORG (0x020D) — 없음
- META_SETVIEWPORTEXT (0x020E) — 없음

## A.2 WMF spec 검증

MM_TEXT (기본 모드):
- Logical 1 unit = Device 1 unit (no scaling)
- `device = logical - window_org + viewport_org`
- Default viewport_org = (0, 0)
- 결과: `device = logical - window_org`

따라서 element 좌표는 `point - window_origin` 로 변환되어야 한다.

## A.3 한컴 vs rhwp 실제 element y 좌표 비교 (rendered SVG 분석)

### rhwp WMF#1 embedded SVG 내부 element y 좌표 (`/tmp/864a/hwp3-sample14_002.svg`):

| element | y 좌표 | 비고 |
|---|---|---|
| `<rect>` (캡션 outline) | **y=1008** | RECTANGLE record |
| `<text>` (캡션 텍스트) | **y=1205** | EXTTEXTOUT (raw 2665 → abs(2665-1536)+76(ascent) = 1205) |
| `<image>` (BMP) | **y=1536** | STRETCHDIB (raw 1536, 변환 없음) |

viewBox: `329 1008 4231 1717` (Task #860 Stage D 의 auto-expansion 결과)

### 시각 결과 (rhwp):

viewBox y 범위 1008 ~ 2725 안에서:
- y=1008 (TOP): rect outline
- y=1205 (UPPER-MID): text
- y=1536 (LOWER-MID): image

→ **outline + text 가 image 위에 그려짐** (한컴과 반대)

### 한컴 PDF page 2 시각:

- image 위 (TOP) + outline 아래 + text 아래

## A.4 가설 결판 + **본질 식별**

| 가설 | 결판 | 근거 |
|---|---|---|
| H1 SetMapMode 결함 | ✗ | WMF binary 에 SetMapMode record 없음 (MM_TEXT default) |
| H2 SetViewportOrg/Ext 미처리 | ✗ | WMF binary 에 SetViewport* record 없음 |
| H3 Logical→Device 변환 결함 | ✓ | **이미지 (STRETCHDIB) 와 텍스트/도형의 좌표 변환이 비일관**: text/polygon/rect 는 `point_s_to_absolute_point` 로 origin-relative 변환되나, **image (TernaryRasterOperator) 는 raw logical 좌표 그대로 사용** |
| H4 한컴 y mirror | ✗ | sample 의 WMF binary 모든 좌표 positive, top-down 정합 |

### **본질 위치**: `src/wmf/converter/svg/mod.rs`

**문제**:

1. `ext_text_out` (line 772+) — text 좌표를 `point_s_to_absolute_point` 변환 (origin-relative). text y=2665 → 1205 (raw - origin + ascent).
2. `polygon` (line 1272+) — polygon points 를 `point_s_to_absolute_point` 변환. y=2496 → 960.
3. `rectangle` 등 도형 — 동일한 origin-relative 변환.
4. **`bit_blt` / `stretch_blt` / `device_independent_bitmap_stretch_blt` / `stretch_device_independent_bitmap`** — image 좌표를 **변환 없이 raw 그대로** `TernaryRasterOperator::new(x_dst, y_dst, ...)` 에 전달. image x="329" y="1536" 그대로 SVG 에 입력.

5. `as_view_box` (`device_context.rs:280`) — Task #860 Stage D 에서 `(origin_x, origin_y, x, y)` 반환으로 변경. 이는 **raw logical 좌표 공간**을 viewBox 로 사용한다는 의미. 그러나 text/polygon 은 **origin-relative 공간**으로 변환되어 있음 → viewBox 와 element 공간 mismatch.

### Task #860 Stage D 의 미완 정정

Task #860 Stage D 는 BMP 가 viewBox 밖으로 나가는 문제를 해결하기 위해 viewBox 를 logical origin 으로 옮겼다 (`as_view_box` 변경). 그러나 이는 element 들의 좌표 변환 공간 (origin-relative) 과 mismatch 를 야기하여, **상대적 element 순서가 뒤집힌** 결과를 낳았다.

선행 fix 는 BMP visibility 는 해결했으나 image-vs-text 의 상대 좌표 정합이 깨졌다.

## 정정 방향 (Stage B 평가용)

### 후보 1: 이미지 좌표도 origin-relative 변환 (권장)

`bit_blt`, `stretch_blt`, `device_independent_bitmap_stretch_blt`, `stretch_device_independent_bitmap` 의 x_dst/y_dst 를 `point_s_to_absolute_point` 로 변환.

동시에 `as_view_box` 를 `(0, 0, x, y)` 로 revert (Task #860 Stage D 변경 되돌림).

장점:
- WMF 표준 (Window/Viewport 변환) 정합
- 모든 element 가 같은 공간 (origin-relative = device)
- 변경 범위 작음 (4 함수 + 1 revert)

단점:
- Task #860 Stage D 변경을 revert 하므로 회귀 위험 있음 → 다른 WMF sample 회귀 검증 필요

### 후보 2: text/polygon 좌표 변환 제거 (raw 그대로 사용)

`point_s_to_absolute_point` 호출 제거 → text/polygon 도 raw logical 좌표 사용.

장점:
- viewBox (logical) 와 정합

단점:
- 변경 범위 큼 (모든 element handler)
- abs() hack 제거 부담
- 기존 다른 sample 회귀 위험 큼

### 후보 3: Task #860 Stage D 부분 revert (image 도 viewBox 와 같은 공간)

후보 1 + 2 의 부분 절충안. 실용성 낮음.

## Stage A 결론

본질: **image (TernaryRasterOperator) 의 x_dst/y_dst 가 `point_s_to_absolute_point` 변환을 거치지 않아, 다른 element (text/polygon) 와 좌표 공간이 다름**.

권장 정정: **후보 1** (image 좌표도 origin-relative 변환 + viewBox revert).

## 회귀 위험 평가 (Stage B 사전 검토)

- exam_eng, exam_math 등 sample 의 WMF/EMF 사용 여부 확인 필요
- HWP3 sample14 외의 WMF Placeable BoundingBox 가 (0, 0) origin 인 경우 (대부분) 후보 1 변경은 영향 없음
- viewBox revert 시 Task #860 Stage D 회귀: BMP 가 viewBox 밖으로 나가는 문제 재발 위험 — 해결: auto-expansion 로직 유지하면 됨

📋 **Stage A 완료. Stage B 정정 후보 평가 진행합니다.**
