# Task #489 최종 결과보고서 — Picture+Square wrap 호스트 텍스트 LINE_SEG 적용

**이슈**: GitHub #489
**브랜치**: `local/task489`
**기간**: 2026-04-30
**관련**: Task #488 (Stage 2 시각 검증에서 발견되어 분리)

## 1. 결함 요약

`samples/exam_science.hwp` 페이지 1 컬럼 1 (단 1, 우측 컬럼) 의 5번 문제 본문(pi=21) 에서 두 가지 시각 결함이 발생.

1. **그림 가림**: 첫 줄 "용기에 H₂O(l)을" 부분이 어울림(Square wrap) 그림 영역(z=180)에 가려져 렌더되지 않음
2. **비정상 단어 간격**: 첫 4~6 줄이 풀컬럼 너비(410px)로 justify 되어 어절 사이 간격이 60~100px

## 2. 근본 원인

`src/renderer/layout/paragraph_layout.rs:layout_composed_paragraph` 함수가 `available_width = col_area.width - margins` 로 계산하여 LINE_SEG 의 `segment_width=19592 HU (~261px)` 를 무시.

표(Table)+Square wrap 케이스(Tasks #362/#439/#463)는 caller (`layout.rs:2399`/`:2634`) 가 `wrap_area` 로 좁아진 col_area 를 만들어 우회. 그러나 그림(Picture)/도형(Shape)+Square wrap 케이스는 호스트 paragraph 와 같은 paragraph 에 anchor (vert=Para+0) 되어 별도 우회 경로가 없었고, LINE_SEG.cs/sw 는 height 계산에만 반영되고 텍스트 그리기 폭에는 미반영.

## 3. 수정 사항

### 3.1 `src/renderer/layout/paragraph_layout.rs`

`layout_composed_paragraph` 에 두 단계 변경:

1. **함수 시작부**: 비-TAC `Control::Picture` / `Control::Shape` 중 `text_wrap == TextWrap::Square` 보유 여부를 1회 계산 (`has_picture_shape_square_wrap`).
2. **줄별 루프**: `has_picture_shape_square_wrap && comp_line.segment_width > 0 && comp_line.segment_width < col_area_w_hu - 200` 조건 충족 시 LINE_SEG.cs/sw 를 px 변환하여 `effective_col_x`/`effective_col_w` 로 적용. 이를 line bbox, available_width, alignment x_start, 인라인 TAC 컨트롤 배치에 반영.

200 HU 가드는 기존 multi-col filter (`paragraph_layout.rs:762`) 와 동일 임계값 — 휴리스틱이 아닌 페이지네이션 노이즈 제거.

### 3.2 `src/renderer/layout/integration_tests.rs`

`test_489_picture_square_wrap_text_does_not_overlap_image` 추가. `samples/exam_science.hwp` 페이지 1 SVG 에서 그림(width=150 height≈136) 의 가로/세로 영역 안에 있는 텍스트가 0 건임을 검증.

### 3.3 `src/main.rs` (보조)

`dump` 명령의 도형 위치 출력에 `horz_align` / `vert_align` 추가. 향후 디버깅 효율 개선.

## 4. 검증 결과

### 4.1 단위 테스트

| 테스트 | 결과 |
|--------|------|
| `cargo test --lib --release` | **1093 passed; 0 failed** (1092 + 신규 1) |
| `cargo test --release --test svg_snapshot` | **6/6 passed** |
| `cargo test --release --test issue_418` | **1/1 passed** |

### 4.2 광범위 byte 비교 회귀 점검 (9 종 샘플 / 263 페이지)

| 샘플 | 페이지 | 동일 | 차이 |
|------|------|------|------|
| exam_kor.hwp | 20 | 20 | 0 |
| **exam_science.hwp** | 4 | 2 | **2** ✓ |
| exam_social.hwp | 4 | 4 | 0 |
| exam_math.hwp | 20 | 20 | 0 |
| exam_eng.hwp | 8 | 8 | 0 |
| 21_언어_기출_편집가능본.hwp | 15 | 15 | 0 |
| aift.hwp | 77 | 77 | 0 |
| kps-ai.hwp | 80 | 80 | 0 |
| synam-001.hwp | 35 | 35 | 0 |
| **합계** | **263** | **261** | **2** |

**차이 2 페이지 모두 의도된 정정** (다른 회귀 0 건):

- **exam_science p1 pi=21** (이슈 핵심): 그림 11250×10230 HU(150×136 px), wrap=Square horz_align=Right. LINE_SEG cs=0 sw=19592. 첫 줄 텍스트 x=534..944 → x=535..798 으로 좁아짐. 그림(807..957) 과 분리 ✓
- **exam_science p2 pi=37** (동일 패턴 추가 발견): 8번 문제 그림 13296×9240 HU(177×123 px), wrap=Square horz_align=Right. LINE_SEG 첫 6 줄 cs=0 sw=17546. 첫 줄 텍스트 x=147..480 → x=71..292 로 좁아짐. 그림(316..493) 과 분리 ✓

### 4.3 회귀 영역 점검

| 영역 | 결과 | 근거 |
|------|------|------|
| 표 Square wrap (#362/#439/#463) | 회귀 0 | caller 가 col_area 를 wrap_area 로 좁혀 호출 → segment_width ≈ col_area_w_hu → 조건 미발동 |
| Picture TopAndBottom wrap (#409 v2) | 회귀 0 | text_wrap≠Square 조건 |
| TAC Picture/Shape | 회귀 0 | !treat_as_char 조건 |
| 일반 paragraph (그림 없음) | 회귀 0 | has_picture_shape_square_wrap=false |
| Multi-col paragraph filter | 회귀 0 | 별개 로직 |
| 인라인 TAC 컨트롤 (수식/도형) | 회귀 0 | 9 종 샘플 byte 비교에서 차이 0 |

### 4.4 Clippy

본 변경 영역에서 신규 경고/에러 0. (기존 `src/document_core/commands/object_ops.rs:1007, 298` 의 `panicking_unwrap` 2 건은 devel 베이스라인에 이미 존재 — 본 이슈와 무관.)

## 5. 결정 / 발견 사항

### 5.1 LINE_SEG.cs/sw 는 한컴 정답값

HWP 2010/2020 파일이 어울림 그림 호스트 paragraph 의 LINE_SEG 에 그림 너비만큼 좁아진 cs/sw 를 인코딩한다. 새 측정 없이 이 값을 그대로 사용 → 휴리스틱 도입 없음, 한컴 호환성 보장.

### 5.2 cs>0 케이스 (그림이 컬럼 좌측)

본 변경은 cs/sw 둘 다 effective 값으로 적용하므로 이론상 cs>0 케이스도 동작. 단 `text_style.line_x_offset = x - col_area.x` 같은 탭 계산 reference 는 col_area.x 기준 유지 (탭 위치는 컬럼 기준). exam_science p1/p2 모두 cs=0 이므로 본 케이스에서 effective_col_x == col_area.x. cs>0 인 실 샘플이 발견되면 별도 회귀 검증 필요 — 현재 9 종 샘플에 cs>0 케이스 없음(차이 0 confirmation).

### 5.3 추가 정정 (exam_science p2 pi=37)

본 이슈 보고에 명시되지 않은 동일 패턴 1 건 자동 정정. 의도된 정정 범위 안 — 동일 결함의 다른 발현.

### 5.4 부수 효과 (clipPath 너비 감소)

exam_science p2 의 body-clip-13 너비가 946 → 897 px 로 좁아짐. 텍스트 bbox extent 가 좁아진 자연스러운 결과 — 시각 결함 아님.

## 6. 산출물

| 종류 | 경로 |
|------|------|
| 코드 | `src/renderer/layout/paragraph_layout.rs` (3 hunks) |
| 단위 테스트 | `src/renderer/layout/integration_tests.rs` (test_489 추가) |
| 보조 변경 | `src/main.rs` (dump 출력에 horz_align/vert_align) |
| 수행계획서 | `mydocs/plans/task_m100_489.md` |
| 구현계획서 | `mydocs/plans/task_m100_489_impl.md` |
| 단계별 보고 | `mydocs/working/task_m100_489_stage1.md` (Stage 1 + Stage 2 통합) |
| 최종 보고서 | `mydocs/report/task_m100_489_report.md` (본 문서) |

## 7. 잔존 후속 이슈 (Task #488 분리 4 건 중 #489 완료)

- [x] **#489** — Picture wrap 호스트 텍스트 정정 (본 작업)
- [ ] #490 — exam_science p1 3번 표 28/36 셀 중앙정렬
- [ ] #491 — exam_science p1 2번 답안지 위치 미세 차이
- [ ] #492 — exam_science p1 컬럼 2 5번 밑단 짤림 (#489 정정으로 자연 해소 가능성 — 별도 확인 필요)

## 8. 결론

`exam_science.hwp` 페이지 1 5번 문제 본문 가림 결함을 해소. LINE_SEG.cs/sw 한컴 정답값 활용으로 휴리스틱 없이 정정. 9 종 샘플 263 페이지 회귀 점검에서 회귀 0, 의도된 정정 2 건. 동일 패턴인 페이지 2 8번 문제도 자동 정정.
