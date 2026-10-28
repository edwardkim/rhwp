# Task M100 #1380 2단계 완료 보고서 — 정정 일괄 (파서 주입 제거 + serializer 방출 생략)

- 구현계획서: `mydocs/plans/task_m100_1380_impl.md` 2단계 (1단계 보고서 6절 조정 범위 승인분)
- 브랜치: `local/task1380`

## 1. 구현 내용

### 1.1 파서 — zero-default 주입 제거 (`parser/hwpx/section.rs`)

원본에 `<hp:linesegarray>` 가 없는 문단에 zero-default LineSeg 1개를 합성 주입하던
블록(구 729행)을 제거. 이제 IR `line_segs` 를 **빈 채 유지**한다. HWP5 파서가 빈 벡터를
유지하는 선례와 정합.

### 1.2 serializer — 빈 IR 문단의 linesegarray 방출 생략 (`serializer/hwpx/`)

| 변경 | 내용 |
|------|------|
| `render_paragraph_parts` (section.rs) | 반환값을 `<hp:linesegarray>` **요소 전체**(또는 빈 문자열)로 변경. `line_segs` 비면 요소 자체를 방출 생략 |
| 호출부 5곳 | 본문(extra)·머리말/꼬리말·각주/미주(section.rs), 셀(table.rs), 글상자(shape.rs)의 수동 `<hp:linesegarray>` 래핑 제거 — 생략 판단을 단일 지점으로 집중 |
| `replace_first_linesegs` | 템플릿 첫 linesegarray 를 내용 치환에서 **요소 전체 치환**으로 변경 — 빈 문자열이면 요소 제거 |
| fallback 합성 제거 | `render_lineseg_array_fallback` / `push_lineseg_static` (vertsize=1000 계열 정적 합성) 및 전용 상수 삭제. 비파싱 IR(`Document::default()`·빈 섹션)도 방출 생략으로 일원화 |

**fallback 완전 제거 근거** (1단계 승인안의 "의존 경로 조사 후 결정" 항목):
H1 샘플의 linesegarray 부재 문단 38·55건은 **텍스트가 있는** 문단이다. 텍스트 유무로
fallback 을 남기면 이 문단들이 fallback 에 걸려 1000 계열을 합성 방출 — 보존이 불가능하다.
원본 HWPX 가 linesegarray 없이 유효(한컴 생산 파일 실재)하고 한컴은 열 때 재계산하므로
방출 생략이 유일한 보존축이다.

### 1.3 document_core — 로드 시 자동 reflow 정합 (`document_core/commands/document.rs`)

의존 경로 조사에서 추가 발견: `needs_line_seg_reflow` 가 종전 주입 패턴
(`len==1 && line_height==0`)에 정확히 의존 — 주입 제거 시 HWPX 로드(에디터·WASM 경로)에서
linesegarray 부재 문단의 자동 합성(`reflow_zero_height_paragraphs`)이 발동하지 않게 된다.

- `needs_line_seg_reflow(para, include_empty)` 로 확장 — `include_empty` 가 참이면 빈
  `line_segs` 도 누락으로 취급
- `from_bytes` 에서 **HWPX 파일에만** `include_empty=true` 전달 (`normalize_hwpx_paragraphs`
  의 기존 HWPX 한정 패턴과 동일)
- on-demand 광역 reflow(`needs_reflow_broadly`)는 `include_empty=false` — #177 고정 동작
  (완전히 빈 programmatic 문단 미개입) 불변

## 2. 구현 중 발견·정정한 회귀 (HWP5 페이지 수)

1.3 을 처음에 형식 무관(`is_empty` 무조건 포함)으로 확장했더니
`cargo test --tests` 에서 **HWP5 회귀 2건 검출**: `hwp3_sample16_hwp5_page_count_64`,
`hwp3_sample16_hwp5_2022_page_count_64` (PR #1009 over-split 재발 방지 게이트).
HWP5 문서에 자연 존재하는 빈 `line_segs` 문단이 신규 reflow 대상이 되어 페이지가
과다 분할된 것. **HWPX 한정 플래그로 정정 후 4건 전부 그린.** 구현계획서의
"HWP5 경로 영향 금지" 제약이 통합 게이트로 실제 방어된 사례.

## 3. 단위 테스트 (구현계획서 2.3 대응)

| 테스트 | 위치 | 고정하는 동작 |
|--------|------|--------------|
| `task1380_no_linesegarray_keeps_line_segs_empty` | parser/hwpx/section.rs | linesegarray 부재 → 주입 없이 빈 채 유지 |
| `task1380_linesegarray_values_loaded_as_is` | 〃 | 존재 시 9필드 그대로 적재 |
| `task1380_linesegarray_omitted_when_ir_empty` | serializer/hwpx/section.rs | 빈 line_segs(텍스트 있어도) → 요소 방출 생략 |
| `task1380_empty_section_omits_linesegarray` | 〃 | 문단 없는 섹션 템플릿에서도 요소 제거 |
| `task1380_no_linesegarray_when_ir_has_none` | serializer/hwpx/mod.rs | 패키지(ZIP) 산출물 수준 원본 무 → RT 무 |
| `linesegs_from_ir_emitted_per_linebreak` (개정) | 〃 | IR 값 그대로 줄 수만큼 방출 (구 fallback 전제 테스트를 IR 기반으로 전환) |
| `task177_fallback_used_when_ir_empty` (대체) | serializer/hwpx/section.rs | 구 fallback 고정 테스트 → 방출 생략 고정으로 대체 |

## 4. 검증

### 4.1 H1 결함 해소 확정 (XML 수준)

```
business_overview: lineseg 원본=0 → RT=0 (구: 0→38) / p 38=38
expense_report   : lineseg 원본=0 → RT=0 (구: 0→55) / p 55=55
```

전수 lineseg 카운트 sweep: 잔존 차이 4건은 전부 기지 귀속 — 1단계 2.2 와 동일
(ta-pic-001-r −1·mel-001 −2·143E… −1 = #1387 캡션, aift −13 = #1387 캡션 13).
**신규 차이 0.**

### 4.2 전수 게이트·측정

```
hwpx-roundtrip --batch → PASS 48 / IR_DIFF 1(#1382) / SERIALIZE_FAIL 4(#1384) / PARSE_FAIL 1
lineseg_diff.tsv → round1=0 round2=0 (1단계와 동일, 드리프트 0)
```

### 4.3 렌더링 영향 0 (SVG 바이트 대조)

| 비교 | 결과 |
|------|------|
| 원본 export-svg 수정 전 vs 후 (business_overview·expense_report) | **md5 동일** |
| RT export-svg 구 코드 RT vs 신 코드 RT | **md5 동일** |

참고(범위 밖, 기존재): RT vs 원본 SVG 에서 셀 배경 rect 1건 소실 관찰 — 구 코드 RT 에도
동일하게 존재하는 serializer 충실도 기지 한계(#1315 계열)로 본 변경과 무관. 텍스트
좌표는 전 항목 동일(시프트 0).

### 4.4 CI급

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | 1714 passed |
| `cargo test --tests` (baseline 게이트 + sample16 페이지 수 게이트 포함) | 전체 그린 |
| `cargo fmt --check` | 통과 |
| `cargo clippy --lib --tests` | 경고 0 |

## 5. 산출물

- 소스: `src/parser/hwpx/section.rs` (주입 제거 + 테스트 2), `src/serializer/hwpx/{section,table,shape,mod}.rs` (방출 생략 + 테스트), `src/document_core/commands/document.rs` (HWPX 한정 include_empty)
- 측정: `output/poc/task1380_s2/` (inventory.tsv, lineseg_diff.tsv, *.rt.hwpx — git 미포함)

## 6. 다음 단계

3단계 (승인분): `diff_linesegs` baseline 게이트 동승 — 주입 제거로 IR 비교가 합성
비대칭(empty vs non-empty)을 검출 가능해진 상태. 현재 전수 0이므로 xfail 0 동승.
