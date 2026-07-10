# PR #2143 검토 - Issue #2098/#2138 footer fit 62px margin

- PR: https://github.com/edwardkim/rhwp/pull/2143
- 작성자: `planet6897`
- base: `devel`
- 원 head: `8528810eb9a2da293250e73d99f9b8a21cc9cb4f`
- 체리픽 커밋: `eed791673`
- 포함 README: 새 샘플 README 없음. 기존 `samples/task2098/README.md`는 `vpos0` 샘플만 설명.

## 결론

코드와 테스트는 PR 목적대로 적용된다. `anchor_vpos <= 0`인 page-bottom fixed footer fit
경계 케이스에 62px margin을 주어 10k warm PDF 기준 분할 정답군을 보존하려는 변경이다.

다만 새 합성 fixture `page_bottom_fixed_anchor_margin_split.hwpx`의 MCP 기준 PDF는 1쪽이고,
rhwp 테스트 기대는 2쪽이다. 따라서 이 fixture는 한컴 기준 PDF와 직접 일치하는 샘플이 아니라,
survey/warm PDF에서 관측된 경계 조건을 고정하는 내부 회귀 fixture로 보아야 한다.

## 참고 자료

- #2144 보고서 `mydocs/report/survey_10k_r12_20260710.md`
  - r12 회귀 60건이 결재문서 발신명의 footer 계열
  - warm PDF 권위 재보정 필요
- 기존 `samples/task2098/README.md`
  - 넉넉한 slack의 흡수 정답 샘플은 기존 fixture가 보증
- 새 테스트 `tests/issue_2098_margin_boundary_split.rs`
  - slack 45.6px 경계 케이스를 분할로 고정

## 검증

- reviewer assign 완료
- 체리픽 충돌 없음
- `target/release-test/rhwp dump-pages samples/task2098/page_bottom_fixed_anchor_margin_split.hwpx --page 0`
  - rhwp: 2쪽
- MCP 기준 PDF 생성:
  - `pdf/task2098/page_bottom_fixed_anchor_margin_split-2020.pdf`
  - sha256 `efaf2a6ca90a815bfb54a901d2609f342934c6e109755b06c7ac28905220dc8b`
  - `pdfinfo`: 1쪽
- focused test: `issue_2098_margin_boundary_split` pass
- 관련 회귀:
  - `issue_2098_page_bottom_fixed_anchor_vpos0` pass
  - `issue_1611_footer_page_bottom_pagination` pass
  - `issue_1733`, `issue_1750_split_guard_spacing_before`, `issue_1842`, `issue_1891` pass
- 전체 검증:
  - `cargo fmt --check` pass
  - `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` pass
  - `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings` pass

## 리스크

- 합성 fixture의 MCP PDF와 테스트 기대값이 갈린다.
- PR 설명의 권위는 새 fixture가 아니라 10k warm PDF survey다. merge 후 issue close나 코멘트에서
  이 차이를 명시해야 한다.
- margin 값은 스칼라 최적점이며 저슬랙 흡수 2건은 기지 한계로 남는다.

## 권고

누적 체리픽 PR에는 포함 가능하다. 단 #2098/#2138 잔여 판별 신호 조사는 별도 후속으로 유지한다.
