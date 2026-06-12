# Task M100 #1379 2단계 완료 보고서 — 셀 경로 공유 직렬화 전환 + charOverlap 방출

- 구현계획서: `mydocs/plans/task_m100_1379_impl.md` 2단계
- 브랜치: `local/task1379`

## 1. 수행 내역

### 1.1 셀 문단 직렬화를 본문 공유 경로로 전환 (`src/serializer/hwpx/table.rs`)

- `write_sub_list()`의 문단 루프를 자체 텍스트-only 방출(`render_text_runs`)에서
  본문과 동일한 **`render_paragraph_parts()` 공유 경로**로 교체
  - 컨트롤 슬롯(중첩 표 재귀 포함) 방출 + run 분할 + char_shapes 경계 보존
  - lineseg IR 보존(있으면 그대로, 없으면 fallback 합성) 동승
  - `ctx.next_para_id()` 채번 — 재귀 시 내부 문단이 외부보다 먼저 채번되나
    본문 경로와 동일 패턴이며 파서는 id 유일성만 요구 (값 의미 없음)
- `section.rs`의 `render_hp_p_open` / `render_paragraph_parts` 가시성을 `pub(crate)`로 조정

### 1.2 `render_control_slot()` CharOverlap arm 보강 (`src/serializer/hwpx/section.rs`)

1단계 승인 사항. 신규 `render_compose()` 추가:

- `<hp:compose>`는 `<hp:run>` 직접 자식 (hp:ctrl 비포장)
- 속성 순서 `circleType→charSz→composeType→charPrCnt→composeText`,
  자식 `<hp:charPr prIDRef>` (u32::MAX=4294967295 미설정 값 포함 파서 적재값 그대로 방출)
- circleType 역매핑 7종 (0=CHAR … 6=SHAPE_REVERSAL_TIRANGLE — 한컴 원문 오타 보존),
  composeType: expansion 1→OVERLAP / 0→SPREAD
- mel-001 셀 charOverlap 3건 해소 확인 (1.4 측정 표 참조)

### 1.3 테스트 7건 추가 (`table.rs`)

헬퍼 2개: `picture_control()`(bin_data_id=1), `serialize_with_bin()`(BinDataContent id=1 등록 ctx).

| 테스트 | 검증 |
|--------|------|
| `task1379_cell_paragraph_emits_picture_control` | 셀 내 hp:pic 방출, char_count=9 |
| `task1379_nested_table_in_cell_recurses_with_unique_para_ids` | 중첩 hp:tbl 재귀 + para id 유일성 |
| `task1379_cell_control_slot_position_between_text` | 슬롯 위치 a<pic<b, char_offsets [0,9] |
| `task1379_cell_char_shape_boundary_after_control_restored` | 컨트롤 뒤 char_shapes 경계 (8,77) run 분할 |
| `task1379_cell_lineseg_preserved_from_ir` | LineSeg IR 보존 (flags=393216 등) |
| `task1379_cell_char_overlap_emitted_as_compose` | hp:compose 속성/charPr 방출 |
| `task1379_ta_pic_001_r_roundtrip_preserves_cell_pictures` | 이슈 대표 샘플 재파싱 후 셀 pic 보존 |

> **발견**: ta-pic-001-r 원본 section0.xml 실측 hp:pic는 **2개**(전부 셀 내).
> 이슈 본문의 "4개"는 부정확 수치로 판명 — 테스트는 실측 2로 고정하고 주석 명기.

### 1.4 해소 측정 (baseline_check 동등 로직 전수 재실행)

`rhwp hwpx-roundtrip` CLI는 IR_DIFF를 하드 실패로 치지 않아(exit 0) 측정에 부적합 —
baseline_check 동등 임시 테스트로 측정 후 임시 파일은 삭제(커밋 제외).

**XFAIL_1378_RECURSIVE 13건 → 해소 5건**

| 결과 | 샘플 | 잔존 원인 |
|------|------|----------|
| PASS | aift, el-school-001, exam-kor-1p, exam-kor-2p, hcar-001 | — (4단계 승격 대상) |
| FAIL | 143E433F503322BD33 | autoNum — #1382 (예정대로 잔존) |
| FAIL | 2024-2Q, 2025-2Q, hwpx-h-02, exam-kor-3p, exam-kor-4p | **셀 내 colPr 미방출 8유닛 시프트** (신규 발견 ①) |
| FAIL | exam_kor | **SERIALIZE_FAIL borderFillIDRef=31 미등록** (신규 발견 ②) |
| FAIL | k-water-rfp | 글상자(3단계) + field 컨트롤 |

**XFAIL_1379_CONTROLS 16건 → 해소 9건**

| 결과 | 샘플 | 잔존 원인 |
|------|------|----------|
| PASS | 2024-1Q, 2024-연간, [2027] 온새미로, footnote-01, form-002, hwpx-h-01, mel-001, ta-pic-001-r, tb-img-03 | — (4단계 승격 대상) |
| FAIL | 2025-1Q, footnote-tbox-01, hwpx-h-03, hy-001, hy-002 | 글상자 경로 — 3단계 해소 대상 (계획대로) |
| FAIL | exam_social-p1, issue_1133 | **SERIALIZE_FAIL borderFillIDRef 미등록 (27/17)** (신규 발견 ②) |

xfail 승격은 구현계획서대로 4단계 일괄 — 현재 `xfail_*_still_fail` 가드가
해소분에서 red인 상태는 2~3단계 사이 계획상 허용 (완료 조건은 `cargo test --lib` 그린).

## 2. 신규 발견 (승인 요청 대상)

### ① 셀 내 colPr(ColumnDef) 미방출 — 8유닛 시프트 5샘플

- 원본 XML이 셀 subList 문단 선두에 `<hp:ctrl><hp:colPr …/></hp:ctrl>`를 가짐
  (예: exam-kor-3p `colPr type="NEWSPAPER" colCount="1"` — 셀 내 다단 정의)
- 파서(parser/hwpx/section.rs:3773)는 `Control::ColumnDef` + 슬롯 8유닛으로 적재하나,
  serializer는 `is_hwpx_inline_slot` 제외 + `render_control_slot` arm 부재로 미방출
  → char_count/char_shapes 경계 8유닛 시프트
- **본문 경로는 의도적으로 인라인 colPr 미방출** (colPr는 섹션 템플릿 첫 run에서 처리,
  mod.rs:349 테스트가 이 동작을 고정). 셀 내 colPr는 섹션 템플릿이 받아줄 수 없으므로
  본문과 동일 취급이 불가 — **셀 경로 한정 방출**이 필요
- 제안: **3단계 범위에 포함** — ColumnDef를 셀/글상자 subList 경로에서만 인라인 방출
  (본문 경로 동작 불변). 해소 시 1378 잔존 5샘플 일괄 해소 예상

### ② borderFillIDRef 미등록 계열 — 3샘플 표면화

- 셀 내 중첩 표/도형 방출이 시작되면서 exam_kor(31)·exam_social-p1(27)·issue_1133(17)이
  `SERIALIZE_FAIL 미등록 ID 참조`로 표면화
- 기존 XFAIL exam_social(31)과 동일 결함 계열 — parser→serializer ID 매핑 경계 문제로
  본 타스크(컨트롤 보존)와 별개 축
- 제안: **별도 이슈 분리** — #1315 하위로 신규 이슈 등록 후 본 타스크에서는 xfail 사유 갱신

### ③ ta-pic-001-r 실측 hp:pic 2개

- 이슈 #1379 본문 "hp:pic 4개 소실"은 부정확 — 원본 실측 2개(전부 셀 내), 전부 보존 확인
- 최종 보고서에서 정정 명기 예정

## 3. 검증

- `cargo test --lib` 1697 passed (table 테스트 26 passed 포함)
- `cargo fmt -- --check` (수정 파일 2건) / `cargo clippy --lib --tests -- -D warnings` 통과

## 4. 변경 파일

| 파일 | 변경 |
|------|------|
| `src/serializer/hwpx/table.rs` | write_sub_list 공유 경로 전환 + 테스트 7건 + 헬퍼 2개 |
| `src/serializer/hwpx/section.rs` | render_hp_p_open/render_paragraph_parts pub(crate) + CharOverlap arm + render_compose |

## 5. 승인 요청 사항

1. 신규 발견 ① — 셀 내 colPr 인라인 방출을 **3단계 범위에 포함** (본문 경로 불변)
2. 신규 발견 ② — borderFillIDRef 미등록 계열 **별도 이슈 분리** (#1315 하위)
3. 3단계(글상자 경로 전환 + rect/drawText 보존) 착수
