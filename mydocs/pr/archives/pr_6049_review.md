---
kind: pr-review
status: accepted-pending-trailing-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-25
---

# PR #6049 self-review — canonical font rule lifecycle·evidence delta (#5955)

## 접수 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#6049](https://github.com/edwardkim/rhwp/pull/6049) |
| Issue | [#5955](https://github.com/edwardkim/rhwp/issues/5955) |
| 작성자·검토자 | [@edwardkim](https://github.com/edwardkim) maintainer self-review |
| base | `devel@385e93b2c317d1f50d874fd655e88cf4b2a1ba07` |
| code candidate | `b1afde931420e497a05b0e29eb26c8e1eefa77d6` |
| 상태 | Draft, `MERGEABLE/CLEAN`, reviewer 미지정 |
| 규모 | 47 files, +57,567/-125 |

## 결론

**수용 가능 — review·오늘할일 trailing head의 최신 CI와 PR 본문 현행화 전 조건부**.

code candidate에서 남은 source blocker는 발견하지 못했다. 첫 code head의 녹색 CI 뒤 발견한 v2 semantic
guard 이관 누락과 validator `TypeError`는 `b1afde931`에서 원인 경계를 복구했고, 최초 허용된 변조와 추가
malformed collection을 negative contract로 고정했다. 이 corrected head는 GitHub Actions 24 success/3 policy
skip, 실패·대기 0으로 끝났다.

이 PR은 W8의 실제 font mapping 변경이 아니다. 봉인 schema 1.0의 830개 rule을 의미 변화 없이 lifecycle
schema 2.0으로 이행하고, 후속 변경을 append-only change set·evidence lineage·active-only projection으로
제어하는 기반만 제공한다.

## 이슈 완료 조건 대사

| #5955 완료 조건 | self-review 판정 |
| --- | --- |
| 830개 rule 무의미 변화 이행 | 830 active/0 retired, selection tuple·projection semantic delta 0 |
| add·evidence-only·retire·replace 계약 | reducer와 positive/negative fixture로 고정 |
| evidence parent·digest와 pre/post delta | stale parent, graph, digest와 migration mapping을 결정론적으로 검증 |
| 한 decision plane의 최소 projection 영향 | 대상 한 projection과 비대상 네 semantic hash 불변을 강제 |
| W2 trace lifecycle 설명 | offline audit이 carried/introduced/retired/replaced/historical/dangling을 구분 |
| 제품 선택·metric·renderer output 0-delta | native/WASM 공개 SVG 173쪽 byte mismatch 0, Canvas visual diff 성공 |

## self-review 정정 계보

1. reducer와 독립 registry validator 사이의 same-plane cross-projection successor 허용을 발견해 동일 projection
   successor를 강제했다.
2. 검증 보고서에 남은 Cargo target 사용자 절대경로를 저장소 상대경로로 정정했다.
3. 첫 Full CI 뒤 v2 validator가 v1의 projection별 relation allowlist, metric/supply 소유권, Canvas2D
   family·URL·external과 CanvasKit capability agreement를 이관하지 않았음을 재현했다.
4. `evidenceRecords`, `projections`, evidence/lifecycle ID collection의 malformed 자료형이 오류 목록 대신
   `TypeError`를 만들던 경로를 공통 안전 순회로 고쳤다.

3·4번 발견 뒤 PR을 Draft로 전환하고 blocker comment를 게시했다. 정정 구현은 registry와 change-set payload가
같은 semantic validator를 사용하며, 신규 `unknown` legacy rule과 host absolute/file URL도 fail-closed한다.
JSON Schema는 shape·자료형·상한을 맡고 교차 필드 의미는 수동 validator가 맡는 경계를 조사 정본과 fallback
전략 문서에 명시했다.

## 제출 경계

- schema 1.0 registry·schema, W7 migration과 pre-migration baseline byte를 다시 쓰지 않았다.
- current v2 registry는 830 active/0 retired이고 실제 mapping·projection semantic output을 바꾸지 않았다.
- 새 integration source는 없고 기존 `tests/cases/issue_4966_font_rule_projection.rs`만 현행화했다.
- `tests/generated/`, `tests/suites/manifest.json`, Cargo generated target은 PR diff에 없다.
- private corpus, Hyper-V Oracle, font bytes, sample·PDF·visual asset과 식별 host path를 추가하지 않았다.
- 실제 renderer 변경이 없으므로 새 기준 PDF/PNG는 만들지 않았으며 기존 공개 SVG 전건 parity와 GitHub Canvas
  visual diff로 0-delta를 판정했다.

## 로컬 검증

- v1/v2 registry, projection generator, pre-migration baseline deterministic check: 통과
- v2 focused: 26/26
- 전체 `scripts/tests/font_rule_*.test.mjs`: 96/96
- Rust unit-tier: 4,221 tests / 299 modules / drift 0
- release library: 4,071 pass / 13 ignore
- release-test nextest: 8,208 pass / 41 skip
- Studio Node: 1,070 pass / 1 skip, production build 223 modules
- Native Skia, Docker optimized WASM, fresh WASM Decision Trace 3/3: 통과
- native/WASM 공개 SVG 173쪽: byte mismatch 0
- mutation rehearsal: 8,490 bytes, SHA-256
  `3d38c0fe958c85a7c262497a0fa54d268a8e2c2817412d41d88fb35dafa09d03`
- `cargo fmt --all -- --check`, `git diff --check`, 변경 문서 링크 검사: 통과

추가 진단인 W6 lineage 전역 `--check-manifest`는 이 PR이 수정하지 않은
`mydocs/report/font_metrics_fallback_causal_lineage_20260816.md`의 기존 evidence digest drift를 보고했다.
이번 validator가 사용하는 600개 metric entry identity·순서와 v1 registry 대사는 통과했으며, 별도 기존 부채를
#5955에 섞어 수정하지 않았다.

## GitHub Actions

code candidate `b1afde931`에서 24 success/3 policy skip, 실패·대기 0을 확인했다.

- CI: preflight, archive build/shards, Lint, Native Skia, Frontend package, Build & Test 성공
- CodeQL: JavaScript/TypeScript, Python, Rust와 GHAS CodeQL 성공
- Render Diff: preflight와 Canvas visual diff 성공
- Proptest roundtrip과 Adapter inter-diff 성공
- Frontend unit gates, WASM Build, 이전 cancel-stale run은 정책상 skip

## 남은 절차

1. PR 본문의 정정 전 `font-rule Node 계약 93/93`을 `96/96`으로 현행화하고 semantic guard 복구를 명시한다.
2. 이 review 문서와 `mydocs/orders/20260825.md`만 trailing commit으로 push한다.
3. 최신 trailing head의 preflight·Build & Test aggregate와 fast-pass 판정을 확인한다.
4. 실패·대기 0, `MERGEABLE/CLEAN`과 exact head를 재확인한 뒤 Ready 전환과 merge 승인을 별도로 받는다.
