# Task M100 #1378 — 2단계 완료 보고서

## 단계 목표

본문 경로 다중 run 출력 — `render_runs()` 전환 + `write_section` 치환 단순화 +
경계 케이스 단위 테스트, 임시 xfail(`XFAIL_1378_RUN_SPLIT`) 해소 — 구현계획서 2단계.

## 구현 내역

### 1. `src/serializer/hwpx/section.rs` — run 분할 출력

#### RunSplitter (신규)

`char_shapes` 경계(`segs[i].start_pos`, i ≥ 1)를 cut point 로 삼아 문단 콘텐츠를
다중 `<hp:run charPrIDRef>` 로 조립하는 상태 기계.

- `new()`: 연속 동일 id 경계 skip 하며 segs 구축 (규칙 2), 비어있으면 `[(0, 0)]` (규칙 3)
- `cut_before(pos)`: `segs[next].start_pos <= pos` 인 동안 현재 run 을 닫고 다음 run 시작
  — 동일 위치에서 경계 먼저, 해당 위치 콘텐츠는 새 run 소속 (규칙 1)
- `close_run()`: content 가 비면 `<hp:t></hp:t>` 로 방출 — 재파싱 시 run 시작 위치에서
  entry 가 기록되므로 빈 run 을 skip 하면 동일 위치 다른 id entry 가 소실된다 (규칙 5 확정)
- `finish()`: 잔여 경계를 빈 run 으로 방출 후 마지막 run 을 닫는다
- `segs[0].start_pos > 0` 인 비정상 IR 도 첫 run 은 위치 0 부터 (규칙 4, cut 조건이 자연 처리)

#### render_runs() — render_run_content() 대체

run 내부 콘텐츠만 반환하던 구조에서 **완전한 run 시퀀스를 반환**하는 구조로 전환.
호출부의 `<hp:run charPrIDRef>` 수동 감쌈(write_section 추가 문단 루프,
`render_header_footer`, `render_note_sublist`)을 모두 제거했다.

UTF-16 위치 축 위 이벤트별 cut 처리:

| 이벤트 | cut 시점 |
|--------|---------|
| 텍스트 문자 | push 전 `needs_cut(char_pos)` → 버퍼 flush 후 `cut_before(char_pos)` |
| 컨트롤 슬롯 | 방출 전 `cut_before(expected_utf16_pos)` (경계==슬롯 위치 → 슬롯은 새 run) |
| 필드 begin/end | fieldEnd 방출 전 `cut_before(expected)` — 메인 루프 내에서는 expected 비advance (char_pos 동기화가 흡수, 기존 동작 보존), after-loop 잔여분과 trailing slots 는 `cut_before(expected)` 후 `expected += 8` |
| run 경계 | 위 각 지점에서 `cut_before` 가 소비 |

- fast path: 슬롯·필드 없고 단일 run 이면 전체 텍스트를 단일 run 으로 (기존 동작)
- mismatch 경로(slot 수 불일치)도 per-char `cut_before(char_pos)` 적용
- `emit_field_end()` 헬퍼 신규 — 4회 반복되던 fieldEnd `<hp:ctrl>` 포장 블록 공통화
- `first_run_char_shape_id()` 는 `render_runs()` 내부로 흡수, `render_paragraph_parts_for_text` 삭제

#### write_section 치환 단순화 + secPr run id 정비 (구현계획서 1.3)

- 기존 2중 치환(TEXT_SLOT 치환 → anchor 재치환)을 템플릿 텍스트 run 전체
  `<hp:run charPrIDRef="0"><hp:t/></hp:run>` (`TEMPLATE_TEXT_RUN`, 템플릿 내 유일 1회)
  → `render_runs()` 결과 1회 치환으로 단순화. `TEXT_SLOT`·`TEMPLATE_RUN_BEFORE_TEXT` 삭제.
- **양상 ② 해소**: 첫 문단 `char_shapes[0].char_shape_id` (first_cs) 가 0 이 아니면
  템플릿 secPr run 의 `charPrIDRef="0"` 을 first_cs 로 치환 (`TEMPLATE_SECPR_RUN_OPEN`).
  재파싱 시 secPr run 의 `(0, first_cs)` 가 텍스트 run 과 dedup 되어 가짜 `(0,0)` entry 와
  텍스트 run id 소실이 함께 사라진다. first_cs == 0 이면 치환 불필요 (이미 정합).

#### ID 참조 무결성 (구현계획서 1.5)

`render_runs()` 가 문단의 **모든 char_shapes entry** 를 `ctx.char_shape_ids.reference()`
한다 (fallback 0 은 제외 — `Document::default()` 가 char_shapes 0건 등록인 합성 문서와의
정합 유지).

### 2. `tests/hwpx_roundtrip_baseline.rs` — 임시 xfail 해소

37건 **전건 해소** 확인 후 1단계에서 등록한 임시 장치를 모두 제거:

- `XFAIL_1378_RUN_SPLIT` 상수 (37건) 삭제
- `xfail_1378_entries_still_fail` 테스트 삭제
- `run_baseline` 의 임시 skip 블록 삭제 (eligible 가드는 원형 유지)

## 경계 케이스 단위 테스트 (구현계획서 1.2 — 11건, `task1378_*`)

| # | 테스트 | 고정하는 규칙 |
|---|--------|--------------|
| 1 | `two_run_split_mid_text` | 텍스트 중간 경계 분할 (출력 정확 문자열 비교) |
| 2 | `boundary_at_slot_position_slot_in_new_run` | 규칙 1 — 경계==슬롯 위치, 슬롯은 새 run 소속 |
| 3 | `boundary_between_slot_and_text` | 슬롯 뒤 텍스트 중간 경계 |
| 4 | `consecutive_same_id_boundary_skipped` | 규칙 2 — 연속 동일 id 경계 skip |
| 5 | `tab_and_linebreak_in_split_runs` | 탭/lineBreak 포함 run 분할 |
| 6 | `empty_paragraph_single_run_id_zero` | 규칙 3 — 빈 문단 단일 run id 0 |
| 7 | `field_end_stays_in_previous_run` | fieldEnd 와 경계 교차 — fieldEnd 는 이전 run |
| 8 | `trailing_boundary_emits_empty_run` | 규칙 5 — 문단 끝 경계는 빈 `<hp:t></hp:t>` run |
| 9 | `trailing_slot_in_new_run` | 문단 끝 슬롯 직전 경계 — 슬롯이 새 run |
| 10 | `section_first_paragraph_secpr_run_id_follows_first_cs` | 양상 ② — secPr run id = first_cs |
| 11 | `serialize_parse_roundtrip_preserves_char_shapes` | full roundtrip char_shapes 일치 (섹션 첫 문단 16 유닛 프리픽스 포함 파서 정합 IR) |

stage1 보고서의 경계 케이스 7종을 모두 포함하며, trailing 경계/슬롯 2종을 추가로 고정했다.

## 임시 xfail 해소 측정

`XFAIL_1378_RUN_SPLIT` 제거 후 `cargo test --test hwpx_roundtrip_baseline` 결과:

- **4 passed / 0 failed** (`baseline_all_samples_roundtrip` + `baseline_large_samples_roundtrip`
  + `xfail_entries_still_fail` + `grade_lists_are_consistent`)
- 1단계 측정 실패 37건(diff 1,483건 — 평탄화 1,438 + 첫 문단 구조 45) → **전건 해소, 잔존 0건**
- 셀·글상자 내부 평탄화가 본문 IrDiff 를 만들지 않음이 함께 확인됨 (게이트 재귀 확장은
  계획대로 3단계)

## 검증 결과

- `cargo test --lib` — 1674 passed / 0 failed (1단계 1663 + 신규 11)
- `cargo test --test hwpx_roundtrip_baseline` — 4 passed / 0 failed (임시 xfail 제거 상태)
- `cargo test --tests` — 123개 바이너리 전부 ok (EXIT=0)
- `cargo fmt --check` — 통과
- `cargo clippy --lib --tests -- -D warnings` — 통과

## 완료 조건 대비

| 구현계획서 2단계 완료 조건 | 결과 |
|---------------------------|------|
| `cargo test --lib` / `--tests` 그린 | 충족 |
| 본문 기인 임시 xfail 0건 | 충족 — 37건 전건 해소, 목록·테스트·skip 블록 제거 |

## 다음 단계

3단계 — 셀·글상자 경로 공유 헬퍼(`table.rs write_sub_list`·`shape.rs
write_draw_text_paragraph` + ctx 시그니처 정비) + `diff_documents` char_shapes 비교의
셀·글상자·각주/미주 내부 문단 재귀 확장.
