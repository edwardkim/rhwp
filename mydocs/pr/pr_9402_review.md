# PR #9402 리뷰 — 줄 삽입 시 가로 병합 상속

## 최종 판정

**머지 보류.** 원 PR의 Rust 테스트 배치 정책 검사가 실패했다. 새 회귀 검사를 통합 테스트로 옮긴 커밋의 CI와, 기준 행 위에 삽입한 결과를 독립 한컴 출력에서 확인하면 재검토할 수 있다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 검토일 | 2026-11-27 KST |
| 원 PR | [#9402](https://github.com/edwardkim/rhwp/pull/9402), `semanticist21`, `devel` 대상, Open |
| 검토한 원 head | `dce71b1a65009de6b1b6a575ed4c76947d2b9151` |
| 실패한 CI의 PR base | `0fbee2c04ecee63cf526a9a7e61b38b32608bc3c` |
| 최신 `devel` 대조 | `3cf4ceb5f1ebca8612906a7a9324ea82a3f724fc`와 `git merge-tree --write-tree` 통과 |
| 변경과 접수 상태 | 1커밋, 4파일, +149/−16. 관련 이슈와 지정된 리뷰어 없음 |

## 변경과 검토 범위

`Table::insert_row_inheriting`은 기준 행을 덮는 가로 병합의 시작 열과 폭을 셀 이동 전에 기록한다. 새 행에서는 가로 병합 셀을 한 번 생성하며 삽입 위치를 걸치는 세로 병합만 연장한다. 편집 후 새 셀의 너비와 경계선이 달라지는 변경이다.

## 검증 입력과 결과

| 검증 | 원 head | 테스트를 옮긴 로컬 대조 |
| --- | --- | --- |
| 실제 PR base를 사용한 `rust-unit-test-tiers --check` | 실패. 전체 4,093 / 기준 4,092, 표 모듈 81 / 기준 80 | 전체 4,092, 표 모듈 80으로 통과 |
| Rust integration manifest | GitHub에서 48/48 통합 타깃 통과 | `--prepare`와 `--check --base-ref` 통과, 48/48 |
| `cargo fmt --all -- --check` | GitHub 성공 | 로컬 성공 |
| `table_merged_only_column_width` | 새 혼합 병합 검사는 모델 소스에 위치 | 기존 3건과 옮긴 혼합 병합 1건 통과 (`regression_suite_027`) |

`src/model/table/tests.rs`의 기존 서식 상속 검사는 유지했다. 새 검사 하나와 전용 헬퍼 둘을 `tests/cases/table_merged_only_column_width.rs`로 옮겨 대조했다. 5행 4열의 행별 시작 셀 수 `[4, 4, 2, 4, 4]`와 공개 `rebuild_grid()`를 사용했다. 제품 소스와 정책 파일은 원 head를 사용했다.

집중 검사는 `bwrap`에서 OS·사용자 설치 글꼴 경로를 격리하고 `RHWP_FONT_PATH`를 해제한 뒤 `release-test`로 실행했다. `fc-list`에서 보인 파일은 저장소 `ttfs/opensource/`의 `NotoSansKR-ExtraLight.ttf`, `NotoSansKR-Regular.ttf`, `NotoSansSymbols2-Regular.ttf`, `RHWPOldHangul-Regular.otf`, `RHWPPublicBatang-Regular.ttf` 다섯 개다. 전체 Rust 회귀의 로컬 재실행은 이번 CI 오류 대조 범위에서 제외했다.

최초 실패는 Lint 내부 `Validate Rust test suite manifest`의 unit-tier 검사다. 이로 인해 `Build & Test`도 실패했고 정확한 head의 `CI Impact Policy`는 `failure`였다. [Lint 원본 로그](https://github.com/edwardkim/rhwp/actions/runs/45539271682/job/138953694304) · [집계 로그](https://github.com/edwardkim/rhwp/actions/runs/45539271682/job/138964822012)

## 시각 증적과 남은 차이

작성자는 한글 12.30(Mac)에서 아래쪽으로 삽입한 뒤 저장한 HWPX의 `cellSpan`을 확인했다. 위쪽 삽입 시 시작 주소와 가로·세로 병합, 인쇄한 셀 폭·경계선은 독립 출력으로 확인해야 한다. 한컴 저장 HWPX와 Print PDF, 같은 입력의 Native·fresh WASM 쪽별 TSV 및 비교 PNG를 요청했다.

## 기여자 후속 조치

기여자가 수정 사항과 위쪽 삽입 근거를 원 PR에 제출하면 해당 커밋의 필수 CI와 실제 출력으로 재검토한다. [원 PR에 게시한 피드백](https://github.com/edwardkim/rhwp/pull/9402#issuecomment-7204592674)
