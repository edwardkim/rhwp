# Task #296 최종 보고서 — WASM Canvas 경로 inline_tabs 무시 버그 수정

## 이슈

[#296](https://github.com/edwardkim/edwardkim-rhwp/issues/296) — WASM Canvas 렌더 경로가 inline_tabs 무시 (#290 SVG 수정의 Canvas 버전 누락)

## 배경

PR #292 (Task #290, merge commit `085beb0`) 로 네이티브 SVG 경로의 cross-run 탭 감지가 `composed.tab_extended` (inline_tabs) 를 존중하도록 수정되었으나, 브라우저 검증에서 `samples/exam_math.hwp` p.7 #18 "수열" 문항이 여전히 우측으로 밀림:

- **SVG (CLI)**: ✅ `translate(109.80, 162.69)` 정상
- **Canvas (브라우저)**: ❌ x≈290.91 밀림

## 근본 원인 (Stage 1 실측)

`src/renderer/layout/text_measurement.rs` 두 측정기 비대칭:

| 측정기 | inline_tabs 분기 | 증상 |
|--------|------------------|------|
| `EmbeddedTextMeasurer` (네이티브) | 존재하나 `tab_type = ext[2]` 전체 u16 → match 실패로 `_ =>` LEFT 폴백 | 우연히 LEFT 케이스는 동작 |
| `WasmTextMeasurer` (WASM) | **분기 자체 없음** | TabDef만 참조 → `auto_tab_right` 폴스루 → 우측 밀림 |

**ext[2] 포맷** (PR #292 실증): high byte = 탭 종류 enum+1 (1=LEFT, 2=RIGHT, 3=CENTER, 4=DECIMAL), low byte = fill_type.

**실측 데이터** (`RHWP_TAB296=1` 네이티브 로그, 문단 0.144):
- `ext[2] = 256 = 0x0100` → high=1 (LEFT)
- 3개 탭 합계 폭 ≈ 12px → "수열"이 x≈38 에 배치 (PDF 일치)

## 수정 방향

**옵션 A (축소판)**: `WasmTextMeasurer` 에만 inline_tabs 분기 추가. `inline_tab_type(ext)` 헬퍼 = `(ext[2] >> 8) & 0xFF`.

### 범위 축소 사유

초기 설계는 네이티브 `EmbeddedTextMeasurer` 의 match arm 도 `ext[2]` → `inline_tab_type(ext)` 로 재작성하는 것이었으나, Stage 2-3 에서 svg_snapshot 2건 (`issue_147_aift_page3`, `issue_267_ktx_toc_page`) FAIL 발견:
- 기존 golden SVG 가 "우연한 LEFT 폴백" 동작에 의존 중
- 네이티브 측 수정은 한컴 PDF 대조로 올바른 동작 확정 후 별도 이슈 필요

→ Task #296 범위는 **WASM 경로 수정만** 으로 확정. 네이티브 측 일관성 복원은 후속 이슈 후보.

## 변경 내역

### 코드 (2개 파일, +101 -2)

**`src/renderer/layout/text_measurement.rs`** (+69 -2):

1. 헬퍼 `inline_tab_type(ext: &[u16; 7]) -> u8` 신규 추가 (pub(super))
2. `WasmTextMeasurer::estimate_text_width` 에 inline_tabs 분기 신규 (+23줄):
   - `tab_char_idx` 변수 도입
   - `inline_tab_type(ext)` 고바이트 판정 → match `{2 => RIGHT, 3 => CENTER, _ => LEFT}`
   - 기존 custom/default 분기에도 `tab_char_idx += 1` 추가
3. `WasmTextMeasurer::compute_char_positions` 에 동일 분기 신규 (+27줄)
4. `EmbeddedTextMeasurer` 의 두 함수 내 inline_tabs 분기 상단에 주석만 추가 ("Task #296 범위 외, 별도 이슈")

**`src/renderer/layout/tests.rs`** (+32):

- `task296_inline_tab_type_left` — ext[2]=0x0100 (실측 exam_math #18) → 1
- `task296_inline_tab_type_right` — ext[2]=0x0203 (PR #292 저작권\t1) → 2
- `task296_inline_tab_type_center` — ext[2]=0x0300 → 3
- `task296_inline_tab_type_decimal` — ext[2]=0x0400 → 4

## 검증 결과

| 항목 | 결과 |
|------|------|
| `cargo test --lib task296` | ✅ 4 passed |
| `cargo test --lib task290` | ✅ 5 passed (PR #292 회귀 없음) |
| `cargo test --test svg_snapshot` | ✅ 6 passed (기존 golden 전부 유지) |
| `cargo test --test tab_cross_run` | ✅ 1 passed |
| `cargo test --lib` 전체 | ✅ 992 passed / 0 failed / 1 ignored (988 → 992, +4 신규) |
| `cargo clippy --lib -- -D warnings` | ✅ clean |
| `cargo check --target wasm32-unknown-unknown --lib` | ✅ clean |
| WASM Docker 빌드 | ✅ 성공 (pkg/rhwp_bg.wasm 4,089,055 bytes, +3,918) |
| **rhwp-studio 브라우저 시각 검증** | ✅ **작업지시자 검증 성공** — `exam_math.hwp` p.7 #18 좌측 정렬 확인 |

## 후속 이슈 후보

**네이티브 `EmbeddedTextMeasurer` 의 `tab_type = ext[2]` 버그**:
- 기존 golden SVG (issue-147, issue-267) 가 "우연한 LEFT 폴백"에 의존 중
- 수정 시 RIGHT/CENTER inline 탭 경로가 실제 동작 → 한컴 PDF 대조로 올바른 동작 확정 후 별도 이슈 처리 필요
- 현재 SVG 경로에서는 #290 수정 (paragraph_layout.rs 의 cross-run 감지) 이 대부분의 케이스를 커버 중

## 교훈

1. **범위 축소 결정의 가치** — Stage 2 중간에 네이티브 수정이 기존 golden 2건을 깨트리는 것을 확인한 시점에 "범위를 좁혀 WASM 만 고치기" 로 전환. 무리하게 밀어붙여 golden 을 수정했다면 한컴 PDF 대조 없이 잘못된 방향으로 golden 을 갱신할 위험.
2. **두 측정기 간 비대칭 패턴** — #142 → #290 → #296 의 교훈 누적: "같은 데이터를 다른 경로로 계산하는 코드는 헬퍼로 중앙화". 이번에 `inline_tab_type` 을 `pub(super)` 로 공개해두어 네이티브 측정기가 후속 이슈에서 재사용 가능한 상태로 만듦.
3. **진단 로그의 실증 가치** — Stage 1 의 임시 `eprintln!` 로 `ext[2] = 256` 값과 tab_width_px 분포를 숫자로 확정 → 가설이 아닌 실측으로 근본 원인 확정. PR #292 의 `RHWP_TRACE290` 기법 재사용.

## 관련 문서

- 수행계획서: `mydocs/plans/task_m100_296.md`
- 구현계획서: `mydocs/plans/task_m100_296_impl.md`
- 단계 보고서: `mydocs/working/task_m100_296_stage{1,2,3}.md`
- 트러블슈팅: `mydocs/troubleshootings/tab_tac_overlap_142_159.md` (#296 섹션 추가 예정)
- 관련 PR: #292 (#290 SVG 경로 수정)
