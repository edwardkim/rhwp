---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-08
---

# Task #3820 Stage 57 — 최신 devel 리베이스 후 PR 게이트

## 목적

Stage 56에서 바로잡은 issue2007 p11-p13 제목 소유권을 포함한 전체 변경을 최신
`upstream/devel` 위의 정확한 PR 후보로 확정한다. 이 단계는 로컬 검증과 PR 본문 준비까지
수행하며, 원격 push와 PR 생성은 사용자 승인 뒤에 수행한다.

## 리베이스

- 이전 기준: `d9c530ee8ed4bd0830ff35bc47e552bb0f32274f`
- 최신 기준: `5a4f26d0d0a4e2fc96f4b73510d2aecdad916722`
- upstream 신규: 12 commits
- 작업 커밋: 76 commits
- 결과: 충돌 없이 완료, ahead 76 / behind 0 / clean

upstream 신규 변경은 오라클 보고서와 도구 문서 정합 보정이며 renderer 파일과 직접
겹치지 않았다. 그래도 검증은 리베이스된 정확한 HEAD에서 다시 수행한다.

## 검증 순서

1. 전용 `CARGO_TARGET_DIR=target/task-3820-stage57-exact-head`에서 release-test 바이너리 빌드
2. issue2007 focused 15개 회귀와 p11-p13 144dpi PDF 재대조
3. `cargo build --release`
4. `cargo test --release --lib`
5. `cargo test --profile release-test --tests` — 최종 summary까지 대기
6. Native Skia 공식 회귀 3종
7. `cargo fmt --all -- --check`, `git diff --check`
8. `cargo clippy --all-targets -- -D warnings`
9. `cargo test --doc`
10. `wasm-pack build --target web --out-dir pkg`

전체 integration은 장시간 걸리는 것이 정상이며 중간 무출력만으로 종료하지 않는다.
다른 작업의 빌드 산출물은 지우지 않고 이 단계 전용 target만 사용한다.

## 완료 조건

- issue2007: 17쪽, p11 `[168,223)`, p12 `[223,271)`, p13 `[271,282)` 유지
- p11에는 exact 제목 `중앙선거관리위원회`가 없고 p12에는 존재
- focused 및 전체 회귀 실패 0
- fmt, diff-check, Clippy, rustdoc, Native Skia, WASM gate 통과
- 최종 SHA와 명령별 결과를 본 문서와 PR 본문 초안에 반영

## 검증 대상

- 기준: `upstream/devel` `5a4f26d0d0a4e2fc96f4b73510d2aecdad916722`
- 코드 검증 HEAD: `c4d0ae5547b7e402f22cc7b1f4e84f2caa52ab13`
- release-test 바이너리 SHA-256:
  `3460d593d12d15502f4b7a01a3115e7be77ff7e7d56de9ccdd4e3d1f22427d5d`
- 전용 target: `target/task-3820-stage57-exact-head`

이 문서를 완료 상태로 바꾸는 후속 커밋은 문서와 검증 증적만 추가하며 실행 코드에는 손대지
않는다. 따라서 아래 cargo·Skia·WASM 결과의 코드 대상은 위 HEAD로 고정한다.

## focused PDF 재검증

`issue2007_nested_cell_pagination_42065.hwp`를 한컴 2020 기준 PDF와 144dpi로 다시 대조했다.

- 전체 export: rhwp 17쪽 / 기준 PDF 17쪽
- 요청·완료·누락: 3 / 3 / 0 (p11-p13)
- SVG / render tree: 17 / 17
- compare / overlay / review: 3 / 3 / 3
- visual sweep 자동 flag: 0쪽
- focused integration: 15 passed / 0 failed
- page cut: p11 `[168,223)`, p12 `[223,271)`, p13 `[271,282)`
- p11은 `국세기본법`의 마지막 문장으로 끝나며 exact 제목 `3 중앙선거관리위원회`가 없다.
- p12가 해당 제목과 다음 표를 소유하고 p13 이후 경계도 유지된다.

pixel·ink 점수에는 기존 글꼴·rasterization 차이가 크게 남으므로 자동 flag 0을 전체 시각
무결함으로 확대하지 않는다. 이번 판정은 제목의 exact `TextRun` owner, page cut, PDF 직접
review를 함께 사용했다.

증적:

- [p11-p13 exact-head review](../pr/assets/task_m100_3820_stage57_exact_head_pr_gate/review_p011_p013_exact_head.png)
- [visual sweep 원장](../pr/assets/task_m100_3820_stage57_exact_head_pr_gate/visual_sweep_summary_exact_head.json)
- [overlay 지표](../pr/assets/task_m100_3820_stage57_exact_head_pr_gate/overlay_metrics_exact_head.json)
- [provenance](../pr/assets/task_m100_3820_stage57_exact_head_pr_gate/provenance.tsv)

## 최종 게이트 결과

모든 cargo 명령은 `CARGO_INCREMENTAL=0`과 위 전용 target을 사용해 순차 실행했다.

| 게이트 | 결과 |
| --- | --- |
| `cargo build --profile release-test` | 통과 |
| issue2007 focused integration | 15 passed / 0 failed |
| `cargo build --release` | 통과 |
| `cargo test --release --lib` | 3316 passed / 0 failed / 10 ignored |
| `cargo test --profile release-test --tests` | exit 0, 전체 integration 통과 |
| `overflow_cell_baseline` | 1 passed / 0 failed, 72.19초 |
| Native Skia `skia --lib` | 58 passed / 0 failed |
| Native Skia missing-picture | 2 passed / 0 failed |
| Native Skia direct PDF | 4 passed / 0 failed |
| `cargo fmt --all -- --check` | 통과 |
| `git diff --check` | 통과 |
| `cargo clippy --all-targets -- -D warnings` | 통과, warning 0 |
| `cargo test --doc` | 4 passed / 0 failed / 2 ignored |
| `(cd rhwp-studio && npx tsc --noEmit)` | 통과 |
| `npm --prefix rhwp-studio test` | 802 passed / 0 failed |
| `wasm-pack build --target web --out-dir pkg` | 통과, `wasm-opt`·패키징 완료 |

과거 증가가 발생했던 overflow-cell 스윕과 issue2007 회귀가 모두 전체 integration 안에서도
통과했다. 이 결과로 최신 devel 기준 코드·focused PDF·정적 검사·native Skia·Studio·WASM
게이트를 완료했다.
