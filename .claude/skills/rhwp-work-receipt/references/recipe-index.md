# 레시피 색인

워크스루·봉투·레이아웃을 한 표로 잇는다. 새 명령을 만들지 않는다.

## 영수증

| 요청 | 예제 | 봉투 | 명령 |
|------|------|------|------|
| 3해시 발급 | 01 | `replay_attest.json` | `replay --json` |
| 제3자 검증 일치 | 02 | `replay_verify_match.json` | `replay --expect-output-sha256` |
| 검증 불일치 | 03 | `replay_verify_mismatch.json` | 위 + exit 3 |
| 계획 파일 | 04 | (01 과 동일 키) | `replay plan.json` |
| 짧은 해시 | — | `replay_expect_not_hex.json` | exit 2 |
| 계획 없음 | — | `replay_usage.json` | exit 2 |
| 계획 IO | — | `replay_io.json` | exit 1 |

## 캡슐

| 요청 | 예제 | 픽스처 | 명령 |
|------|------|--------|------|
| 캡슐 발급 | 05 | `capsules/*.capsule.json` | `replay --capsule` |
| 같은 폴더 체인 | 06 | `audit-layouts/same-folder-chain` | `--parent a.capsule.json` |
| 하위 폴더 상대 경로 | 07 | `lineage-layouts/relative-subdir` | `parent.capsule=../root/…` |
| 불변 위반 | 08 | `tamper_pretty_print.capsule.json` | 재발급 |
| 같은 파일 | 09 | `replay_parent_same_file.json` | exit 2 |
| 실산출 후 체인 | 10 | — | `run` 다음 `replay --parent` |
| 부모 IO | — | `replay_parent_missing.json` | exit 1 |

## 감사

| 요청 | 예제 | 레이아웃 | 기대 |
|------|------|----------|------|
| 전건 재현 | 11 | `audit-layouts/all-ok` | rate 1.0, exit 0 |
| 혼합 회계 | 12 | `audit-layouts/mixed` | 2/3, exit 3 |
| 비재귀 | 13 | `audit-layouts/nested-ignored` | total 1 |
| 빈 폴더 | 14 | `audit-layouts/empty` | exit 2 |
| 위장 확장자 | — | `audit-layouts/mixed-ext` | `*.capsule.json` 만 |
| 입력 변조 | — | `audit_input_tamper.json` | `kind: inputSha256` |

## 계보

| 요청 | 예제 | 봉투/레이아웃 | 기대 |
|------|------|----------------|------|
| 뿌리 | 15 | `lineage_root.json` | 3축 null |
| 두 링크 | 16 | `lineage_two_link.json` | parentOk·lineageOk |
| deep | 17 | `lineage_deep.json` | reproduced true |
| brokenAt | 18 | `lineage_parent_tamper.json` | exit 3 |
| 연대 불변식 | — | `lineage-layouts/lineage-broken` | lineageOk false |
| 해시 누락 | — | `lineage-layouts/missing-parent-sha` | fail-closed |
| 머리 없음 | — | `lineage_missing_head.json` | exit 1 |
| 무인자 | — | `lineage_usage.json` | exit 2 |
| 3링크 | — | `lineage-layouts/three-link` | depth 3 |

## 함정

| 요청 | 예제 | 문서 |
|------|------|------|
| 버전 불일치 | 19 | [pitfalls.md](pitfalls.md) §1 |
| 누가 했는지 | 20 | [pitfalls.md](pitfalls.md) §2 |

시나리오 전수(요청 문구 포함)는 `fixtures/scenario_catalog.json`.
해시 벡터는 `fixtures/hash-vectors/vectors.json`.
CLI argv 표본은 `fixtures/transcripts/`.
