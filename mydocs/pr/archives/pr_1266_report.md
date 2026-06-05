# PR #1266 처리 보고서 - HWPX 문단 borderFill NONE side 렌더링 정정

- PR: https://github.com/edwardkim/rhwp/pull/1266
- 관련 이슈: https://github.com/edwardkim/rhwp/issues/1205
- 작성일: 2026-06-03
- 작성자: @postmelee
- 처리 브랜치: `local/pr1266-integration`
- 통합 방식: 최신 `devel` 기준 cherry-pick + 문서 archive 정책 반영 + orders 충돌 수동 해결

## 1. 반영 내용

PR #1266의 핵심 구현을 수용했다.

- HWPX 문단 `borderFill`에서 side별 `type="NONE"`을 실제 비가시 border로 처리
- 4면이 모두 visible이고 동일 stroke인 경우에만 기존 `RectangleNode` stroke 경로 사용
- 일부 side만 visible이거나 stroke가 다른 경우 fill-only rect와 visible side별 `LineNode`로 분해
- partial/cross-column 문단 border 회귀 테스트 유지 및 보강

PR에 포함된 `mydocs/plans`, `mydocs/report`, `mydocs/working` 루트 문서는 현행 archive 정책에 맞춰 각 `archives/` 아래로 이동했다.

## 2. 충돌 해결

GitHub가 표시한 충돌은 `mydocs/orders/20260603.md`에서 발생했다.

해결 방식:

- 현재 `devel`의 2026-06-03 작업 기록을 보존
- PR #1266 / issue #1205 처리 항목만 추가
- 기존 문서 archive 정책과 충돌하지 않도록 PR 문서 위치 정리

## 3. 검증 결과

| 항목 | 결과 |
|---|---|
| `cargo fmt --all --check` | 통과 |
| `git diff --check devel..HEAD` | 통과 |
| `cargo test --lib task_1205 -- --nocapture` | 통과 |
| `cargo test --lib test_469_partial_start_box_does_not_cross_col_top -- --nocapture` | 통과 |
| `cargo test --lib test_471_cross_column_box_no_bottom_line_in_col0 -- --nocapture` | 통과 |
| `cargo test --tests --quiet` | 통과 |
| `cargo clippy --all-targets -- -D warnings` | 통과 |
| `cargo check --target wasm32-unknown-unknown --lib` | 통과 |
| `docker compose --env-file .env.docker run --rm wasm` | 통과 |

`rhwp-studio/public` 번들은 `pkg` 산출물로 재복사했으나, 실제 내용 diff는 발생하지 않았다.

## 4. 시각 판정 준비물

메인테이너 시각 판정용 SVG를 생성했다.

| file | 비고 |
|---|---|
| `output/poc/pr1266-para-border/[2027] 온새미로 1 본교재_010.svg` | `samples/hwpx/[2027] 온새미로 1 본교재.hwpx`, page index 9, debug overlay |

생성 명령:

```text
cargo run --bin rhwp -- export-svg "samples/hwpx/[2027] 온새미로 1 본교재.hwpx" -p 9 -o output/poc/pr1266-para-border --debug-overlay
```

## 5. 남은 절차

1. 메인테이너 SVG / 웹 캔버스 시각 판정
2. 판정 통과 시 `local/pr1266-integration`을 `devel`에 병합
3. `devel`에서 최종 테스트 확인
4. 원격 `devel` push
5. GitHub CI 확인
6. PR #1266 및 issue #1205 종료 처리

## 6. 판정

자동 검증 기준으로는 통합 가능하다.

렌더링 변경 PR이므로 최종 수용 전 메인테이너 시각 판정을 게이트로 둔다.
