---
kind: working
status: completed
canonical: mydocs/report/task_m100_4939_report.md
last_verified: 2026-08-16
---

# Task M100 #4939 Stage 5 — 행동 불변·최종 감사

## 1. 종료 판정

Stage 5를 통과했다. W0 baseline과 W1 ledger는 저장된 snapshot과 byte 동일하게 재생성됐고,
`upstream/devel..HEAD` 및 작업트리에서 제품 source 변경은 0개다. fresh native/WASM build 뒤 공개
fixture 7개 167페이지도 전부 byte match였다.

이 판정은 W0·W1 조사 산출물 완료를 뜻한다. 44개 `unknown`의 근거 조사, W2 Font Decision Trace,
W3 이후 coverage·개별 face 개선을 이번 이슈에서 구현했다는 뜻은 아니다.

## 2. 재생성 감사

baseline과 ledger는 임시 파일을 만들지 않고 in-memory canonical JSON으로 재생성해 저장본과 비교했다.

| 산출물 | 저장본 SHA-256 | 재생성 SHA-256 | byte equal |
| --- | --- | --- | --- |
| `font_rule_baseline.json` | `a0fac05c3138471eb3e7404fc701f0053caa6c01a923afae60fd4da64064b466` | 동일 | true |
| `font_rule_ledger.json` | `284afd72259eb0e8465ff6f10da4e6285d792d73dcde5cfa90daa8e4520b8c23` | 동일 | true |

ledger validator 결과는 error 0, 상충 target group 14개 전부 order 설명, self-loop 5개 전부
`knownLimitations` 설명이다.

W0 CLI의 `baseline` subcommand는 historical snapshot의 `sourceCommit`과 현재 Stage 5 HEAD가 다른 경우
의도적으로 중단한다. 따라서 snapshot을 현재 HEAD로 다시 찍지 않고, 같은 exported `buildBaseline()`을
현재 source digest 검증과 함께 호출했다. 이 경로는 source input이 변하면 그대로 실패하며 기존
`sourceCommit`을 변조하지 않는다.

## 3. 제품 source 0-delta

다음 두 범위가 모두 빈 diff였다.

```bash
git diff --name-status upstream/devel..HEAD -- src rhwp-studio/src web
git diff --name-status -- src rhwp-studio/src web
```

- `src/`: 변경 없음
- `rhwp-studio/src/`: 변경 없음
- `web/`: 변경 없음
- `FONT_METRICS`, alias target, fallback order, font asset bytes: 변경 없음
- 미추적 `examples/poc_font_layout_habits.rs`: 접근·stage·수정 없음

## 4. focused gate

| gate | 결과 |
| --- | --- |
| validation HEAD | `487da51cafe9dc3d1abeec01608c9227c6bed4ea` |
| Stage 1~4 Node contract | 25/25 PASS |
| Rust `font_metrics` | 9/9 PASS, 4,038 filtered out |
| Studio font contract | 33/33 PASS |
| frontend font asset | 6/6 PASS |
| ledger validator | error 0 |
| 변경 문서 상대 링크 | 6개 문서, 이상 없음 |
| 변경된 metadata 필수 경로 | 1개 문서, error 0 |

실행 명령은 다음과 같다.

```bash
node --test \
  scripts/tests/font_rule_ledger.test.mjs \
  scripts/tests/font_rule_candidates.test.mjs \
  scripts/tests/font_rule_ledger_evidence.test.mjs
cargo test --profile release-test --lib font_metrics
node --test \
  rhwp-studio/tests/font-substitution.test.ts \
  rhwp-studio/tests/local-fonts.test.ts \
  rhwp-studio/tests/canvaskit-font-plan.test.ts \
  rhwp-studio/tests/canvaskit-sfnt-face.test.ts \
  rhwp-studio/tests/renderer-baseline-font-loading.test.ts
node --test scripts/frontend-font-assets.test.mjs
```

## 5. fresh native/WASM parity

같은 HEAD에서 순차 실행했다.

```bash
cargo build --release
wasm-pack build --target web --out-dir pkg
node scripts/svg_native_wasm_diff.mjs \
  samples/exam_kor.hwp samples/exam_eng.hwp \
  samples/exam_math.hwp samples/exam_science.hwp \
  samples/synam-001.hwp samples/aift.hwp samples/2010-01-06.hwp
```

| 항목 | 결과 |
| --- | --- |
| native / WASM version | `rhwp v0.8.4` / `0.8.4` |
| fixture | 7개 |
| page | 167 |
| match | 7개 문서, 167페이지 |
| mismatch | 0 |
| native binary SHA-256 | `dedabb18064973f7483a71bb8e8a707011bd1b2df379979db19746caa6d88b30` |
| `pkg/rhwp.js` SHA-256 | `20d445f1e9c424a7d72d94bfe17032608bfad4d9a1af2a56275f00b91162cb2c` |
| `pkg/rhwp_bg.wasm` SHA-256 | `ebacf1dc16f13ab26b901fe10082ea958a739f3b31d209a9947d31797b75cbb8` |

세 build digest도 Stage 2와 동일하다. 하네스 원본 `output/` 보고서는 절대 workspace path를 가지므로
Git 산출물에 포함하지 않았다.

전체 `check_document_metadata.py`는 이번 diff 밖의 기존
`mydocs/tech/benchmark_vs_alternatives.md`가 front matter 4개 필드를 갖지 않아 실패한다. 해당 파일은
사용자 변경으로 취급해 수정하지 않았다. 이번에 변경한 metadata 필수 경로인 issue-4939 README는
validator의 `validate_file()`로 분리 검사해 error 0을 확인했다.

## 6. 보안·저작권 경계

- private 10k corpus를 읽거나 재계측하지 않았다.
- 비공개 원문, 식별 파일 목록, 사용자 절대 경로를 산출물에 기록하지 않았다.
- 저장소 밖 font bytes를 복사·수정·재배포하지 않았다.
- 정부상징 비교에는 앞서 허용된 filename과 SHA-256 evidence만 사용했다.

## 7. 후속

FI-01~FI-14 전항 disposition과 Issue #4939 완료 조건은
[최종 결과 보고서](../report/task_m100_4939_report.md)에 기록한다. Stage 5 commit 뒤에도 remote push,
PR 생성, issue comment·close는 별도 승인 전 수행하지 않는다.
