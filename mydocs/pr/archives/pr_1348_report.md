# PR #1348 처리 보고서 — 단 구분선 부분 페이지 길이 콘텐츠 높이 복원

## 1. 개요

| 항목 | 내용 |
|---|---|
| PR | #1348 |
| 작성자 | `planet6897` |
| 관련 이슈 | #1347 |
| 검토 브랜치 | `local/pr1348-merge-test` |
| 통합 방식 | 현재 `origin/devel` 기준 PR 단일 커밋 cherry-pick 검증 |
| 원 PR head | `fcaf4998` |
| 검증 커밋 | `1129f6c8` |
| 문서 정리 커밋 | `7665e857` |
| PR close | `2026-06-10T14:30:11Z` |
| Issue #1347 close | `2026-06-10T14:30:15Z` |

## 2. 처리 내용

PR #1348은 #1333에서 맞췄던 단 구분선 콘텐츠 높이 기준이 이후 `4e7f191f`의 full-height 강제 분기로 다시 회귀한 문제를 복원한다.

변경 내용:

- `src/renderer/layout.rs`
  - `prev_zone_sep_full_body` 플래그 제거
  - zone 전환 시 단 구분선 emit에서 `prev_zone_y_end` 직접 전달
  - 마지막 zone 단 구분선 emit에서도 `prev_zone_y_end` 직접 전달
  - body 하단 cap은 `emit_zone_column_separators()`에 유지

기대 동작:

- 콘텐츠가 페이지를 채운 경우: 콘텐츠 높이와 body 높이가 사실상 같으므로 전체 높이에 가깝게 표시
- 부분 페이지인 경우: 콘텐츠 하단까지만 단 구분선 표시

## 3. 검증 결과

GitHub checks:

| 체크 | 결과 |
|---|---|
| Build & Test | pass |
| Canvas visual diff | pass |
| CodeQL | pass |
| Analyze rust | pass |
| Analyze javascript-typescript | pass |
| Analyze python | pass |
| WASM Build | skipped |

로컬 검증:

| 명령 | 결과 |
|---|---|
| `cargo fmt --all -- --check` | 통과 |
| `cargo test --lib` | 통과, 1622 passed / 0 failed / 6 ignored |
| `cargo clippy --lib -- -D warnings` | 통과 |
| p16 SVG export | 통과 |
| p23 SVG export | 통과 |

시각 판정:

| 산출물 | 판정 |
|---|---|
| `output/poc/pr1348-column-separator/3-09월_교육_통합_2024-구분선아래20구분선위20_016.svg` | 통과 |
| `output/poc/pr1348-column-separator/3-09월_교육_통합_2024-구분선아래20구분선위20_023.svg` | 통과 |

23쪽 SVG에서 가운데 단 구분선은 `y=90.706...`에서 `y=600.133...`까지 출력되어, body 전체 하단이 아니라 콘텐츠 하단 기준으로 짧아진 것을 확인했다.

## 4. 판정

**수용 가능**.

수정 범위가 `layout.rs`의 단 구분선 높이 결정 로직에 한정되어 있고, GitHub CI와 로컬 검증이 모두 통과했다. 메인테이너 시각 판정에서도 p16/p23 기준이 통과되었으므로 PR #1348은 수용 절차로 진행 가능하다.

주의점:

- 이번 PR은 메인테이너 커밋 `4e7f191f`의 일반 페이지 full-height 분기를 되돌리는 성격이 있다.
- 하지만 body 하단 cap이 유지되어 overflow/꽉 찬 페이지에서도 과도한 선 출력은 방지된다.
- contributor 작업 문서 중 `mydocs/report/task_m100_1347_report.md`, `mydocs/working/task_m100_1347_stage2.md`가 활성 폴더에 남아 있으므로 통합 전 archive 이동 여부를 결정해야 한다.

## 5. 후속 절차

처리 완료:

- [x] 작업지시자 완료 보고서 승인
- [x] `local/devel`에 PR 커밋 반영 — `1129f6c8`
- [x] contributor 작업 문서 archive 정리 — `7665e857`
- [x] `origin/devel` push — `7665e857`
- [x] PR #1348에 메인테이너 코멘트 작성 — https://github.com/edwardkim/rhwp/pull/1348#issuecomment-4671315729
- [x] PR #1348 close — `2026-06-10T14:30:11Z`
- [x] Issue #1347 completed close — `2026-06-10T14:30:15Z`
