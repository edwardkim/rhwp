---
kind: report
status: active
canonical: mydocs/plans/archives/task_m100_4966.md
last_verified: 2026-08-23
---

# Task M100 #4966 — Stage W7-R4 교차 단계·전체 재검증

## 1. 판정

원격 PR head `745660467`을 부모로 포함한 정정 후보 `fc2194b2c`에서 W1·W2·W3·W6·W7 계약과
renderer 전체 gate를 다시 실행했다. 제품 source 내부의 신규 test oracle은 남지 않았고,
`tests/cases/issue_4966_font_rule_projection.rs` 원본만 review worktree의 generated harness로 자동
배정됐다. 전체 release-test, Native Skia, Docker optimized WASM과 공개 native/WASM parity는 모두
통과했다.

이 결과는 로컬 후보 검증 완료 판정이다. 원격 push와 PR 본문 정정, 최신 GitHub CI와 self-review는
각각 후속 승인 게이트로 남긴다.

## 2. 제출·회귀 경계

| 항목 | 결과 |
| --- | --- |
| PR-base unit-tier | 4,221 tests / 299 modules / cfg support items 28, drift 없음 |
| review manifest | 891 sources / 4,158 static test attrs / 32 suites + 9 exceptions |
| 새 integration 원본 | `regression_suite_013`, 2/2 |
| generated 제출물 | `tests/generated/**`, manifest, Cargo target 변경 없음 |
| W1·W2·W3·W6·W7 Node contract | 87/87 |
| source Rust focused | 35/35 |
| Clippy | `--all-targets -- -D warnings` 통과 |
| fmt | prepared review worktree에서 `cargo fmt --all`과 `--check` 통과 |

주 source checkout에서 `cargo fmt --all`을 실행하면 PR에 포함하지 않는
`tests/generated/regression_suite_001.rs`~`032.rs`를 Cargo target이 열려고 해 파일 부재로 중단됐다.
CONTRIBUTING의 원본-only 제출 계약을 어겨 주 checkout에 파생물을 만들지 않고, manifest가 준비된 동일
SHA의 review worktree에서 필수 fmt를 통과했다. 이는 코드 포맷 실패가 아니라 source 제출 절차와 Cargo
generated target registry 사이의 운영 마찰이며 후속 운영 개선 후보로 남긴다.

## 3. 전체 Rust·Native Skia

| gate | 결과 |
| --- | --- |
| release build | 통과, 9분 55초 |
| release library | 4,071 pass / 13 ignore |
| release-test nextest | 8,200/8,200 pass / 41 skip / slow 5 |
| Native Skia library | 4,128 pass / 13 ignore |
| missing picture placeholder | 2/2 |
| direct PDF export | 4/4 |
| rustdoc | 8 pass / 3 ignore |

설치된 nextest `0.9.137`이 저장소 권고 `0.9.140`보다 낮다는 비차단 경고가 있었지만, 실행된 8,200건은
실패 없이 끝났다. W7-6 대비 release library 3건과 nextest 1건이 줄어든 것은 금지된 source-side 회귀
3개를 제거하고 공개 API integration 2개로 재구성한 제출 경계 변화다.

## 4. Docker WASM·공개 parity

| gate | 결과 |
| --- | --- |
| Docker 29.7.2 optimized WASM | 통과, 6분 22초 |
| fresh WASM Decision Trace E2E | 3/3 |
| W1 공개 HWP | 7문서 / 167페이지 / mismatch 0 |
| W2 대표 HWP/HWPX | 6문서 / page 0 6개 / mismatch 0 |

Decision Trace E2E의 최초 실행은 분리 worktree에 `rhwp-studio/node_modules`가 없어
`@noble/hashes`를 해석하지 못했다. 제품 실패로 분류하지 않고 기존 설치 디렉터리를 임시 symlink로
연결한 뒤 같은 fresh Docker `pkg`에서 3/3을 통과했다.

| 산출물 | SHA-256 |
| --- | --- |
| native `rhwp` | `9b7756b75db6c8411f080c3b20dc7af5e105138a0b7917d3323cdb0423d34ca7` |
| `pkg/rhwp.js` | `cc92b88840c9c21e52504232668c62fedd799b9f59d7ba561264a6debb890efb` |
| `pkg/rhwp_bg.wasm` | `5187b0708f86c5b0ebe85703f3f33dfc30e198d3a9048cfe10fd57f086f5fcd1` |

검증에는 공개 sample만 사용했다. parity 원문은 review worktree의 절대 경로를 포함하므로 제출하지 않고
집계만 기록한다.

## 5. 후속 절차

1. 동일 SHA의 prepared review worktree에서 fmt·unit-tier를, 주 checkout에서 문서 링크·diff 검사를 통과한다.
2. W7-R4 문서 정정을 trailing local commit으로 고정한다.
3. 메인테이너 승인 뒤 code·문서 후보를 원격 PR branch에 push한다.
4. PR 본문의 integration source, 4,221 unit tests, 87/87 Node contract와 최신 전체 gate 수치를 정정한다.
5. 최신 GitHub CI 성공 전에는 self-review로 진입하지 않는다.
