# Task M100 #1378 — 3단계 완료 보고서

## 단계 목표

셀·글상자 경로 공유 헬퍼 적용 + `diff_documents` char_shapes 비교의
셀·글상자·각주/미주 내부 문단 재귀 확장 — 구현계획서 3단계.

## 구현 내역

### 1. `src/serializer/hwpx/section.rs` — 공유 헬퍼 추출

- `split_text_into()` 신규: 기존 mismatch 경로 본체(문자별 `needs_cut` →
  `flush_text_fragment` → `cut_before`)를 함수로 추출. `render_runs()` mismatch
  경로가 그대로 사용.
- `render_text_runs()` 신규 (`pub(crate)`): 텍스트만 char_shapes 경계로 분할하는
  공유 헬퍼 — 셀·글상자 경로용. 컨트롤은 방출하지 않는다 (subList 컨트롤 보존은
  #1379 범위). `char_offsets` 로 문자 idx→UTF-16 위치를 매핑하므로 IR 에 컨트롤
  8 유닛 갭이 있어도 경계 위치가 어긋나지 않는다. 문단의 모든 char_shapes entry
  를 `ctx.char_shape_ids.reference()` 한다 (본문 경로와 동일한 ID 무결성 규칙).
- `render_shape()` 시그니처 `&SerializeContext` → `&mut SerializeContext`
  (글상자 경로 ctx 전파).

### 2. `src/serializer/hwpx/table.rs` — 셀 경로 run 분할

- `write_sub_list()` 의 run 조립을 `render_text_runs()` 결과 raw write 로 대체.
  기존 첫 entry 단일 run + first-only reference 를 제거 — 셀 문단도 다중
  `<hp:run>` 분할 출력.
- `write_cell_text()` 삭제 (대체 완료로 미사용).

### 3. `src/serializer/hwpx/shape.rs` — 글상자 경로 run 분할

- `write_rect()` / `write_draw_text()` / `write_draw_text_paragraph()` 에
  `ctx: &mut SerializeContext` 추가, run 방출을 `render_text_runs()` 로 대체.
- 부수 개선: 기존 단순 escape 출력이 `render_hp_t_content` 기반으로 바뀌어
  글상자 텍스트의 탭(`<hp:tab/>`)·lineBreak(`<hp:lineBreak/>`)가 정상 직렬화된다.

### 4. `src/serializer/hwpx/roundtrip.rs` — 게이트 재귀 확장 (구현계획서 1.4)

- `ParagraphCharShapes` 에 `path: String` 필드 추가 (본문은 `""`), Display 는
  `section[i] paragraph[j]{path} char_shapes: expected=… actual=…`.
- `diff_paragraph_char_shapes()` 신규: 문단 char_shapes 비교 후 컨트롤 쌍을
  zip 인덱스 대응으로 재귀 —
  - Table → `{path}/ctrl[{ci}]tbl.cell[{k}].p[{m}]` (cells flat Vec)
  - Shape → `{path}/ctrl[{ci}]shape` 로 `diff_shape_char_shapes()`
  - Footnote/Endnote → `fn.p[{m}]` / `en.p[{m}]`
- `diff_shape_char_shapes()`: text_box 쌍이면 `.tb.p[{m}]` 재귀, Group 이면
  `.child[{k}]` 재귀. `shape_text_box()` 가 도형 변형별 text_box 접근 공통화.
- 컨트롤 쌍은 **수·타입이 일치하는 인덱스만** 대응 — 컨트롤 보존 소실 검출은
  #1379 범위로 본 게이트 범위 밖.

### 5. `tests/hwpx_roundtrip_baseline.rs` — 임시 xfail 등록

재귀 게이트 측정에서 검출된 범위 밖 실패 13건을 `XFAIL_1378_RECURSIVE`
(상대 경로, 사유) 로 등록 + `run_baseline` skip 블록 +
`xfail_1378_recursive_entries_still_fail` 가드 테스트 (1단계와 동일 패턴 —
후속 이슈에서 해소되어 통과하게 되면 테스트가 실패하므로 승격 누락 방지).

## 신규 단위 테스트 (11건, `task1378_*` 외)

| # | 위치 | 테스트 | 고정하는 동작 |
|---|------|--------|--------------|
| 1 | table.rs | `task1378_cell_paragraph_multi_run_split` | 셀 문단 경계 분할 (정확 문자열) |
| 2 | table.rs | `task1378_cell_boundary_with_control_gap_offsets` | 컨트롤 갭 offsets 에서 경계 정확 |
| 3 | table.rs | `task1378_cell_tab_in_split_runs` | 셀 run 분할 + `<hp:tab/>` |
| 4 | shape.rs | `task1378_drawtext_multi_run_split` | 글상자 문단 경계 분할 |
| 5 | shape.rs | `task1378_drawtext_tab_and_linebreak_rendered` | 글상자 탭/lineBreak 직렬화 |
| 6 | roundtrip.rs | `diff_documents_detects_cell_char_shapes` | 셀 재귀 검출 + path 형식 |
| 7 | roundtrip.rs | `diff_documents_same_cell_char_shapes_is_empty` | 셀 일치 시 무 diff |
| 8 | roundtrip.rs | `diff_documents_detects_textbox_char_shapes` | 글상자 재귀 검출 |
| 9 | roundtrip.rs | `diff_documents_detects_footnote_char_shapes` | 각주 재귀 검출 |
| 10 | roundtrip.rs | `serialize_parse_roundtrip_preserves_cell_char_shapes` | 셀 full roundtrip 보존 |
| 11 | roundtrip.rs | `serialize_parse_roundtrip_preserves_textbox_char_shapes` | 글상자 full roundtrip 보존 |

## 재귀 게이트 측정 (xfail 등록 전)

확장된 `diff_documents` 로 baseline 전수 실행 — **13개 샘플 실패, 총 66건 diff**.
본문 경로 diff 는 0건 (2단계 해소 상태 유지 확인).

| 샘플 | diff | 경로 | 시프트 |
|------|------|------|--------|
| exam_kor.hwpx | 20 | tbl | 8 배수 |
| 2024년 2분기 해외직접투자 보도자료ff.hwpx | 10 | tbl | 8 배수 |
| 2025년 2분기 해외직접투자 (최종).hwpx | 9 | tbl | 8 배수 |
| hwpx-h-02.hwpx | 9 | tbl | 8 배수 |
| k-water-rfp.hwpx | 5 | tbl 4 + shape.tb 1 | 8 배수 |
| aift.hwpx | 2 | tbl | 8 배수 (16 포함) |
| el-school-001.hwpx | 2 | tbl | 8 |
| exam-kor-3p.hwpx | 2 | tbl | 8 |
| exam-kor-4p.hwpx | 2 | tbl | 8 |
| hcar-001.hwpx | 2 | tbl | 8 |
| exam-kor-1p.hwpx | 1 | tbl | 8 |
| exam-kor-2p.hwpx | 1 | tbl | 8 |
| 143E433F503322BD33.hwpx | 1 | fn | **1 유닛** |

## 잔존 실패 분류 (구현계획서 1.6 — 승인 요청 대상)

### 분류 ① — 65건 (tbl 64 + shape.tb 1): subList 컨트롤 미출력, #1379 범위

셀·글상자 subList 는 컨트롤을 직렬화하지 않으므로 (#1379 — subList 컨트롤 보존)
재파싱 시 경계 위치가 8×(경계 앞 컨트롤 수) 유닛 당겨진다
(예: `(8,77)→(0,77)`, `25→17`, `14→6`, `16→0`). **id 시퀀스는 전건 보존** —
경계 위치만 시프트. #1379 에서 컨트롤이 출력되면 함께 해소된다.

### 분류 ② — 1건 (fn): 파서 autoNum 폭 비일관, #1378 범위 밖

`143E433F503322BD33.hwpx` 각주 첫 문단
(`expected=[(0,10),(2,11)] actual=[(0,10),(1,11)]`). 원본 XML 은
run1(id 10) = `<hp:ctrl><hp:autoNum/></hp:ctrl><hp:t> </hp:t>` + run2(id 11) 텍스트.

파서(`src/parser/hwpx/section.rs`)가 autoNum 을 두 축에서 다르게 집계한다:

| 축 | autoNum 폭 | 사용처 |
|----|-----------|--------|
| `calc_utf16_len_from_parts` (4909) | 1 유닛 (`_` 분기) | char_shapes 경계 위치 |
| offsets 조립 루프 (642-648) | 8 유닛 jump | char_offsets·슬롯 추론 |

원본 경계 2 = autoNum(1)+공백(1)은 calc 축 산출. serializer 는 offsets 축
(`[0, 8, 9, …]`)으로 재구성하므로 경계 2 가 placeholder 공백 직후에 cut 되고
autoNum 슬롯은 trailing 으로 밀린다 → 재파싱 `(1,11)`. **1단계 보고서에서
"#1378 범위 밖(파서 비일관)" 으로 기 분류한 원인과 동일**하며, #1379 로도
해소되지 않는 별도 건 (별도 이슈 후보).

두 분류 모두 사유를 명기해 `XFAIL_1378_RECURSIVE` 로 임시 등록했다.
분류와 임시 xfail 처리에 대한 승인을 요청한다.

## 검증 결과

- `cargo test --lib` — 1685 passed / 0 failed (2단계 1674 + 신규 11)
- `cargo test --test hwpx_roundtrip_baseline` — 5 passed / 0 failed
  (xfail 등록 상태, still-fail 가드 포함)
- `cargo test --tests` — 123개 바이너리 전부 ok (EXIT=0)
- `cargo fmt --check` — 통과
- `cargo clippy --lib --tests -- -D warnings` — 통과

## 완료 조건 대비

| 구현계획서 3단계 완료 조건 | 결과 |
|---------------------------|------|
| 셀·글상자 경로 다중 run 출력 | 충족 — render_text_runs 공유 헬퍼 적용 |
| diff_documents 재귀 확장 | 충족 — 셀·글상자(Group 포함)·각주/미주 |
| `cargo test --lib` / `--tests` 그린 | 충족 |
| 잔여 임시 xfail 전부 해소 (범위 밖 분류 승인분 제외) | 13건 범위 밖 분류 — 승인 요청 |

## 다음 단계

4단계 — 전수 검증 + SVG 비교 + 한컴 판정 요청 + 매뉴얼
(`manual/hwpx_roundtrip_baseline.md`) 갱신 + 최종 보고서 + CI급(wasm check 포함).
