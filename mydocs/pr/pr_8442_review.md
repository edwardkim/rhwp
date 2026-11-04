# PR #8442 리뷰 — fix(edit): 서식 JSON의 문자열 밖 공백을 걷어내 읽고, 읽은 키가 없는 --props는 거부한다 (#8439)

## 최종 판정

**머지 보류 — 통합 최종 head의 Native/fresh WASM 전쪽 측정과 기준 devel 대비 대조, 전체 로컬 검증이 진행 중입니다.**
아래 값은 명시한 코드 SHA의 실측 결과이며 원 PR의 승인이나 닫기를 뜻하지 않습니다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR | [#8442](https://github.com/edwardkim/rhwp/pull/8442) · `planet6897` → `devel` |
| 원 head | `9b7af377c4e293c49c4cbbe776fa094563ffe6b9` |
| 통합 적용 커밋 | `3d492e9ad10e` |
| 검토 기준 devel | `7d06d5c706f1d89f1662e739abf2b1b25d2a98e4` |
| 실측 제품 코드 | `dc87f4fa2e` (이후 회귀·문서 변경과 구분) |
| reviewer | `jangster77` 요청. 2026-11-04 원 head 불변·Open·non-Draft·요청 상태 재확인 |
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

## 남은 검증

- 현재 통합 최종 head의 집중·전체 Rust 회귀, lint와 fresh WASM을 완료하고 로그를 연결합니다.
- 고정 `upstream/devel`과 같은 입력·Print·글꼴 조건에서 전쪽 raw 하락을 비교합니다.
- 위 주장 쪽 review와 overlay를 독립 Print와 확대 대조해 내용·순서·격자·간격을 직접 판독합니다.
- 누락 증적이나 90% 미만 쪽이 있으면 보정 후 다시 측정합니다.

## Merge 후 원 PR 후속 처리

통합 PR의 merge SHA와 검증한 동일 입력·쪽의 TSV 및 review/overlay를 원 PR에 연결합니다.
댓글 게시와 원 PR 닫기는 최종 통합 검증 및 merge 후에 판정합니다.

<!-- planet6897-20261103-generated-review -->
