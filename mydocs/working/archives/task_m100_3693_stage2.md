# task_m100_3693 Stage 2 완료보고서 — WIP 독립 대조와 focused 재검증

- **Issue**: #3693
- **상위 이슈**: #1528
- **브랜치**: `codex/issue-3693-export-structure-clause`
- **승인된 범위**: WIP 독립 대조, focused 검증, 단계 보고서 작성
- **검토 대상 WIP**: `652e2ee27`
- **절차 복구 커밋**: `b439ce1d2`
- **계획 승인 체크포인트**: `8f9064271`
- **완료 시각**: 2026-08-01 19:28 KST
- **상태**: Stage 2 완료·승인, WIP 채택 확정

## 1. 검토 방법

기존 WIP 보고서의 결론을 전제로 삼지 않고 `upstream/devel..652e2ee27`의 소스·테스트 diff를
정정 승인된 수행·구현 계획과 다시 대조했다. 현재 HEAD와 WIP 사이의 소스·테스트 diff가 0임을 먼저
확인해 절차 복구 문서 커밋이 구현을 바꾸지 않았음을 고정했다.

이번 단계에서는 소스를 수정하지 않았다. 계획 미충족이나 추가 개선 필요 사항은 보고서에만 기록하고
다음 승인 지점에서 멈춘다.

## 2. 계획 대비 독립 대조

| 승인 계획 항목 | WIP 근거 | 판정 |
| --- | --- | --- |
| `제N조의M` marker 전체 보존 | `classify_clause()`가 `조` 뒤 `의`와 연속 숫자를 marker에 포함 | 충족 |
| `N)`/`가)` 변형 후보 인식 | 숫자·한글 marker delimiter를 `.` 또는 `)`로 판정 | 충족 |
| 일반 숫자 목록 과검출 완화 | `clause_heading_allowed()`가 `호`에 열린 `조|항`, `목`에 열린 `호` 요구 | 충족 |
| 거부 후보 텍스트 보존 | heading 거부 시 기존 preamble/body 경로 사용 | 충족 |
| `조 → 항 → 호 → 목` 계층 | synthetic 4단계 트리 단위 테스트 | 충족 |
| 실제 협정서 positive 보존 | `hwp3-sample16-hwp5.hwp`의 `제1조 → 1./2./3.` | 충족 |
| 일반 문서 negative | 업무계획 날짜와 편람 목차 번호 실문서 테스트 | 충족 |
| JSON·explicit outline 호환 | 구조체·CLI parser 변경 없음, CLI JSON 계약 4건 통과 | 충족 |
| 파서·렌더·직렬화 비범위 | 변경 파일이 structure query와 #3693 테스트로 한정 | 충족 |
| 신규 binary fixture 미추가 | 저장소 기존 sample과 synthetic Document만 사용 | 충족 |

계획과 WIP 사이의 누락·범위 초과·문서와 다른 구현은 발견하지 못했다.

## 3. 코드 수준 검토

- 가지번호 suffix는 `조`에만 적용되고 `의` 뒤 숫자가 있을 때만 marker 끝을 확장한다.
- 괄호형 delimiter 확장은 기존 점형 marker 동작을 보존한다.
- 약한 후보의 수용 판정이 tree stack 변경 전에 실행되지만, 이전 strong heading을 처리할 때 같은/하위
  stack이 이미 닫히므로 닫힌 조문 문맥이 다음 일반 목록에 남는 경로는 발견하지 못했다.
- 거부된 후보는 `None` 경로로 들어가 공백 제거 뒤 preamble 또는 현재 node body에 보존된다.
- 공개 `StructureDoc`·`StructureNode` shape, mode 문자열, 좌표 필드는 바뀌지 않는다.

## 4. focused 재검증

모든 Cargo 명령은 `CARGO_INCREMENTAL=0`으로 순차 실행했다.

| 명령 | 결과 |
| --- | --- |
| `cargo test --lib document_core::queries::structure -- --nocapture` | 5 passed |
| `cargo test --test issue_3693_structure_clause_context -- --nocapture` | 3 passed |
| `cargo test --test cli_json_contract export_structure_ -- --nocapture` | 4 passed |
| `cargo fmt --check` | 통과 |
| `cargo clippy --all-targets -- -D warnings` | 통과 |
| `git diff --check` | 통과 |
| `git diff --exit-code 652e2ee27..HEAD -- <source/test>` | 차이 0 |

## 5. 잔여 위험과 범위 경계

- 열린 `조|항` 안에서 문단이 실제로 `2022. 1.`처럼 시작하면 부모 문맥 때문에 `호` 후보로 수용될
  수 있다. 현재 승인 계획은 standalone 날짜·목차를 거부하면서 실제 `조 → 호`를 보존하는 부모 문맥
  정책이며, 두 형태를 더 구분하려면 이웃 연속성·번호 흐름 같은 별도 정책이 필요하다.
- 원문자 `항`은 기존 strong marker로 유지돼 standalone 원문자 목록 과검출 가능성이 남는다. 이는
  #3693의 승인 범위에서 변경하지 않기로 한 항목이다.
- 실문서 테스트의 section/paragraph 좌표는 sample이 편집되면 갱신이 필요하지만, 현재 코퍼스에서는
  구조 위치를 직접 고정하는 유효한 회귀 기준이다.

위 위험은 승인된 계획의 누락이 아니라 계획이 의도적으로 남긴 경계다. 현재 재현된 positive/negative
완료 조건을 막지는 않지만, 더 강한 clause confidence가 필요하면 별도 후속 범위로 다루는 편이 안전하다.

## 6. Stage 2 판정 결과

`652e2ee27`은 정정 승인된 #3693 계획과 일치하고 focused 검증을 모두 통과했다. 소스 수정 없이 이
WIP를 #3693의 승인된 구현으로 **채택**하는 것을 권고했다.

작업지시자가 2026-08-01 채택을 승인했다. 이에 따라 기존 WIP는 재작성하지 않고 승인된 #3693
구현으로 확정한다. full release-test, push, PR은 별도 승인 게이트에 남기고, #3695는 별도 계획 승인
뒤에만 재개한다.
