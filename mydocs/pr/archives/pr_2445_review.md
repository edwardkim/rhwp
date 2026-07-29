# PR #2445 리뷰 - HWPX 차례 필드 `TOC` 왕복 보존

## 메타데이터

| 항목 | 값 |
|---|---|
| 원 PR | [#2445](https://github.com/edwardkim/rhwp/pull/2445) |
| 작성자 | `kevin9327` |
| base | `devel` |
| 리뷰 경로 | collaborator-mediated 외부 PR, G3 체리픽 통합 |
| 적용 커밋 | `976ef2672b0b3f45a80c372bfe3b928965013935` |
| 적용 순서 | G3 1/3, 최신 `upstream/devel` 위 체리픽, 충돌 없음 |
| 작성 시점 참고값 | 원 PR은 `BEHIND`, 기존 CI 통과. 통합 PR의 최신 head CI를 merge 전 다시 확인한다. |

## 변경 검토

HWPX 직렬화기는 `FieldType::TableOfContents`를 `TOC`로 기록하지만 파서는
`TABLE_OF_CONTENTS`와 `TABLEOFCONTENTS`만 수용했다. 따라서 rhwp가 저장한 HWPX를 다시
열 때 차례 필드가 `Unknown`으로 바뀌는 내부 왕복 불일치가 있었다.

`TOC`를 같은 `TableOfContents` 분기에 추가했고, 기존의 두 허용 표기는 유지한다. 변경은
파서 token 수용 범위만 넓히며 renderer, 페이지네이션, Studio UI에는 영향을 주지 않는다.

## 검증

| 게이트 | 결과 |
|---|---|
| `parse_field_type_accepts_toc` | PASS |
| `cargo fmt --check` | PASS |
| `git diff --check` | PASS |
| `cargo clippy --all-targets -- -D warnings` | PASS |
| `cargo test --profile release-test --tests` | PASS |

## 시각 검증 판단

파서의 HWPX enum token 복원만 변경하며 렌더러나 레이아웃을 수정하지 않는다. 별도 visual
sweep 또는 MCP PDF 검증 대상이 아니다.

## 최종 의견

G3 통합 PR에 원 커밋을 보존해 수용한다. merge 전 통합 PR 최신 head의 GitHub Actions와
mergeable 상태를 재확인한다.
