# Task M100 #2214 Stage 3 완료보고서 — scoped cache coherence와 flow 신호 정합

## 0. 판정 요약

- **Stage 판정**: 완료
- **production 변경 범위**: Rust layout cache와 cell text insert result
- **Studio 변경**: 없음. `cellFlowChanged` 소비와 경계 flush는 Stage 4 범위
- **정확성 결과**: HWP/HWPX warm deferred 44자 직후 model/tree가 모두 174이고 exact cursor를 반환
- **성능 결과**: prewarm 44자 direct query가 기존 약 1.96~2.02초에서 약 27.0~27.8ms로 감소
- **페이지네이션 결과**: deferred transient는 115쪽과 cut 37/bounds 945.9를 유지하고,
  explicit flush에서만 cut 38/bounds 971.5로 갱신
- **다음 단계**: Stage 4 pre-cursor 경계 1회 flush와 Studio/WASM 전달 구현 승인 대기

## 1. 구현 내용

### 1.1 단일 nested-text predicate

`LayoutEngine`에 문단 로컬 기여 조건을 단일 helper로 추출했다.

```text
trim-visible text && 같은 문단의 직접 Control::Table 존재
```

표 전체 flag 계산과 편집 전후 판정이 같은 helper를 사용한다. table-wide predicate scan과
test-only scan counter는 기존 cached helper 한 곳에만 남겼다.

### 1.2 조건부 cell-units 무효화

텍스트 삽입 전후의 local contribution과 기존 cached owner flag로 범위를 결정한다.

| 조건 | cell-units 처리 | owner flag 처리 |
|------|-----------------|-----------------|
| local contribution 불변 | edited cell만 제거 | 보존 |
| false→true, cached `true` | edited cell만 제거 | `true` 보존 |
| false→true, cached `false` | owner의 직접 cell 전체 제거 | 재스캔 없이 `true` 갱신 |
| false→true, cache 없음 | edited cell만 제거 | local witness로 `true` 기록 |

unrelated table과 nested table cache는 모든 분기에서 보존한다. cached false→true의 direct
owner-cell key 제거는 O(owner cells) eviction이지만 paragraph/control predicate 재스캔은
아니며, 안정 입력과 cached-true 입력은 O(1)이다.

이 API는 텍스트 **삽입** 전용이다. 삽입은 true→false 변화를 만들지 않는다는 전제를
`debug_assert`로 고정했다. 마지막 witness 삭제처럼 true→false가 가능한 경로는 이 API 범위가
아니다.

### 1.3 `cellFlowChanged` mutation result

삽입 전후 target paragraph의 상대 flow advance를 `i64`로 비교한다.

```text
last.vertical_pos + last.line_height + last.line_spacing - first.vertical_pos
```

deferred insert 반환 JSON은 additive하게 확장됐다. 기존 immediate insert 응답 schema는
`cellFlowChanged` 없이 그대로 유지한다.

```json
{"ok":true,"charOffset":174,"cellFlowChanged":true}
```

처리 순서는 text mutation → target reflow → 후속 paragraph vpos 재계산 → scoped cache
invalidation → section/page-tree invalidation이다. 표 캡션 sentinel `65534`, Shape와 Picture
텍스트에는 table cell-units cache가 없으므로 scoped table 처리를 적용하지 않는다.

독립 리뷰에서 표 캡션은 폭 계산만 sentinel을 처리하고 실제 mutable reflow에서는
`table.cells[65534]`를 찾다가 아무 작업도 하지 않는 기존 누락이 확인됐다. 캡션 paragraph를
직접 reflow하도록 보완하고, 연속 입력이 wrap 경계를 지날 때 실제 상대 advance와
`cellFlowChanged=true`가 일치하는 회귀 테스트를 추가했다.

또한 Shape의 nonzero `cell_idx`는 불변 getter만 거부하고 mutable getter는 허용해, 새 post-edit
검증에서 mutation 후 오류가 될 수 있었다. mutable getter가 `cell_idx==0`을 mutation 전에
검증하도록 맞추고 invalid 호출이 text를 바꾸지 않는 원자성 회귀를 추가했다.

## 2. RED→GREEN 전환

Stage 2의 의도적 RED 6건을 같은 desired assertion의 non-ignored GREEN으로 전환했다.

1. HWP/HWPX warm deferred tree/cursor 공개 동작
2. 실제 stable/44번째 deferred insert의 edited-cell-only eviction
3. 실제 empty nested host false→true의 owner-wide eviction
4. flag 불변 시 sibling/unrelated cache 보존
5. cached true에서 local false→true 시 edited-cell-only eviction
6. cached false→true 시 unrelated cache 보존과 owner flag 직접 갱신

기존 raw `dump_page_items()` 문자열 fingerprint는 영구 GREEN에서 제거했다. crate-internal
`PaginationResult`와 `PageItem::PartialTable`을 사용하는 structured test가 115쪽 cut-chain과
transient/full-flush 상태를 검증한다. #2214 관련 intentional RED나 필수 `#[ignore]`는 남지
않았고, 30-case matrix만 diagnostic 용도로 `#[ignore]`를 유지한다.

## 3. 정확성 결과

### 3.1 warm tree/cursor

| 형식 | model | tree max | path cursor | bounds h | page count |
|------|------:|---------:|-------------|---------:|-----------:|
| HWP | 174 | 174 | `(569.7,341.1)`, exact | 945.9 | 115 |
| HWPX | 174 | 174 | `(569.7,341.1)`, exact | 945.9 | 115 |

기존 tree 129와 첫 줄 fallback은 두 형식 모두 사라졌다.

### 3.2 flow 경계 신호

| 입력 | 상대 advance delta | `cellFlowChanged` |
|------|----------------------:|-------------------|
| batch 28자 | 0HU | `false` |
| 순차 1~43번째 | 각 0HU | 각 `false` |
| 순차 44번째 | +1920HU | `true` |
| 순차 45~50번째 | 각 0HU | 각 `false` |

각 결과의 `charOffset`도 실제 삽입 후 offset과 일치했다.

### 3.3 transient와 full flush 분리

HWP/HWPX 모두 structured cut-chain이 115쪽 전체에서 gap/overlap 없이 이어졌다.

| 상태 | tree max | page 0 end cut | bounds h |
|------|---------:|---------------:|---------:|
| scoped deferred transient | 174 | 37 | 945.9 |
| explicit full flush | 174 | 38 | 971.5 |

scoped eviction은 초기 pagination fragment 115개를 모두 보존했다. full flush에서는 0~114쪽
모든 target fragment가 갱신됐고 새 chain도 연속성을 유지했다. 따라서 Stage 4의 경계 1회
flush는 geometry 정확성에 필요하지만, 일반 입력에서 cursor fallback을 막기 위한 매-key
pagination은 필요하지 않다.

## 4. 성능·회귀 결과

30-case matrix는 결과 기록용 diagnostic으로 최종 production 코드를 한 번 실행했다.

| 형식 | case | Stage 2 | Stage 3 | 결과 |
|------|------|--------:|--------:|------|
| HWP | prewarm seq44 direct | 1960.76ms | 27.78ms | exact, tree 174 |
| HWPX | prewarm seq44 direct | 2016.54ms | 27.00ms | exact, tree 174 |
| HWP | prewarm seq44 path | fallback | 27.28ms | exact, tree 174 |
| HWPX | prewarm seq44 path | fallback | 27.38ms | exact, tree 174 |

direct query 기준 약 70.6배(HWP), 74.7배(HWPX) 단축됐다. 전체 matrix는 78.03초에 완료됐다.
시간은 환경 의존 관찰값이며 hard assertion으로 사용하지 않는다.

cache scope 테스트에서는 stable/flow-boundary 입력 모두 target만 재계산하고 sibling과 owner
flag를 재사용했으며 table predicate scan은 0회였다. false→true branch는 owner 직접 cell을
재계산하고 owner flag를 `Some(true)`로 갱신했으며 nested/unrelated cache와 identity를
보존했다.

## 5. 검증 결과

| 검증 | 결과 |
|------|------|
| `cargo test --profile release-test --test issue_2214_page_local_repaint -- --nocapture` | 2 passed, 0 ignored |
| `cargo test --profile release-test --lib issue2214 -- --nocapture` | 9 passed, 0 ignored |
| `cargo test --profile release-test --test issue_2185_korean_break_unit -- --nocapture` | 1 passed |
| `cargo test --profile release-test --test issue_2063 -- --nocapture` | 1 passed |
| `cargo test --profile release-test --test issue_1949_giant_cell_render_perf -- --nocapture` | 1 passed |
| `cargo test --profile release-test --lib test_insert_text_in_cell -- --nocapture` | 1 passed, immediate 응답에 `cellFlowChanged` 미포함 |
| `cargo test --profile release-test --lib issue2214_deferred_table_caption_reports_flow_change -- --nocapture` | 1 passed, caption wrap boundary 신호 일치 |
| `cargo test --profile release-test --lib issue2214_invalid_shape_cell_index_does_not_mutate_text -- --nocapture` | 1 passed, 오류 전 text 불변 |
| `cargo test --profile release-test --lib text_in_cell -- --nocapture` | 5 passed, 기존 cell edit API 회귀 유지 |
| 30-case HWP/HWPX matrix | diagnostic 1 passed, 78.03s |
| targeted Clippy `-D warnings` | 통과 |
| `cargo fmt --check` | 통과 |
| `git diff --check` | 통과 |

계획서의 기존 `cargo test --lib layout_cache` 필터는 `0 passed; 1 ignored` false-green이므로
`--lib issue2214`로 바로잡았다.

## 6. 변경 파일

| 파일 | 역할 |
|------|------|
| `src/renderer/layout/table_layout.rs` | 단일 predicate와 scoped cache invalidation API |
| `src/document_core/commands/text_editing.rs` | pre/post flow·local contribution, result JSON |
| `src/wasm_api.rs` | deferred API result 계약 문서화 |
| `tests/issue_2214_page_local_repaint.rs` | warm public GREEN과 flow result 계약 |
| `src/wasm_api/tests.rs` | structured 115쪽 transient/full-flush GREEN |
| `mydocs/plans/task_m100_2214_impl.md` | 검증 필터와 no-scan 표현 보정 |
| `mydocs/orders/20260713.md` | Stage 3 완료·Stage 4 승인 대기 상태 |

## 7. Stage 4 입력 계약

Stage 4에서는 Rust 결과를 변경하지 않고 Studio까지 전달·소비한다.

1. `cellFlowChanged=false`: cursor 조회 전 flush 0회, page-local 표시 유지
2. `cellFlowChanged=true`: deferred pending을 먼저 등록하고 cursor 조회 전 full flush 정확히 1회
3. 44번째 경계 뒤 cut 38/bounds 971.5에서 cursor 조회
4. 45~50번째 입력은 추가 flow 경계가 없으므로 누적 flush 수 1회 유지
5. IME/iOS raw insert도 같은 effect를 한 번만 소비

Stage 3에서는 Studio, parser/serializer, font metric, Canvas timer와 paginator를 변경하지 않았다.

## 8. 업스트림 통합 메모

Stage 3 구현·검증 기준은 Stage 2에서 동기화한 `upstream/devel@c7864c62`다. 검증 중 local
`upstream/devel`이 #2183 CI/문서 merge `4f9aaaff`까지 14커밋 전진했다. production/test source
overlap은 없으므로 검증 완료 뒤 Stage 3 commit에 이 변경을 섞지 않는다.

Stage 4 승인 후 구현 전에 최신 upstream을 merge하고 관련 Rust 게이트를 다시 실행한다.
양쪽에 추가된 `mydocs/orders/20260713.md`는 add/add 충돌이 예상되므로 upstream의 #2183/#2233
기록과 이 브랜치의 #2214 행을 모두 보존한다.
