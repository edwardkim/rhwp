# PR #2450 리뷰 - OWPML 필드 타입 스키마 표기 정합

## 메타데이터

| 항목 | 값 |
|---|---|
| 원 PR | [#2450](https://github.com/edwardkim/rhwp/pull/2450) |
| 작성자 | `kevin9327` |
| base | `devel` |
| 리뷰 경로 | collaborator-mediated 외부 PR, G3 체리픽 통합 |
| 적용 커밋 | `e01647321b5453c77c787f693d64ff1a062815b5` |
| 적용 순서 | G3 2/3, 최신 `upstream/devel` 위 체리픽, 충돌 없음 |
| 작성 시점 참고값 | 원 PR은 `BEHIND`, 기존 CI 통과. 통합 PR의 최신 head CI를 merge 전 다시 확인한다. |

## 변경 검토

권위 OWPML schema의 `FieldType` enum은 `SUMMERY`, `USER_INFO`, `DOC_DATE`를 사용한다.
기존 직렬화기의 `SUMMARY`, `USERINFO`, `DOCDATE`는 이 스키마 표기와 달라 Hancom이 필드
타입을 인식하지 못할 수 있었다.

저장 표기를 스키마와 맞추고, 파서는 호환성을 위해 기존 표기와 `SUMMERY`를 모두 수용한다.
스키마의 `SUMMERY` 오탈자도 실제 enum 값이므로 그대로 보존하는 것이 맞다.

## 근거

[`ParaList XML schema.xml`](../../manual/OWPML%20SCHEMA/ParaList%20XML%20schema.xml)의
`FieldType` enum은 `SUMMERY`, `USER_INFO`, `DOC_DATE`를 명시한다. 리뷰 시 실제 파일의
2707-2710행을 대조했다.

## 검증

| 게이트 | 결과 |
|---|---|
| `field_type_str_matches_owpml_schema` | PASS |
| `cargo fmt --check` | PASS |
| `git diff --check` | PASS |
| `cargo clippy --all-targets -- -D warnings` | PASS |
| `cargo test --profile release-test --tests` | PASS |

## 시각 검증 판단

HWPX parser/serializer 문자열 정합만 변경한다. renderer, 페이지네이션, Studio UI를 바꾸지
않으므로 별도 visual sweep 또는 MCP PDF 검증 대상이 아니다.

## 최종 의견

G3 통합 PR에 원 커밋을 보존해 수용한다. merge 전 통합 PR 최신 head의 GitHub Actions와
mergeable 상태를 재확인한다.
