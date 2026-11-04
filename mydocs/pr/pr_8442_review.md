# PR #8442 리뷰 — fix(edit): 서식 JSON의 문자열 밖 공백을 걷어내 읽고, 읽은 키가 없는 --props는 거부한다 (#8439)

## 최종 판정

**메인터너 보정 후 수용 가능**
메인터너가 통합 후보의 실제 전쪽 증적과 회귀 근거를 보강했습니다. 원 PR에 대응하는 입력·쪽과 반례는 아래에 기록했습니다.
원 head와 리베이스·메인터너 보정을 적용한 통합 후보를 구분합니다. 실제 적용 커밋은 아래 표에 기록했습니다.
완료된 검증과 남은 공통 검증을 아래에서 구분합니다. 최종 수용 판정과 PR 생성은 아직 대기입니다.
아래 값은 명시한 코드 SHA의 실측 결과이며 원 PR의 승인이나 닫기를 뜻하지 않습니다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR | [#8442](https://github.com/edwardkim/rhwp/pull/8442) · `planet6897` → `devel` |
| 원 head | `9b7af377c4e293c49c4cbbe776fa094563ffe6b9` |
| 통합 적용 커밋 | `65f67b13405517d38bd9a8bb76415b12c676cd0a` |
| 최종 검토 기준 devel | `22947f6a90c998e95fa51238d5b907a0ebd37162` |
| 최종 필수 검증 제품 코드 | `a7e5ce7112ea8b227c317a9cbfb9d6fed0b00973` |
| 전쪽 시각 측정 producer / 당시 base | `4b149b6cdfc514b9858ed644be2cf8120c731cd4` / `7fdd5441ff1e77c083818e920a48089f294de35e` |
| reviewer | `jangster77` 요청. 2026-11-04T17:44:08.556Z 원 head 불변·Open·non-Draft·요청 상태 재확인 |
| 기본 경로 | `maintainer_general` |
| 보조 경로 | `intake_and_review`, `local_validation`, `multi_pr_update_branch`, `post_merge` |
| 관련 이슈 | [#8439](https://github.com/edwardkim/rhwp/issues/8439) |

## 변경과 검토 범위

원 PR의 변경 주장은 **fix(edit): 서식 JSON의 문자열 밖 공백을 걷어내 읽고, 읽은 키가 없는 --props는 거부한다 (#8439)**입니다. 해당 주장에 연결된 입력과 쪽을 아래에 분리해 적었습니다.
적층 PR에 중복 포함된 선행 커밋과 이 PR의 실제 신규 커밋은 최종 통합 기록에서 구분합니다.

## 검증 입력과 전쪽 결과

이 PR은 위 원본 대조 표의 직접 대상이 아닙니다. 해당 변경의 독립 계약 검사와 입력 생성 결과는 최종 검토에 연결합니다.

## 조판 원칙 준수 검토

조판 구현 변경은 비해당입니다. 서식 JSON의 해석·명령 적용·거부 계약을 바꾸며 줄 구성·배치 알고리즘은 변경하지 않습니다.

## 구현 주장과 검증 증거의 대조

서식 JSON의 문자열 밖 공백을 처리하면서 문자열 내부 공백·escape를 보존하고, 인식한 서식 키가 없는 --props는 오류로 거부한다.

document_core/helpers.rs의 공통 JSON 읽기 경로와 CLI의 본문·셀·머리말/꼬리말·각주 서식 명령이 수정 결과와 빈 변경 여부를 함께 확인한다.

공백이 있는 값·배열·중첩 객체와 빈 객체·알 수 없는 키를 CLI 및 DocumentCore 검사에서 대조한다. 조판 알고리즘 변경은 없으므로 이 PR의 입증 대상은 명령의 적용·거부 계약이다.

관련 정식 회귀 파일: [issue_8439_props_json_whitespace.rs](../../tests/cases/issue_8439_props_json_whitespace.rs)

## 검증 입력 커밋 확인

이번 통합의 [입력·PDF SHA-256 대조표](assets/planet6897_20261103/input-integrity.tsv)에 있는
148개 파일은 증적 고정 커밋 `fd97ae040f`의 파일 내용과 일치합니다.
위 개별 입력은 저장소의 동일 파일을 재사용합니다.

## 원 PR 주장 쪽의 시각 증적

JSON 해석과 명령의 적용·거부 계약이 대상입니다. 해당 명령을 직접 실행하는
정식 회귀의 결과와 최종 전체 회귀 로그를 검증 근거로 연결합니다.

## 통합 최종 검증 조건

- 제품 `a7e5ce7112ea8b227c317a9cbfb9d6fed0b00973`의 전체 Rust 12,561개 통과와 Native Skia·CI 통합 모듈 21개·Rust lint를 확인했습니다.
- 최적화 fresh WASM의 실제 브라우저 실행과 공개 API 검사를 완료했습니다.
- 전쪽 측정 producer에서 Native/fresh WASM 각 80개 입력·406쪽은 모두 90% 이상입니다.
- Native의 당시 base 대비 406쪽 대조와 PR #8389·#8397의 원 TSV 표시값 무하락 대조는 완료했습니다.
- WASM의 당시 base 대비 전쪽 대조와 최신 base 기능 보존의 남은 실행은 아래와 같습니다.

- 대기: 동일 환경에서 당시 base 대비 WASM 80개 입력·406쪽의 0.1%p 하락 게이트 완료
- 새 base의 실제 Native/WASM 제품과 후보의 집중 대조 14쪽은 모두 같은 raw입니다. [실행·대조 원본](assets/planet6897_20261103/rebase_scope_22947f6a90/base-comparison.json)을 확인했습니다.
- 대기: 주장 쪽 직접 판독·회귀 주석·수동 작성 리뷰와 PR 본문 최종 감사

이 대기 항목을 완료하지 않은 상태에서는 최종 검증 완료·승인으로 판정하지 않습니다.
[완료·대기 검증 원본 기록](assets/planet6897_20261103/review-preparation-a7e5ce7112.json)

## Merge 후 원 PR 후속 처리

통합 PR 생성·병합과 원 PR 댓글·닫기는 아직 수행하지 않았습니다.
실제 병합 뒤 원 PR의 주장 문서·페이지에 대응하는 증거와 merge SHA를 연결합니다.

<!-- planet6897-20261103-generated-review -->
