---
kind: report
status: completed
canonical: mydocs/plans/archives/task_m100_4966.md
last_verified: 2026-08-23
---

# Task M100 #4966 — Stage W7-3 결정론적 backend projection generator

## 1. 판정

Stage W7-3는 통과했다. canonical registry 830행을 Rust layout 2종과 TypeScript Studio 3종의 정적
source로 생성하고, 다섯 파일과 manifest를 하나의 paired output set으로 관리한다. 생성 source는 아직
어느 runtime module에도 연결하지 않았으므로 Rust·Canvas2D·webfont·CanvasKit의 소비 동작은 바뀌지
않았다.

초기 구현 검토에서는 각 output metadata에 전체 registry digest를 넣지 않았다. 그렇게 하면 한 backend
규칙 변경이 global digest 때문에 무관한 네 파일까지 바꾸기 때문이다. 전체 registry digest는 manifest에만
두고 각 source는 자기 `inputSha256`과 `projectionSha256`만 가지므로 W7-I13 최소 영향이 성립한다.

## 2. 생성 구조와 소유권

| projection | 언어 | rule | 생성 source |
| --- | --- | ---: | --- |
| `rust-layout-name` | Rust | 171 | `src/renderer/font_rule_projections/layout_name.rs` |
| `rust-layout-metric` | Rust | 67 | `src/renderer/font_rule_projections/layout_metric.rs` |
| `canvas2d-paint` | TypeScript | 281 | `rhwp-studio/src/core/generated/font-rule-projections/canvas2d-paint.ts` |
| `canvas2d-webfont` | TypeScript | 153 | `rhwp-studio/src/core/generated/font-rule-projections/webfont-supply.ts` |
| `canvaskit-sfnt` | TypeScript | 158 | `rhwp-studio/src/core/generated/font-rule-projections/canvaskit-sfnt.ts` |
| **합계** |  | **830** | 고정 allowlist 5개 |

각 source 첫 줄은 `@generated` whole-file ownership sentinel이다. hand-written module의 marker 구간을
수정하는 대신 전용 생성 디렉터리의 파일 전체만 소유한다. 따라서 generator는 renderer 알고리즘,
metric data/overlay와 Studio loader/substitution source를 덮어쓸 경로가 없다.

각 항목은 W1에서 이어진 `ruleId`, relation, decision plane, source/target, condition, order, mode와 필요한
W6 metric ID 또는 supply payload를 보존한다. Rust·TypeScript 파일은 schema version, backend별 input
digest와 projection digest를 함께 가진다.

## 3. manifest와 digest

`assets/font-rules/font_rule_projection_manifest.json`은 다음 authority를 고정한다.

- registry canonical digest와 `rulesSha256`
- generator source와 manifest schema digest
- 고정 output path, language, ownership과 rule count
- backend별 `inputSha256`, semantic `projectionSha256`와 source `contentSha256`
- projection bundle과 content bundle digest

| 항목 | SHA-256 |
| --- | --- |
| registry | `f549ca3a8807be712cc197daf14d96abb1e5f075ac55f1d9142db67c1a56681a` |
| registry rules | `34838af25531327b9e697b065ed5771a11f310c970a9923c83a0b6e1235a68bd` |
| projection bundle | `a8a07043a689e91760e9a09d1e3ee2b0aaf416b006e1b9dce5dfde42f00699e1` |
| content bundle | `6feb537db1dbfa42f2fb1a8e266f064a5d505c9af6b0fc77c11d55bca7d756b7` |

generator source digest는 generator 자체가 바뀌면 manifest를 stale로 만든다. wall-clock, filesystem
열거 순서, host path와 locale은 source나 digest에 들어가지 않는다.

## 4. 생성·검사 계약

```bash
node scripts/font_rule_projection_gen.mjs generate
node scripts/font_rule_projection_gen.mjs check
node --test scripts/tests/font_rule_projection_gen.test.mjs
```

`generate`는 다음 순서로 동작한다.

1. registry schema/semantic 계약을 먼저 검사하고 다섯 source와 manifest를 memory에서 완성한다.
2. fixed checkout-root allowlist, non-symlink parent와 기존 whole-file ownership을 확인한다.
3. 모든 내용을 sibling staging file에 쓰고 `fsync`한다.
4. 기존 set 전체를 backup한 뒤 새 set을 commit한다.
5. 중간 commit 실패 시 이미 반영한 파일을 제거하고 기존 set 전체를 복원한다.
6. 성공한 경우 backup/staging을 정리한다.

호출자가 output 경로를 지정할 수 없고, generated 전용 디렉터리에 allowlist 밖 파일이 있으면 생성 자체를
거부한다. 기존 hand-written 파일, symlink escape와 manifest 일부만의 갱신도 fail-closed한다.

`check`는 다시 생성한 canonical bytes와 실제 다섯 파일·manifest를 비교한다. missing, stale, manual
edit, partial set, generator/schema/registry digest drift와 예상 밖 파일을 각각 오류로 판정한다.

## 5. 부정 계약과 최소 영향

focused contract는 다음 10개 축을 검증한다.

- 동일 입력 연속 생성의 byte 결정론과 830행 폐합
- path/language/ruleId 순서의 canonical registry 일치
- 단일 Canvas2D paint rule 변경 시 `canvas2d-paint.ts`만 변경
- projection 항목 순서 교란 시 해당 projection digest만 변경
- missing/manual/unexpected output 탐지
- ownership sentinel 없는 source overwrite 거부
- preflight 실패 시 기존 set 불변
- commit 중간 실패 주입 시 전체 set rollback
- checkout 밖 symlink parent 거부
- 호출자 지정 output path 거부

manifest는 전체 registry 변경 때문에 갱신될 수 있지만, 다섯 backend source 중 무관한 파일은 바뀌지
않는다. 이것이 global registry digest를 source metadata에서 분리한 이유다.

## 6. 검증

```bash
rustfmt --edition 2021 --check \
  src/renderer/font_rule_projections/layout_name.rs \
  src/renderer/font_rule_projections/layout_metric.rs

cd rhwp-studio
npm exec -- tsc --noEmit
```

Rust generated source는 rustfmt parser를 통과하고 TypeScript 3개 source는 Studio strict typecheck를
통과해야 한다. runtime module import가 없으므로 이 단계에서는 renderer 결과 동등성 주장을 하지
않는다. 소비자 전환 전후의 exhaustive lookup과 trace 비교는 W7-4·5의 승인 게이트다.

최종 결과는 generator focused contract 10/10, W1·W6·W7 결합 Node contract 63/63 통과다. Rust 두
source는 rustfmt와 독립 `rustc --emit metadata`를, TypeScript 세 source는 Studio strict typecheck를
통과했다. 세 JSON의 Draft 2020-12 schema validation, 변경 문서 607개 링크 검사와
`git diff --check`도 통과했다.

## 7. Stage W7-4 인계

Stage W7-4는 Rust source 두 개만 module tree에 연결해 `style_resolver`의 유한 mapping과
`resolve_metric_alias`를 generated projection 소비로 바꾼다. language 판정, metric first-match와
exact → bold-only → name-first 사다리, W6 metric data/index는 hand-written owner에 그대로 둔다.
Rust 전수 lookup·W6 hash·W2 trace가 migration 전 기준선과 같을 때만 W7-5 진입을 요청한다.
