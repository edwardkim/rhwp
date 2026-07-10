# PR #2142 검토 - Issue #2137 TopAndBottom float saved-bounds 신뢰

- PR: https://github.com/edwardkim/rhwp/pull/2142
- 작성자: `planet6897`
- base: `devel`
- 원 head: `ce6a7eced019412cd5cc1f7898b23fa6c64e181f`
- 체리픽 커밋: `bb9f9734a`
- 포함 README: `samples/task2137/README.md`

## 결론

blocking finding 없음. 공개 보도자료 샘플에서 비-TAC TopAndBottom 소형 float를 가진
단일 줄 앵커를 saved-bounds 신뢰 경로에 포함하는 변경은 PR 목적과 맞다.

## README 반영

README는 샘플 출처와 기대값을 확인 가능하게 적고 있다.

- 샘플: `samples/task2137/156618554_petfood_press.hwp`
- 출처: korea.kr 공개 동정자료
- 결함: 마지막 빈 앵커와 소형 부동 글상자가 2쪽 단독으로 밀림
- 기대: 한글 1쪽, rhwp도 1쪽

## 검증

- reviewer assign 완료
- 체리픽 충돌 없음
- `target/release-test/rhwp dump-pages samples/task2137/156618554_petfood_press.hwp --page 0`
  - rhwp: 1쪽
- MCP 기준 PDF 생성:
  - `pdf/task2137/156618554_petfood_press-2020.pdf`
  - sha256 `446464fbce176f99f9cbc18c409a757f5e370ae37bf3e66a53e52d81a44933af`
  - `pdfinfo`: 1쪽
- focused test: `issue_2137_topbottom_float_anchor_saved_fit` pass
- 관련 회귀: `issue_2098_page_bottom_fixed_anchor_vpos0`, `issue_1733`,
  `issue_1750_split_guard_spacing_before`, `issue_1842`, `issue_1891` pass
- 전체 검증:
  - `cargo fmt --check` pass
  - `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` pass
  - `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings` pass

## 리스크

TopAndBottom float 전체가 아니라 “비-TAC 그림/도형 float만 있고 저장 page-last 증거가 있는
단일 줄 앵커”로 제한되어 있다. 표 float는 footer 로직으로 남겨 둔 점이 적절하다.

## 권고

누적 체리픽 PR에 포함하고 admin merge 가능하다.
