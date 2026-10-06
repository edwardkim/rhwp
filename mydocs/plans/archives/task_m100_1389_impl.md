# Task M100 #1389 구현계획서 — 그림 크기 요소 IR 보존

- 수행계획서: `mydocs/plans/task_m100_1389.md` (승인 완료)
- 브랜치: `local/task1389`
- 작성일: 2026-06-14
- 단계: 3단계

## 0. 사전 조사 확정 (1단계 측정 완료)

### 0.1 결손 3축 + IR 보존 상태 (ta-pic IR 덤프)

| 요소 | 원본 pic0 | RT pic0 | IR 보존 | 정정 |
|------|----------|---------|---------|------|
| curSz | 13668×12686 | 18425×18160(sz) | ✓ `shape_attr.current_width/height`=13668×12686 | serializer를 shape_attr로 |
| imgRect | pt2=49380×45840 | common 합성 | ✓ `border_x=[0,0,49380,0]`·`border_y` | serializer를 border_x/y로 |
| imgDim | 49380×45840 | **0×0** | ✗ **미적재** | **모델+파서+serializer** |

- pic1 curSz가 RT에서 정상이던 이유: pic1은 `current_width == sz`(22792)라 common
  방출이 우연히 일치. pic0은 `current_width(13668) ≠ sz(18425)`라 변형 표면화.

### 0.2 imgDim은 verbatim 적재 (clip 파생 금지) — 전수 측정 근거

- imgClip extent(right-left, bottom-top)와 imgDim 비교: 동반 170건 중 일치 146,
  **불일치 24건**. 예: exam-kor clip 102366 vs imgDim 174000(독립), k-water clip
  7260 vs imgDim 0, aift clip left/top≠0(실크롭). → imgDim ≠ clip 파생.
- imgDim은 **원본 이미지 픽셀 크기**(별도 의미)이므로 IR 필드로 verbatim 적재·방출.

### 0.3 게이트 + 판정

- pic 크기는 `diff_documents` 미비교(#1379 한계축). curSz/imgRect/imgDim 게이트 동승은
  Picture 비교에 추가 가능 — 2단계에 포함(IR 보존 검증 겸).
- 최종 확인은 **한컴 편집기 판정**(자기 정합 ≠ 한컴 — #1379 선례).

## 1단계 — (측정 완료) 보고

0절이 1단계 결과. 보고서에 3축·imgDim verbatim 근거·게이트 판정 기재 → 승인.
(코드 수정 없음)

## 2단계 — 모델+파서+serializer+게이트

### 2.1 모델

- `Picture`에 `img_dim: (u32, u32)` (또는 `dim_width/dim_height: u32`) 추가.

### 2.2 파서

- `parse_picture`에 `b"imgDim"` arm: `dimwidth/dimheight` → `pic.img_dim` 적재.

### 2.3 serializer (3함수 정정)

- `write_cur_sz`: `&pic.common` → `&pic.shape_attr` (current_width/height, 0이면
  common 폴백 — 원본도 그 경우 sz=curSz).
- `write_img_rect`: `&pic.common` → `pic.border_x/border_y` 꼭짓점 방출.
- `write_img_dim`: 간이 계산 제거 → `pic.img_dim` 방출.
- 호출부(86/90/93행) 인자 변경.

### 2.4 게이트

- Picture 비교에 curSz(shape_attr current_w/h)·imgRect(border_x/y)·imgDim 추가 —
  `IrDifference::PictureSize`(또는 ObjectComment류). char_shapes 재귀 Picture arm 동승.

### 2.5 단위 테스트

- curSz/imgRect/imgDim 보존 방출 (shape_attr/border_x/img_dim 사용)
- imgDim 파서 적재
- 실샘플(ta-pic) 왕복 — 세 요소 원본 일치
- 게이트: 크기 변형 주입 검출

## 3단계 — 전수 검증 + 문서 + 한컴 판정

1. `hwpx-roundtrip --batch` 전수 → `output/poc/task1389/` (세 요소 복원)
2. ta-pic RT 세 요소 원본 일치 대조
3. baseline + CI급 (release-test + fmt + clippy)
4. 매뉴얼 갱신 (#1389 해소 + 게이트 항목)
5. 최종 보고서 + **한컴 판정 요청** (ta-pic-001-r.rt 셀 그림 크기 반영)

## 위험 관리

| 위험 | 단계 | 대응 |
|------|------|------|
| current_width=0 폴백이 의도된 동작 | 2 | 0이면 common 폴백 (원본 sz=curSz 케이스) |
| border_x 일부 0 (미적재) | 2 | 0이면 common 합성 폴백 |
| imgDim IR 추가가 HWP5 변환 회귀 | 2·3 | HWPX 한정 적재, 전수 배치 회귀 검출 |
| 자기 정합 ≠ 한컴 | 3 | 한컴 판정 게이트 필수 |
