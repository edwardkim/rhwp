# Task M100 #1389 최종 보고서 — 그림 크기 요소(curSz/imgRect/imgDim) IR 보존

- 이슈: #1389 "HWPX serializer: hp:pic 크기 요소(curSz/imgRect/imgDim) IR 미반영 — 셀 내 그림 크기 변형"
- 마일스톤: M100 (v1.0.0), #1315 하위
- 브랜치: `local/task1389`
- 작성일: 2026-06-14

## 1. 결함과 해소 (결손 3축)

#1379 한컴 시각 판정 발견 — ta-pic-001-r.rt 셀 안 이미지 크기 미반영.

| 요소 | 종전 RT | IR 보존 | 해소 |
|------|---------|---------|------|
| curSz | sz 값(common) 방출 — current≠sz pic 변형 | ✓ `shape_attr.current_width/height` | serializer를 shape_attr로 (0 폴백 common) |
| imgRect | common 합성 | ✓ `border_x/border_y` (HWP5 스칼라 레이아웃) | border 역매핑 방출 (0 폴백 합성) |
| imgDim | 간이 계산 → 0×0 | ✗ 미적재 | **IR 필드 추가 + verbatim 적재·방출** |

imgDim을 imgClip 파생이 아닌 **verbatim**으로 한 근거: 전수 측정에서 imgDim ≠
imgClip extent 불일치 24/170 (exam-kor clip 102366 vs imgDim 174000 등 독립).

## 2. 단계 요약

| 단계 | 내용 | 커밋 |
|------|------|------|
| 1 | 결손 3축 + IR 보존 상태 + imgDim verbatim 근거 (구현계획서 0절) | `04342203` |
| 2 | 모델+파서+serializer 3함수+PictureSize 게이트 + 테스트 7종 | `0b2233d4` |
| 3 | 매뉴얼 + CI + 최종 보고서 | (본 커밋) |

수정 파일: `src/model/image.rs`, `src/parser/hwpx/section.rs`,
`src/serializer/hwpx/{picture,roundtrip}.rs` — 렌더러·레이아웃 무변경.

## 3. 검증

### 3.1 ta-pic 세 요소 복원

| 요소 | 원본 pic0 | 종전 RT | 수정 후 RT |
|------|----------|---------|-----------|
| curSz | 13668×12686 | 18425×18160 | **13668×12686** |
| imgRect | pt0~3 (49380,45840) | common 합성 | **원본 일치** |
| imgDim | 49380×45840 | 0×0 | **49380×45840** |

### 3.2 전수 배치 (`output/poc/task1389/`)

- PASS 53 / **IR_DIFF 0** (PictureSize 게이트 동승 후에도 차이 0) / SERIALIZE_FAIL 0 /
  PARSE_FAIL 1(제외 hwpx-01) / ROUND2_DIFF 0
- baseline 4 passed — **B=0 유지** (#1384 승격분 회귀 없음).

### 3.3 통합 테스트

`tests/issue_1389_picture_size_roundtrip.rs` — ta-pic 그림 크기 4요소 멀티셋 보존
+ 2-round 안정, 1 passed.

### 3.4 CI급 검증 (release-test 프로필)

- `cargo test --profile release-test --tests` — **2326 passed, 0 failed** (기존 2318 + 신규 8: picture 5 + 게이트 2 + 통합 1)
- `cargo fmt --check` 통과, clippy 경고 0

## 4. 구현 중 발견 (보고)

`write_img_rect`에서 border_x를 pt{i}.x로 직출했더니 imgRect 좌표 어긋남 → 파서
`parse_picture_img_rect`가 **HWP5 SHAPE_PICTURE 스칼라 레이아웃**
(`border_x=[pt0.x,pt0.y,pt1.x,pt1.y]`, `border_y=[pt2.x,pt2.y,pt3.x,pt3.y]`)으로
저장함을 확인하고 역매핑 적용. 신규 결함 아님 — 보존 방출 정정 과정의 매핑 정합.

## 5. 한컴 판정 요청

`output/poc/task1389/ta-pic-001-r.rt.hwpx` 1건 (한컴 편집기):

- **셀 안 이미지 크기가 원본대로 반영**되는지 (#1379 판정의 "셀 내 이미지 크기
  미반영" 해소 확인). 표 캡션(#1387 해소)·여백(#1388 해소)은 별개 — 이미지 크기에
  집중.

## 6. 잔존 한계 (기지 이슈)

| 한계 | 이슈 |
|------|------|
| 표 pageBreak 일괄 TABLE 방출 | #1393 |
| 열거 속성 표면 표기 정합 검사 | #1402 |
| newNum 슬롯 위치 + 143E RT 페이지 수 | #1407 |
| numbering 등록 축 잠재 불일치 | #1409 |

신규 발견 없음.

## 7. 산출물

- 계획서: `mydocs/plans/task_m100_1389{,_impl}.md`
- 단계별 보고서: `mydocs/working/task_m100_1389_stage2.md`
- 매뉴얼 갱신: `mydocs/manual/hwpx_roundtrip_baseline.md` (게이트 항목 + #1389 해소)
- 검증 산출물: `output/poc/task1389/`
