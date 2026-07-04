---
대상 PR: #1541, #1544, #1545, #1558, #1559, #1561, #1563, #1566, #1571
컨트리뷰터: @planet6897
처리일: 2026-06-27
처리 방식: 최신 devel 순차 반영 후 GitHub Actions 재검증, admin merge, 이슈 close 확인
최종 devel: 24c1c1d2e59b5575a20525f7172b618eb19bf380
---

# PR #1541-#1571 planet6897 시리즈 처리 보고서

## 1. 처리 결과

@planet6897 이 연속으로 갱신한 외부 PR 중 작업 지시 범위였던 #1541 이후 시리즈 9건을 최신 `devel` 기준으로 순차 처리했다.

| PR | 제목 | merge commit | 관련 이슈 | 후속 처리 |
|----|------|--------------|-----------|-----------|
| #1541 | Task #1512: HWPX 누름틀 필드 고유 ID 보존 | `bf5300a1dd4234373ad8bbcde1868d462d6180e4` | #1512 | 수동 close |
| #1544 | Task #1488: RowBreak 표 셀 내부 빈 오버레이 vpos 리셋 hardbreak 제외 | `161a09d95fcb9299867a8e377804cb46c1177e07` | #1488 | 이미 close |
| #1545 | Task #1472: ParaShape indent IR 정답화 + 미주 페이지네이션 보정 분리 | `498b8a5f8f13853e35e32e82b9f891a129c8196f` | #1472 | 수동 close |
| #1558 | HWP5 roundtrip 무손실 게이트 + BinData 고아 스트림 드롭 수정 | `82e4ac4aed990dfdea966c96c74b9b7973b231de` | #1552, #1554 | 수동 close |
| #1559 | Task #1557: HWPX 저장본 한글 페이지 붕괴 해소 | `944d5f7ade282ebe2bf0ae4294dde387d98b63dd` | #1557 | 수동 close |
| #1561 | Task #1556: 다단락 누름틀 fieldEnd 8유닛 시프트 수정 | `97dc0017cf76a89ea990407fe419d069f9a909d8` | #1556 | 충돌 수동 해결 후 수동 close |
| #1563 | Task #1560: 한글 페이지 충실도 오라클 정식화 | `04a7397523df4c520520ada5f39dd701b27c6554` | #1560 | 수동 close |
| #1566 | Task #1564: opengov 고정 실문서 회귀 말뭉치 + 스냅샷 게이트 | `a66da26d96bd06fb5cc09149dd1432864a3eb994` | #1564 | 이미 close, snapshot 보정 |
| #1571 | Task #1567: HWPX 표 셀 pic 드롭 해소 | `24c1c1d2e59b5575a20525f7172b618eb19bf380` | #1567 | snapshot/visual 보정 후 수동 close |

최종 확인 시점에 위 관련 이슈 10건은 모두 `CLOSED` 상태다.

## 2. 처리 순서와 주요 판단

### 2.1 순차 merge 원칙

각 PR은 선행 PR merge 후 다시 branch update 또는 contributor branch 수동 merge를 수행했다. 기존 통합 cherry-pick 접근은 사용하지 않고, GitHub PR head를 최신 `devel`에 맞춘 뒤 해당 head 기준 remote CI를 재확인했다.

### 2.2 #1561 충돌 처리

`gh pr update-branch 1561` 은 conflict로 실패했다. 별도 worktree에서 contributor branch `pr-task1556` 에 `upstream/devel`을 merge하고 `src/parser/hwpx/section.rs` 충돌을 해결했다.

로컬 확인:

| 검증 | 결과 |
|------|------|
| `cargo fmt --check` | pass |
| `git diff --check` | pass |
| `cargo test --release --lib task1556` | 5 passed |
| `cargo test --release --lib task1512` | 1 passed |

보정 merge commit `cd0df107949717b6b7543fdcc80002f03e54982c` 을 contributor branch에 push한 뒤 remote CI 통과를 확인하고 merge했다.

### 2.3 #1566 snapshot 승격 보정

최신 `devel` 반영 후 #1566 첫 CI에서 `opengov_corpus_snapshot` 이 실패했다. 원인은 앞선 직렬화 정정들이 opengov 말뭉치 결과를 개선하면서 snapshot 승격이 필요한 상태가 된 것이다.

보정:

| id | 변경 |
|----|------|
| `36383351` | `IR_DIFF/1` -> `PASS/0` |
| `36388571` | `IR_DIFF/2` -> `IR_DIFF/1` |
| `36388853` | `IR_DIFF/1` -> `PASS/0` |

로컬 확인:

| 검증 | 결과 |
|------|------|
| `cargo test --profile release-test --test opengov_corpus_snapshot` | 2 passed |
| `git diff --check` | pass |

보정 commit `9330f9949e24b8d1d3b9c25a4e38091b74f28f77` push 후 remote CI를 재실행했고, Build & Test / CodeQL 통과 후 merge했다.

### 2.4 #1571 snapshot + visual XFAIL 승격 보정

#1571은 최신 `devel` 반영 후 두 차례의 정상적인 승격 게이트 실패가 있었다.

1차 보정:

| id | 변경 |
|----|------|
| `36385464` | `IR_DIFF/1` -> `PASS/0` |
| `36387103` | `IR_DIFF/1` -> `PASS/0` |
| `36388571` | `IR_DIFF/1` -> `PASS/0` |

2차 보정:

`visual_roundtrip_baseline` 에서 opengov 3건이 더 이상 XFAIL이 아니므로 `VISUAL_XFAIL` 에서 제거했다.

| sample | 처리 |
|--------|------|
| `opengov/36385464_...hwpx` | baseline 승격 |
| `opengov/36387103_...hwpx` | baseline 승격 |
| `opengov/36388571_...hwpx` | baseline 승격 |

로컬 확인:

| 검증 | 결과 |
|------|------|
| `cargo test --profile release-test --test opengov_corpus_snapshot` | 2 passed |
| `cargo test --profile release-test --test visual_roundtrip_baseline` | 3 passed |
| `git diff --check` | pass |

보정 commits:

- `fda034d99b4503d4012b627cbcd43105f7c0f166` — opengov snapshot 승격
- `66951a12a2c67f4c7257dc7ac58407b87c286794` — visual XFAIL 승격

최종 remote CI에서 Build & Test 15m08s pass, CodeQL/Analyze pass, WASM skipped를 확인하고 merge했다.

## 3. Remote CI 결과

각 PR은 merge 직전 최신 head 기준 GitHub Actions 상태를 확인했다.

| PR | 최종 remote CI |
|----|----------------|
| #1541 | Build & Test pass, CodeQL/Analyze pass, WASM skipped |
| #1544 | Canvas visual diff pass, Build & Test pass, CodeQL/Analyze pass, WASM skipped |
| #1545 | Canvas visual diff pass, Build & Test pass, CodeQL/Analyze pass, WASM skipped |
| #1558 | Canvas visual diff pass, Build & Test pass, CodeQL/Analyze pass, WASM skipped |
| #1559 | Build & Test pass, CodeQL/Analyze pass, WASM skipped |
| #1561 | Build & Test pass, CodeQL/Analyze pass, WASM skipped |
| #1563 | Build & Test pass, CodeQL/Analyze pass, WASM skipped |
| #1566 | Build & Test pass, CodeQL/Analyze pass, WASM skipped |
| #1571 | Build & Test pass, CodeQL/Analyze pass, WASM skipped |

## 4. GitHub 후속 처리

각 PR에는 감사/검증 요약 코멘트를 남겼다. 자동 close되지 않은 이슈는 PR merge 확인 후 수동 close했다.

| PR | PR comment |
|----|------------|
| #1541 | https://github.com/edwardkim/rhwp/pull/1541#issuecomment-4811762289 |
| #1544 | https://github.com/edwardkim/rhwp/pull/1544#issuecomment-4811921497 |
| #1545 | https://github.com/edwardkim/rhwp/pull/1545#issuecomment-4812091729 |
| #1558 | https://github.com/edwardkim/rhwp/pull/1558#issuecomment-4812309914 |
| #1559 | https://github.com/edwardkim/rhwp/pull/1559#issuecomment-4812454502 |
| #1561 | https://github.com/edwardkim/rhwp/pull/1561#issuecomment-4812682933 |
| #1563 | https://github.com/edwardkim/rhwp/pull/1563#issuecomment-4812866887 |
| #1566 | https://github.com/edwardkim/rhwp/pull/1566#issuecomment-4813182892 |
| #1571 | https://github.com/edwardkim/rhwp/pull/1571#issuecomment-4813578995 |

## 5. 잔존 상태

- 이번 지시 범위였던 #1541 이후 planet6897 시리즈 9건은 모두 처리 완료.
- 관련 이슈 #1512, #1488, #1472, #1552, #1554, #1557, #1556, #1560, #1564, #1567 은 모두 close 확인.
- 처리 후 @planet6897 open PR은 #1533, #1538 두 건이 남아 있으나, 이번 지시 범위 밖이다.
- `upstream/devel` 최종 HEAD는 `24c1c1d2e59b5575a20525f7172b618eb19bf380`.

---

작성: 2026-06-27
