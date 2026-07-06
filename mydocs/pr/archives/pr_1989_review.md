# PR #1989 리뷰 - breakLatinWord 및 secPr 스칼라 보존

## 메타

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1989 |
| 제목 | 10k 서베이 저장 충실도 수정: #1986 breakLatinWord 보존 + #1987 secPr 스칼라 보존 |
| 작성자 | planet6897 |
| base | `devel` |
| 문서 작성 시점 head SHA | `3dc0d217dd56566d00c6dd52b0e7488389ee7a22` |
| 원 commits | `92e557016ddce622ad6283c2bf4a92352c1b0278`, `3dc0d217dd56566d00c6dd52b0e7488389ee7a22` |
| 체리픽 commits | `4cfa371ff`, `44bde5ce2` |
| 메인터너 보강 commit | `fcee51770` |
| 규모 | 6 files, +71 / -2 |
| 처리 방식 | planet6897 PR 8건 통합 체리픽 |

## 변경 범위

- #1986: HWPX `ParaShape`의 `breakSetting@breakLatinWord` 값을 원문 보존한다.
- #1987: HWPX `secPr`의 `spaceColumns`, `outlineShapeIDRef` 같은 스칼라 값을 원문 보존한다.
- 보정 근거는 HWPX 문서 속성 자체이며, 특정 샘플명/페이지/계수로 분기하지 않는다.

## 체리픽 검토

- 적용 순서: 5/8
- 충돌: 없음
- 선행 PR 의존: 없음
- 메인터너 보강: 기존 주석의 이슈 번호가 `#1984`로 오기되어 있어 `#1986`으로 정정하고, `HYPHENATION` 보존 테스트를 추가했다.

## 시각 검증

이 PR은 저장 충실도/roundtrip 직렬화 보존이 핵심이며 직접 렌더 레이아웃을 고치는 PR이 아니다. 기준 PDF가 첨부되어 있지 않으므로 visual sweep은 수행하지 않았다. 검증은 parser/serializer 단위 테스트와 전체 회귀 테스트로 수행했다.

## 로컬 검증

- `env CARGO_INCREMENTAL=0 cargo test --lib test_parse_hwpx_para_shape_break_non_latin_word_bit`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --lib write_para_pr_emits_align_and_break_from_preserved_bits`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --lib issue1987`: 1 passed
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`: 통과
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과
- `cargo fmt --check`: 통과
- `git diff --check`: 통과

참고: `env CARGO_INCREMENTAL=0 cargo test --lib issue1986`은 테스트명 필터 기준 0건이었다. 그래서 메인터너 보강 commit에서 실제 `breakLatinWord` 파서/직렬화 테스트를 명시적으로 보강했다.

## 검토 결과

문서 속성 기반 보존으로 구현되어 있고 테스트 보강 후 검증이 통과했다. 최종 권고는 통합 PR merge 후보다.

