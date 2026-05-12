# Task #860 Stage D-A 단계 보고서

**선행**: Stage D 계획서 (수행+구현 승인)
**브랜치**: `local/task860`
**작성일**: 2026-05-13

## 작업 요약

Task #860 의 잔여 결함 (paragraph 순서 / page boundary) 본질 진단. dump-pages + SVG inner element layout 분석.

## 진단 결과

### A1. rhwp page 1/2 boundary paragraph 분포

**Page 1 (idx=0)**:
- pi=0~13 본문 텍스트 + 빈 paragraph
- pi=14 **그림** (h=315.5 px, TAC, vpos=36100)
- pi=15 본문 "'p' 명령은 삭제된 내용을 커서 다음에 붙여 넣었다..." (h=26.7)

**Page 2 (idx=1)**:
- **pi=16 그림** (h=181.1 px, TAC, vpos=0)
- pi=17 본문 "삭제된 내용을 붙여넣는 것은..." (h=26.7)
- pi=19 본문 "Yank..."
- pi=20 그림 (h=209.9, TAC)
- pi=22 "1.2 마크"

### A2. 한컴 PDF page 1/2 텍스트

| 페이지 | 한컴 PDF 시작 텍스트 |
|---|---|
| Page 1 | "3 블록 설정, 다중창 및 단축키 지정 지금까지 배워온 vi 에..." |
| Page 2 | "삭제된 내용을 붙여넣는 것은..." |

→ **한컴 page 2 시작 = rhwp 의 pi=17** (본문). rhwp 의 pi=16 (그림) 은 한컴 정합으로는 page 1 의 마지막 또는 page 2 의 후반.

### A3. rhwp page 2 inner SVG element 분석

`<image href="data:image/svg+xml;base64,..." />` 의 inner SVG 의 element y 좌표 (viewBox 0 0 4231 3000):

| 요소 | y 좌표 | 비고 |
|---|---|---|
| rect (캡션 outline 점선) | **1008** (위) | 점선 박스 |
| text (캡션 글자) | 1205 | "'P' 명령 이후 d 앞에 ple 가 붙여짐" |
| image (BMP, Vim 박스) | **1536** (아래) | terminal screenshot |

**rhwp 시각**: 캡션 outline (위, y=1008) + 캡션 텍스트 (1205) + BMP image (아래, y=1536)
**한컴 PDF 시각**: BMP image (위) + 캡션 outline (아래) + 캡션 텍스트 (밑)

→ **rhwp 의 WMF metafile element 들의 y 좌표가 한컴과 반대 방향**.

### A4. 가설 결판

| 가설 | 결판 |
|---|---|
| H1 Page boundary 가 그림과 캡션 사이 | ✗ (page boundary 는 paragraph 사이) |
| H2 Paragraph 순서 IR 해석 차이 | ✗ (paragraph 순서는 정상) |
| H3 그림 paragraph 의 page_break flag | ✗ |
| **H4 그림 control 내 element 의 layout 순서** | **✓ 확정** (WMF metafile element y 좌표 한컴 반대) |

추가 가설:
- **H5**: WMF 의 좌표계 (top-down vs bottom-up) 처리 차이 — `src/wmf/converter/svg/` 의 SetMapMode / SetWindowOrg / SetWindowExt 의 y direction
- **H6**: WMF SVG 결과의 element 들이 y-flip 필요 (= y → viewBox_height - y - elem_height)

### 본질 위치 추정

`src/wmf/converter/svg/` 의 좌표 처리:
- `device_context.rs` 의 SetWindowExt / SetWindowOrg / SetMapMode 처리
- 또는 element y 좌표 변환 시 y-flip 필요한 경우

WMF 의 좌표 처리는 sample 별로 다를 수 있음 (MM_TEXT vs MM_ANISOTROPIC 등 map mode).

## 정정 후보

### 후보 1 (가능성, 보수적 검증 필요): WMF SVG 결과의 element y 좌표 flip

`convert_wmf_to_svg` 결과 SVG 의 element 들의 y 좌표를 viewBox 안에서 flip (y → viewBox_height - y - elem_height).

**위험**: 다른 WMF sample (다른 map mode / 좌표계) 회귀 가능성 매우 큼.

### 후보 2 (본질적): WMF parser 의 SetWindowExt / SetWindowOrg 의 y direction 정정

WMF binary 의 actual values 추출 + 한컴 정합으로 변환 로직 수정.

**위험**: WMF spec 의 정확한 좌표 처리 이해 필요. 회귀 위험.

### 후보 3: WMF binary 의 BoundingBox / SetMapMode 분석 + 적절한 처리

WMF binary 의 헤더 / records 의 모든 좌표 변환 정보 추출 + 한컴 정합으로 정정.

**가장 본질**, **가장 깊은 작업**.

## 후속 단계

Stage D-B: 정정 후보 (1~3) 평가 + 회귀 위험 측정. 우선 가장 보수적 검증 필요.

## 산출 아티팩트

- `output/issue_hwp3s14/rhwp_p2_after.png` (rhwp page 2 fix 후 PNG)
- 본 보고서
