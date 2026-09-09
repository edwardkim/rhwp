# task_m100_3695 Stage 2 완료보고서 — WIP 독립 대조와 focused 재검증

- **Issue**: #3695
- **상위 이슈**: #1528
- **브랜치**: `codex/issue-3695-export-structure-auto`
- **승인된 범위**: WIP 독립 대조, focused 검증, 단계 보고서 작성
- **선행 승인 구현**: #3693 `652e2ee27`
- **검토 대상 WIP**: `8343c98c6`
- **절차 복구 커밋**: `1f375d431`
- **계획 승인 체크포인트**: `3de8b1709`
- **완료 시각**: 2026-08-01 19:39 KST
- **상태**: Stage 2 완료·승인, WIP 채택 확정

## 1. 검토 방법

#3693 채택본 `652e2ee27`과 #3695 WIP `8343c98c6` 사이의 source·test·CLI manual diff를 분리해
정정 승인된 수행·구현 계획과 다시 대조했다. 현재 HEAD와 WIP 사이의 해당 파일 diff가 0임을 먼저
확인해 절차 복구·승인 문서 커밋이 구현이나 공개 매뉴얼을 바꾸지 않았음을 고정했다.

이번 단계에서는 소스와 CLI 매뉴얼을 수정하지 않았다. 계획 미충족이나 추가 개선 필요 사항은
보고서에만 기록하고 다음 승인 지점에서 멈춘다.

## 2. 계획 대비 독립 대조

| 승인 계획 항목 | WIP 근거 | 판정 |
| --- | --- | --- |
| 명시적 `HeadType::Outline` 최우선 | selector가 참조 paragraph의 Outline에서 즉시 outline 반환 | 충족 |
| primary clause marker가 Number보다 우선 | 편·장·절·관·조 검출 시 clause 선택 | 충족 |
| strong clause가 없을 때 Number fallback | `has_number`가 true면 outline 선택 | 충족 |
| 증거가 없을 때 clause fallback | 마지막 분기에서 clause 선택 | 충족 |
| 항·호·목을 auto 강한 증거에서 제외 | primary kind match가 편·장·절·관·조로 한정 | 충족 |
| explicit mode 동작 불변 | `StructureMode::Auto`에서만 selector 호출 | 충족 |
| pure clause·mixed·single Number 경계 | synthetic 테스트 5건 | 충족 |
| 실제 Outline·복수/단일 Number 보존 | 기존 sample 3종 통합 테스트 | 충족 |
| #3693 clause 회귀 보존 | #3693 실문서 테스트 3건 재통과 | 충족 |
| JSON mode·봉투 계약 불변 | effective mode assertion과 CLI JSON 계약 4건 통과 | 충족 |
| CLI auto 정책 문서화 | `mydocs/manual/cli_commands.md`에 같은 우선순위 명시 | 충족 |
| 파서·렌더·직렬화 비범위 | structure query·정책 테스트·CLI manual로 변경 한정 | 충족 |

계획과 WIP 사이의 누락·범위 초과·문서와 다른 구현은 발견하지 못했다.

## 3. 코드 수준 검토

- selector는 실제 paragraph가 참조하는 para shape만 증거로 사용해 미사용 Outline/Number 정의가 mode를
  바꾸지 않는다.
- Outline은 발견 즉시 반환하므로 문서 순서와 무관하게 authoritative 증거가 된다.
- Number는 flag로만 보존하고 전체 문서의 primary clause marker 조사 뒤 사용해 두 증거의 우선순위가
  코드 흐름과 일치한다.
- primary marker 텍스트는 build 단계와 같은 수식 포함 조립기를 사용해 selector와 실제 heading 분류의
  텍스트 표면이 어긋나지 않는다.
- effective mode를 한 번 선택한 뒤 기존 outline/clause builder를 그대로 사용한다.
- 공개 `StructureDoc`·`StructureNode` shape, mode 문자열, CLI option parser는 바뀌지 않는다.

## 4. focused 재검증

모든 Cargo 명령은 `CARGO_INCREMENTAL=0`으로 순차 실행했다.

| 명령 | 결과 |
| --- | --- |
| `cargo test --lib document_core::queries::structure -- --nocapture` | 5 passed |
| `cargo test --test issue_3695_structure_auto_policy -- --nocapture` | 8 passed |
| `cargo test --test issue_3693_structure_clause_context -- --nocapture` | 3 passed |
| `cargo test --test cli_json_contract export_structure_ -- --nocapture` | 4 passed |
| `cargo fmt --check` | 통과 |
| `cargo clippy --all-targets -- -D warnings` | 통과 |
| `git diff --check` | 통과 |
| `git diff --exit-code 8343c98c6..HEAD -- <source/test/manual>` | 차이 0 |

## 5. 잔여 위험과 범위 경계

- Outline head type 없이 Number만 쓰는 개요 문서 본문에 `제N조` 모양 인용이 있으면 primary clause
  증거가 Number보다 우선해 clause를 선택할 수 있다. 현재 승인 정책의 의도된 우선순위이며, 이를 더
  구분하려면 marker 연속성·비율·위치 같은 추가 confidence 정책이 필요하다.
- auto는 mode 선택을 위해 문단 텍스트를 전수 조립하고 실제 build에서 다시 순회한다. 정확성을 위한
  단순한 2-pass 구조지만 대형 문서 성능 수치는 이번 범위에서 측정하지 않았다.
- 명시적 Outline 하나가 문서 전체를 outline으로 선택하는 정책은 그대로다. 이는 작성자 지정 증거를
  authoritative로 본 승인 설계이며 일반 Number 하나와 구분한 결과다.
- 실제 Outline sample 테스트는 mode와 100개 초과 node를 고정하지만 정확한 155개를 고정하지 않는다.
  완료 조건은 pure outline 선택 보존이므로 충족하지만 세부 node 수 golden은 별도 강화 후보다.

위 위험은 승인된 정책의 누락이 아니라 현재 confidence 경계다. 지금 재현된 pure/mixed/실문서 완료
조건을 막지는 않지만 더 세밀한 auto 분류나 성능 예산이 필요하면 별도 후속 범위로 다루는 편이 안전하다.

## 6. Stage 2 판정 결과

`8343c98c6`은 정정 승인된 #3695 계획과 일치하고 focused 검증을 모두 통과했다. 소스 수정 없이 이
WIP를 #3695의 승인된 구현으로 **채택**하는 것을 권고했다.

작업지시자가 2026-08-01 채택을 승인했다. 이에 따라 기존 WIP는 재작성하지 않고 승인된 #3695
구현으로 확정한다. full release-test, push, PR은 별도 승인 게이트에 남기고, #1528 최종 통합 검증은
#3693·#3695가 통합된 뒤에만 진행한다.
