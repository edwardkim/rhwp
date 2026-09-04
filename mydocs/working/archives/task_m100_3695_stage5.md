# task_m100_3695 Stage 5 완료보고서 — PR 리뷰 confidence 보정

- **Issue**: [#3695](https://github.com/edwardkim/rhwp/issues/3695)
- **상위 이슈**: [#1528](https://github.com/edwardkim/rhwp/issues/1528)
- **Draft PR**: [#3749](https://github.com/edwardkim/rhwp/pull/3749)
- **리뷰**: [pullrequestreview-4838218628](https://github.com/edwardkim/rhwp/pull/3749#pullrequestreview-4838218628)
- **보정 전 head**: `4df21a0219733d70911373f2824073437213580b`
- **최신 devel**: `3d4863a0d58d9abf93544318e14856d3c72e92ce`
- **devel 통합 commit**: `f2b93b7ee`
- **보정 code commit**: `fd45184f1`
- **완료일**: 2026-08-02

## 1. 리뷰 판정

리뷰의 High 두 건을 독립 재현했다.

1. 시장구조조사 실문서는 이전 devel에서 `outline / 3 nodes`였지만 보정 전 PR head에서
   `clause / 51 nodes`로 바뀌었다. 목차 쪽번호 행 22건과 실제 본문 절이 중복되고 장 계층은 없었다.
2. Number 개요 문서의 본문 한 줄이 `제3조의 규정에 따라`로 시작하면 문서 전체가 clause로 바뀌고,
   상호참조 문장이 조 제목으로 승격됐다.

두 사례 모두 #3695의 “순수 outline 유지” 완료 조건에 걸리므로 #3744로 미루지 않고 현재 PR에서
보정했다. #3744는 explicit clause 문맥 만료·날짜·목 confidence 범위를 그대로 유지한다.

## 2. red와 정책 결정

회귀 테스트를 먼저 추가한 결과 9 passed / 3 failed였다.

| 실패 | 보정 전 결과 | 기대 |
| --- | --- | --- |
| 시장구조조사 실문서 | clause / 51 | outline / 3 |
| Number + `제3조의 규정에 따라` | clause | outline |
| Number + `제1조 목적\t12` | clause | outline |

단순 marker 개수 임계값은 목차 22건을 막지 못하고 단일 조문 문서를 손상하므로 채택하지 않았다.
Number와 충돌할 때 편·장·절·관은 일반 보고서 container에도 흔하다는 점을 반영해 독립 clause 증거에서
제외했다. 다음을 모두 만족하는 `조` 제목만 Number보다 강한 증거로 인정한다.

- 탭 뒤 ASCII 쪽번호로 끝나는 목차 행이 아니다.
- marker 뒤가 `의`, `에`, `을/를`, `은/는`, `에서` 등 조사형 상호참조가 아니다.
- `제1조의무 규정`처럼 조사와 같은 음절로 제목이 시작하더라도 조사 뒤 경계가 없으면 정상 제목이다.

explicit `--mode clause`의 classifier와 #3693 stack gate는 변경하지 않았다.

## 3. 구현과 회귀

- `has_toc_page_number_tail()`로 탭+쪽번호 목차를 배제한다.
- `starts_with_reference_particle()`는 marker 뒤 조사와 다음 경계를 함께 확인한다.
- `auto_clause_heading_allowed()`는 위 confidence와 `heading.kind == "조"`를 결합한다.
- 조 증거를 찾은 뒤에는 나머지 문단의 텍스트 조립을 생략하되, 뒤쪽 explicit Outline 탐색은 계속한다.
- 시장구조조사 negative, 조사형 상호참조·쪽번호 synthetic negative를 추가했다.
- 실제 협정서 `hwp3-sample16-hwp5.hwp`에 테스트 안에서 Number 증거를 주입해 실제 `제1조`가 clause를
  유지하는 positive를 추가했다. 새 binary fixture는 만들지 않았다.

green 결과는 13 passed / 0 failed다.

## 4. 코퍼스 영향

이전 devel auto 정책과 보정 auto를 동일한 parse 결과에서 비교했다. 읽기·parse 가능한 파일만 결과를
비교했고 암호 sample 3건은 별도로 셌다.

| 범위 | 후보 | parse 성공 | parse 실패 | mode 변화 | node_count 변화 |
| --- | ---: | ---: | ---: | ---: | ---: |
| `samples/` top-level | 351 | 348 | 3 | 0 | 0 |
| `samples/` 재귀 | 668 | 665 | 3 | 0 | 0 |

parse 실패는 `HWP3-password-123456.hwp`, `HWP5-password-123456.hwpx`,
`hwp3-sample16-hwp5-2024-password-123456.hwp`이며 password 없는 공통 parser 호출의 예상 결과다.
보정 전 직접 재현된 시장구조조사 mode 변화는 기존 outline 3으로 복구됐다.

## 5. 최신 devel 결합 검증

모든 Cargo 명령은 `CARGO_INCREMENTAL=0`으로 순차 실행했다.

| 게이트 | 결과 |
| --- | --- |
| structure lib 단위 | 6 passed |
| `issue_3695_structure_auto_policy` | 13 passed |
| `issue_3693_structure_clause_context` | 3 passed |
| CLI JSON `export_structure_` | 4 passed |
| `cargo test --profile release-test --tests` | 최종 exit 0, 실패 0 |
| `cargo fmt --check` / `git diff --check` | 통과 |
| `cargo clippy --all-targets -- -D warnings` | 통과 |

query-only 변경이라 renderer/layout/paint 및 visual fixture에는 영향이 없어 시각 검증은 비대상이다.

## 6. 잔여 trade-off와 다음 게이트

조 증거가 없거나 문서 뒤쪽에서 처음 나타나면 auto selector와 build의 2-pass 텍스트 조립 비용은 남는다.
이번 보정은 조 증거 발견 뒤의 불필요한 조립만 생략한다. 공개 JSON·exit code·explicit mode는 불변이다.

review·review_impl·오늘할일을 같은 PR diff에 포함했다. 보정 head를 push하고 PR 코멘트로 공유한 뒤 최신
GitHub Actions를 확인한다. draft 해제·merge는 최신 CI와 작업지시자 승인 뒤에만 진행한다.
