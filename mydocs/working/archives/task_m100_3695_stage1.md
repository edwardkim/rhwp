# task_m100_3695 사전 승인 WIP 기록 — auto 증거 우선순위

- **Issue**: #3695
- **상위 이슈**: #1528
- **선행 작업**: #3693 `652e2ee27`
- **브랜치**: `codex/issue-3695-export-structure-auto`
- **기준**: stacked branch base `652e2ee27`
- **기록 시각**: 2026-08-01 18:54 KST
- **승인 상태**: 비승인 — 단계 완료보고서가 아닌 WIP 실측 기록

## 0. 문서 성격 정정

이 문서는 #3693의 승인된 단계 완료와 #3695 계획 승인 전에 코드·테스트와 같은 커밋
`8343c98c6`으로 작성됐다. 따라서 Hyper-Waterfall의 승인된 Stage 1 완료보고서가 아니며, 사전 구현
draft에서 관찰한 red/green 결과의 감사 기록으로만 사용한다. 선행 게이트와 계획 승인 후 기존 draft를
다시 검토·검증하고 별도의 단계 보고서를 작성한다.

## 1. red 기준

Number 하나가 섞인 synthetic 조문 문서를 추가했다. 수정 전 `has_outline()`은 Number 하나만 보고
전체 문서를 outline으로 선택했다.

| 기대 | 수정 전 결과 |
| --- | --- |
| effective mode `clause` | `outline` |
| `제1조` node 1개 | Number 문단 node 1개 |
| Number 일반 문단은 조의 body | Number 문단은 outline heading |

신규 통합 테스트는 수정 전 5 passed / 1 failed였고 실패는 이 혼합 증거 사례 하나였다.

## 2. 구현

- boolean `has_outline()`을 문서 단위 `select_auto_mode()`로 교체했다.
- 명시적 Outline을 최우선 증거로 정했다.
- 편·장·절·관·조 marker를 Number보다 강한 clause 증거로 정했다.
- Number는 strong clause marker가 없을 때 outline을 선택하는 fallback 증거로 유지했다.
- 항·호·목은 일반 목록과 모호하므로 auto 선택 증거에서 제외했다.
- explicit outline/clause mode는 selector를 거치지 않는다.

## 3. 표본 조사와 경계

저장소 sample 668개를 읽기 전용으로 조사해 단일 Number도 유효한 제목일 수 있음을 확인했다.
따라서 `Number >= 2` 같은 개수 임계값은 채택하지 않았다.

| sample | head type 증거 | 고정할 결과 |
| --- | --- | --- |
| `hwpctl_API_v2.4.hwp` | Outline 155 | outline |
| `biz_plan.hwp` | Number 20 | outline |
| `추진일정.hwp` | Number 1 | outline |

혼합 조문 결함은 최소 synthetic fixture로 재배포 가능하게 고정했다. 새 binary fixture는 추가하지 않았다.

## 4. green 검증

| 명령 | 결과 |
| --- | --- |
| `cargo test --lib document_core::queries::structure` | 5 passed |
| `cargo test --test issue_3695_structure_auto_policy` | 8 passed |
| `cargo test --test issue_3693_structure_clause_context` | 3 passed |
| `cargo test --test cli_json_contract export_structure_` | 4 passed |
| `cargo fmt --check` | 통과 |
| `cargo clippy --all-targets -- -D warnings` | 통과 |
| `git diff --check` | 통과 |

모든 Cargo 실행은 `CARGO_INCREMENTAL=0`으로 순차 수행했다.

## 5. 다음 단계

- #3693의 정정 계획 승인과 승인된 단계 완료를 기다린다.
- 이후 #3695 수행·구현 계획을 별도로 승인받는다.
- 승인 전에는 추가 구현, release-test, push, PR 생성을 하지 않는다.
- 승인 후 `8343c98c6`을 계획과 대조해 채택·수정·폐기를 판정하고 새 단계 보고서를 작성한다.
