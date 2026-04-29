# PR #447 + PR #448 검토 — deleteParagraph + insertParagraph WASM API 추가

본 두 PR 은 같은 작성자 (@oksure) + 같은 영역 (WASM API 추가) + 같은 작업 패턴 (`merge_paragraph_native` / `split_paragraph_native` 와 동일한 구조) 으로 **하나의 사이클로 처리** 함이 효율적이라 통합 검토.

## PR 정보

| 항목 | PR #447 | PR #448 |
|------|--------|--------|
| 제목 | feat: deleteParagraph WASM API 추가 | feat: insertParagraph WASM API 추가 |
| 이슈 | [#271](https://github.com/edwardkim/rhwp/issues/271) (closes) | [#269](https://github.com/edwardkim/rhwp/issues/269) (closes) |
| 규모 | +76 / -0, 3 files | +64 / -0, 3 files |
| 작성자 | @oksure (본 사이클 7번째 PR) | @oksure (본 사이클 8번째 PR) |
| 검토 일자 | 2026-04-29 | 2026-04-29 |

## 본질

| API | 시그니처 | 패턴 |
|-----|---------|------|
| `deleteParagraph(section, paraIdx)` | → `{ ok, removedCharCount, newParagraphCount }` | `merge_paragraph_native` 동일 (구역 마지막 문단은 삭제 거부) |
| `insertParagraph(section, paraIdx)` | → `{ ok, paraIdx, newParagraphCount }` | `split_paragraph_native` 동일 (`paraIdx == count` 시 append) |

### `splitParagraph` 와의 차이 (PR #448 본문)

| | splitParagraph | insertParagraph |
|--|---|---|
| 입력 | 기존 문단 + char offset | 삽입 위치 (0..=count) |
| 결과 | 기존 문단 분리 | 새 빈 문단 추가 |
| append | 불가 | `paraIdx == count` 로 가능 |

## 변경 영역

두 PR 모두 같은 3 파일에 추가:

- `src/document_core/commands/text_editing.rs`: `delete_paragraph_native` + `insert_paragraph_native` (각 67/64 lines)
- `src/model/event.rs`: `ParagraphDeleted` + `ParagraphInserted` enum variants
- `src/wasm_api.rs`: `deleteParagraph` + `insertParagraph` WASM 바인딩

## 처리 — cherry-pick 충돌 정황 + 수동 해결

### 정황

두 PR 이 같은 파일의 같은 영역 (text_editing.rs 끝, event.rs, wasm_api.rs 인접 라인) 에 추가만 하는 정황. 작성자가 두 PR 을 서로 모르고 같은 base (devel) 위에 작성 → 둘 다 cherry-pick 시 충돌.

해결: **두 변경이 추가만이라 양쪽 모두 보존**. 자동 해결 (regex pattern) 시도 → entire 함수가 잘못 병합되어 abort. **수동 해결** 진행:
- event.rs: 두 enum variant + match arm 모두 보존
- text_editing.rs: 두 함수 (delete + insert) 분리 보존
- wasm_api.rs: 두 WASM 바인딩 모두 보존

## dry-run cherry-pick 결과

`local/pr447-448` 브랜치 (`local/devel` 분기) — 작성자 attribution 보존:

| commit (cherry-pick) | 작성자 | 내용 |
|---------------------|--------|------|
| `7685c47` (← `352f0ee`) | @oksure | feat: deleteParagraph WASM API 추가 (#271) |
| `f705efb` (← `6d2a6d8`) | @oksure | feat: insertParagraph WASM API 추가 (#269) |

PR #448 cherry-pick 시 충돌 → 수동 해결 → `--continue`.

## 검증 (cherry-pick 후)

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ **1069 passed** (회귀 0) |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --test issue_418` | ✅ 1/1 (Task #418 보존) |
| `cargo clippy --lib -- -D warnings` | ✅ warning 0건 |
| WASM 빌드 (Docker) | ✅ 1m 20s |

## 시각 판정 정황

본 두 PR 은 **WASM API 추가** 패턴 — 화면 변화 없음. PR #405, #411, #419, #444 와 동일 패턴으로 시각 판정 불필요.

→ **시각 판정 불필요** (API 노출만, Rust core 알고리즘 변경 없음).

## 두 PR 의 좋은 점

1. **기존 패턴 정확 모방**: `merge_paragraph_native` / `split_paragraph_native` 와 동일 구조 (reflow + recalculate_vpos + recompose + paginate + 다단 수렴)
2. **범위 검사 + 제약 명시**:
   - `deleteParagraph`: 구역 마지막 문단 삭제 거부 (HWP 규약: 구역당 최소 1 문단)
   - `insertParagraph`: `paraIdx == paragraphCount` 시 append (PR 본문에 splitParagraph 와의 차이 명시)
3. **이벤트 발행**: `ParagraphDeleted` / `ParagraphInserted` 이벤트로 다운스트림 (event_log) 추적 가능
4. **사용자 시나리오 직접 해결**:
   - PR #447: 빈 문단 구조 누적 방지 (replaceAll 폴백 회피)
   - PR #448: append 시 "insert before last" → 문단 순서 뒤바뀜 회피
5. **변경 범위 한정**: 각 PR 매우 가벼움 (+76, +64 라인)

## 메모리 원칙 부합

| 메모리 | 부합 여부 |
|--------|----------|
| 외부 PR 머지 전 검증 게이트 | ✅ cargo test 1069 + svg_snapshot 6/6 + clippy 0 + WASM |
| 시각 판정 게이트 | (해당 없음 — API 추가, 화면 변화 없음) |
| 작은 단위 PATCH 회전 | ✅ 두 PR 모두 매우 가벼운 API 추가 |
| 보고서는 타스크 브랜치에서 커밋 | ✅ `local/pr447-448` 에서 커밋 |

## 다음 단계

1. 본 통합 보고서 + 오늘할일 갱신 commit
2. `local/pr447-448` → `local/devel` → `devel` 머지 + push
3. PR #447, #448 close + 작성자 댓글 (이슈 #271, #269 자동 close)

## 참고

- PR: [#447](https://github.com/edwardkim/rhwp/pull/447), [#448](https://github.com/edwardkim/rhwp/pull/448)
- 이슈: [#271](https://github.com/edwardkim/rhwp/issues/271), [#269](https://github.com/edwardkim/rhwp/issues/269)
- 같은 작성자 머지 PR (본 사이클): [#395](https://github.com/edwardkim/rhwp/pull/395), [#396](https://github.com/edwardkim/rhwp/pull/396), [#427](https://github.com/edwardkim/rhwp/pull/427), [#444](https://github.com/edwardkim/rhwp/pull/444), [#446](https://github.com/edwardkim/rhwp/pull/446)
