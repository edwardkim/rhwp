# PR #1474 리뷰 - ir-diff tab_extended 예약 필드 노이즈 제거

- PR: https://github.com/edwardkim/rhwp/pull/1474
- 제목: `fix(ir-diff): tab_extended 예약 필드[3,4,5] 거짓 차이 제외 (#1473)`
- 작성일: 2026-06-22
- 컨트리뷰터: `oksure` (Hyunwoo Park)
- 관련 이슈: #1473 `ir-diff: tab_extended 예약 필드[3,4,5] 거짓 차이로 실제 parity 차이가 묻힘`
- base/head: `edwardkim/rhwp:devel` <- `oksure/rhwp:contrib/fix-1473-irdiff-tabext-reserved`
- 처리 경로: collaborator-mediated 외부 PR 경로
- 작성 시점 참고값: open, draft 아님, `MERGEABLE` / `CLEAN`, `maintainer_can_modify=true`
- 규모: 1 file, +54 / -1

`mergeable`, `head SHA`, `CI 상태`는 변하는 값이므로 이 문서는 작성 시점 참고값으로만 기록한다.
최종 merge 판단은 PR head 최신 커밋 기준 GitHub Actions 통과와 작업지시자 승인 후 진행한다.

## 1. 요약 판단

PR #1474는 `rhwp ir-diff`가 HWPX와 HWP5의 인라인 탭 확장 배열을 raw 비교하면서,
HWPX가 표현하지 못하는 `tab_extended` 슬롯 차이를 실제 IR parity 차이처럼 보고하던 노이즈를 줄인다.

변경은 `src/main.rs`의 진단 도구 경로에 한정되어 있으며, 파서·직렬화·렌더러 동작은 바꾸지 않는다.
PR 작성자가 Copilot 지적을 반영해 `ext[1]` 처리까지 재검토했고, 최종 비교 대상은 `[0]` width,
`[2]` type/leader pack, `[6]` marker 로 제한되었다.

검토 결과, #1473 범위의 `ir-diff` 노이즈 제거 목적에는 부합한다. 다만 `tab_extended[1]`은 일부
렌더 보조 경로에서 leader/fill 후보로 읽히는 코드가 남아 있으므로, 이 PR은 "HWPX↔HWP5 parity 비교에서
포맷 비대칭 슬롯을 제외하는 진단 도구 변경"으로 범위를 제한해 이해해야 한다.

## 2. 변경 범위

| 파일 | 변경 |
|---|---|
| `src/main.rs` | `tab_ext_semantic_differs()` 추가 |
| `src/main.rs` | `ir_diff`의 `tab_extended` raw 비교를 semantic 비교로 교체 |
| `src/main.rs` | 단위 테스트 2건 추가: 비대칭/예약 필드 무시, 의미 필드 검출 |

커밋:

| SHA | 내용 |
|---|---|
| `fbd6516d` | `fix(ir-diff): tab_extended 예약 필드[3,4,5] 거짓 차이 제외 (#1473)` |
| `a2bf0f9c` | `test(ir-diff): 리뷰 반영 — tab_ext leader/type 인코딩 주석·테스트 정확화 (#1473)` |
| `e7385faf` | `fix(ir-diff): exclude tab_extended[1] from semantic comparison too` |

## 3. 코드 검토

### 3.1 수용 근거

- 변경 지점이 `ir-diff` 출력 판정에만 한정되어 파서/렌더러/직렬화의 기존 보존 동작을 건드리지 않는다.
- 기존 `tab_extended` 개수 비교는 유지하므로 탭 개수 자체의 차이는 계속 잡힌다.
- `tab_ext_semantic_differs()`는 `width`, `type/leader pack`, marker 차이를 계속 검출한다.
- 단위 테스트가 false positive 제거와 의미 필드 차이 검출을 함께 고정한다.
- 대표 샘플에서 PR 본문 주장처럼 `tab_ext` 노이즈가 사라지고 실제 `indent` 차이만 남는 것을 확인했다.

### 3.2 비차단 리스크

- `src/renderer/layout/text_measurement.rs`에는 `tab_extended[tab_idx][1]`을 leader/fill 후보로 읽는 경로가 있다.
  따라서 `[1]`을 "코드 전체에서 의미 없음"으로 일반화하면 부정확하다.
- 다만 이번 변경은 `ir-diff`의 HWPX↔HWP5 parity 비교 노이즈 제거가 목적이고, HWPX 직렬화 경로는
  leader/type을 `ext[2]`에서 읽는다. PR #1474는 렌더러의 inline tab leader 모델을 정리하는 PR이 아니다.
- 향후 HWPX/HWP5 inline tab leader 렌더 정합 이슈가 발견되면 별도 이슈에서 `ext[1]`/`ext[2]` 정규화 모델을
  다뤄야 한다.

위 리스크는 #1473 범위의 도구 출력 노이즈 제거를 막을 정도는 아니지만, PR 리뷰 코멘트에는 범위 제한을
명확히 언급하는 편이 좋다.

## 4. 검증

### 4.1 GitHub Actions

작성 시점 참고값:

| 체크 | 결과 |
|---|---|
| CI / Build & Test | success |
| CodeQL / Analyze (rust) | success |
| CodeQL / Analyze (javascript-typescript) | success |
| CodeQL / Analyze (python) | success |
| Render Diff / Canvas visual diff | success |
| CI / WASM Build | skipped (조건상 skip) |

문서 커밋을 PR head에 추가하면 GitHub Actions가 재실행될 수 있으므로, merge 전 최신 결과를 다시 확인한다.

### 4.2 로컬 검증

검증 기준: PR head `e7385faf`를 `/private/tmp/rhwp-pr1474-nolfs`에 archive 추출하여 실행.
`pdf-large/`는 Git LFS smudge 문제 때문에 제외했으나, 이번 검증에는 필요하지 않다.

| 명령 | 결과 |
|---|---|
| `git diff --check upstream/devel...FETCH_HEAD` | 통과 |
| `cargo test --bin rhwp tab_ext` | 2 passed |
| `cargo fmt --check` | 통과 |
| `cargo clippy --bin rhwp -- -D warnings` | 통과 |
| `cargo run --bin rhwp -- ir-diff 'samples/3-11월_실전_통합_2024-구분선위0미주사이0구분선아래0.hwpx' 'samples/3-11월_실전_통합_2024-구분선위0미주사이0구분선아래0.hwp' --summary --max-lines 30` | `indent` 14건만 남음 |
| `cargo run --bin rhwp -- ir-diff samples/pic2.hwpx samples/pic2.hwp --summary --max-lines 30` | 차이 0건 |
| `cargo run --bin rhwp -- ir-diff samples/table-vpos-01.hwpx samples/table-vpos-01.hwp --summary --max-lines 30` | 차이 0건 |

## 5. 처리 계획

이 PR은 `maintainer_can_modify=true`인 외부 contributor PR 이므로, collaborator-mediated 외부 PR 경로로 처리한다.

1. 본 review 문서, 사전 처리 판단 보고서, PR workflow 매뉴얼 보강 커밋을 PR #1474 head에 추가한다.
2. PR comment 또는 GitHub review 로 검토 결과를 남긴다.
3. 문서 커밋 push 후 GitHub Actions 재실행 결과를 확인한다.
4. 작업지시자 승인 후 merge한다.
5. merge 후 #1473 자동 close 여부를 확인한다.
6. `closes #1473`가 default branch 문제로 자동 close되지 않으면 작업지시자 승인 후 수동 close한다.

## 6. 권고

**수용 / merge 권고.**

변경 범위가 작고 #1473의 진단 도구 노이즈 제거 목적과 일치한다. 로컬 targeted 검증과 대표 샘플 검증도
통과했다. merge 전에는 문서 커밋이 포함된 최신 PR head 기준 GitHub Actions 재통과와 작업지시자 승인을
최종 조건으로 둔다.
