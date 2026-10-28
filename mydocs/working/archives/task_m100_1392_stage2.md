# Task M100 #1392 — 2단계 완료 보고서 (파서 + serializer + 게이트)

- 브랜치: `local/task1392`
- 작성일: 2026-06-14
- 수정 파일: `src/parser/hwpx/section.rs`, `src/serializer/hwpx/{picture,shape,section,roundtrip}.rs`

## 1. 구현 내용 (4경로)

### 2a. 파서 (equation/container 적재)

- `parse_equation`(:4820): `b"shapeComment"` arm 추가 → `common.description` 적재
  (`read_dutmal_text` 공유). `common`은 이미 `Equation`에 전달되므로 모델 변경 불요.
- `parse_container`(:3703): caption arm(#1403) 옆에 shapeComment Start-guard arm 추가
  → `common.description` 적재 (GroupShape.common 경유).
- pic/rect는 기존 적재 정상 — 무수정.

### 2b. serializer (방출)

- `write_shape_comment`(shape.rs)를 `pub(super)`로 공유 (빈 description 미방출 가드 유지).
- `write_picture`: caption(#1403) 직후 호출 — aift 실물 공존 순서.
- `render_equation`(section.rs): outMargin 직후 `hp:shapeComment` 삽입 (xml_escape 경유).
- `write_container_close`(shape.rs): `common` 파라미터 추가 + caption 직후 호출.
- rect(`write_rect`)는 기존 방출 정상 — 무수정.

### 2c. 게이트 (`ObjectComment` 동승)

- `IrDifference::ObjectComment` variant + Display + `diff_object_comment` 헬퍼.
- char_shapes 재귀: Picture arm·Equation arm 신설 + `diff_shape_char_shapes`에
  `shape_common` 접근자 경유 description 비교 (Group 자식 자동 재귀).
- 소비처 3곳(roundtrip_ir_diff / baseline / 배치 IR_DIFF) 자동 동승.

## 2. 단위 테스트

| 테스트 | 검증 |
|--------|------|
| `task1392_pic_shape_comment_emitted_after_caption` (picture) | pic 방출 + caption→shapeComment 순서 |
| `task1392_pic_no_description_omits_comment` (picture) | 빈 설명 미방출 |
| `task1392_pic_comment_loss_in_gate` (roundtrip) | 소실 검출 + path·detail 고정 |
| `task1392_equation_comment_loss_in_gate` (roundtrip) | equation 경로 검출 (`/ctrl[0]eq`) |
| `task1392_shape_comment_loss_in_gate` (roundtrip) | 도형 경로 검출 (`/ctrl[0]shape`) |
| `task1392_equal_comment_no_diff` (roundtrip) | 동일 설명 차이 0 |

`cargo test --lib serializer::hwpx` 193 passed / `--lib parser::hwpx` 79 passed / fmt 통과.

## 3. spot·전수 검증

### 3.1 통합 테스트 (`tests/issue_1392_shape_comment_roundtrip.rs`)

- aift(pic 13+container 2)/math-001(equation 44+pic 1)/온새미로(pic 31+container 1+rect 1)
  3샘플 parse→serialize→재parse description 멀티셋 보존 + 2-round 안정 — **3 passed**.

### 3.2 전수 배치 (`output/poc/task1392/`)

- PASS 49 / **IR_DIFF 0** (게이트 동승 후에도 차이 0) / SERIALIZE_FAIL 4(#1384) /
  PARSE_FAIL 1(제외) / ROUND2_DIFF 0
- **RT 가능 파일 전수 shapeComment: 원본 119 → RT 119 (불일치 0)**
  (나머지 exam_kor 계열은 #1384 SERIALIZE_FAIL로 RT 부재 — 기존 xfail)
- baseline 4 passed — **신규 xfail 0** (1단계 예측 적중)

## 4. 다음 단계

3단계 — 매뉴얼 갱신 + CI급(release-test) + 최종 보고서.

승인 요청드립니다.
