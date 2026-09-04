# Task M100 #1389 — 2단계 완료 보고서 (모델+파서+serializer+게이트)

- 브랜치: `local/task1389`
- 작성일: 2026-06-14
- 수정 파일: `src/model/image.rs`, `src/parser/hwpx/section.rs`,
  `src/serializer/hwpx/picture.rs`, `src/serializer/hwpx/roundtrip.rs`,
  `src/wasm_api/tests.rs`(초기화)

## 1. 구현 내용 (결손 3축)

### 2.1 모델

- `Picture.img_dim: (u32, u32)` 추가 (원본 이미지 픽셀 크기 verbatim).

### 2.2 파서

- `parse_picture`에 `b"imgDim"` arm: `dimwidth/dimheight` → `pic.img_dim`.

### 2.3 serializer (3함수 정정)

- `write_cur_sz(pic)`: `shape_attr.current_width/height` 방출 (0이면 common 폴백).
- `write_img_rect(pic)`: `border_x/border_y` **스칼라 레이아웃 역매핑**
  (`border_x=[pt0.x,pt0.y,pt1.x,pt1.y]`, `border_y=[pt2.x,pt2.y,pt3.x,pt3.y]`) →
  pt0~pt3 복원. 전부 0이면 common 합성 폴백.
- `write_img_dim(pic)`: 간이 계산 제거 → `pic.img_dim` verbatim.

**구현 중 정정**: border_x를 pt{i}.x로 직출했더니 imgRect 좌표가 어긋남 → 파서
`parse_picture_img_rect`가 HWP5 SHAPE_PICTURE 스칼라 레이아웃(꼭짓점 x/y 배열 아님)
으로 저장함을 확인하고 역매핑 적용.

### 2.4 게이트 (`PictureSize` 동승)

- `IrDifference::PictureSize` + `diff_picture_size`: curSz(shape_attr current)·
  imgRect(border_x/y)·imgDim 비교. Picture char_shapes arm 동승.

### 2.5 단위 테스트

| 테스트 | 검증 |
|--------|------|
| `task1389_cur_sz_uses_shape_attr` (picture) | curSz shape_attr 사용 |
| `task1389_cur_sz_falls_back_to_common_when_zero` | 0 폴백 |
| `task1389_img_rect_uses_border_scalar_layout` | 스칼라 레이아웃 역매핑 |
| `task1389_img_rect_synthesizes_when_border_zero` | border 0 합성 폴백 |
| `task1389_img_dim_verbatim` | imgDim verbatim (clip 파생 금지) |
| `task1389_picture_size_diff_in_gate` (roundtrip) | 크기 변형 검출 |
| `task1389_ta_pic_size_roundtrips` (roundtrip) | 실샘플 게이트 0 |

`cargo test --lib serializer::hwpx` 234 passed / fmt 통과.

## 2. 검증

### 2.1 ta-pic 세 요소 복원 (실측)

| 요소 | 원본 pic0 | 종전 RT | 수정 후 RT |
|------|----------|---------|-----------|
| curSz | 13668×12686 | 18425×18160 | **13668×12686** |
| imgRect | pt0~3 (49380,45840) | common 합성 | **원본 일치** |
| imgDim | 49380×45840 | 0×0 | **49380×45840** |

### 2.2 통합 테스트 (`tests/issue_1389_picture_size_roundtrip.rs`)

- ta-pic 그림 크기 4요소 멀티셋 보존 + 2-round 안정 — 1 passed.

### 2.3 전수 배치 (`output/poc/task1389/`)

- PASS 53 / **IR_DIFF 0** (게이트 동승 후에도 차이 0) / SERIALIZE_FAIL 0 / PARSE_FAIL 1(제외)
- baseline 4 passed — B=0 유지 (#1384 승격분 회귀 없음).

## 3. 다음 단계

3단계 — 매뉴얼 + CI급(release-test) + 최종 보고서 + **한컴 판정 요청**
(ta-pic-001-r.rt 셀 그림 크기 반영).

승인 요청드립니다.
