# PR #2045 self-review — #2019 PDF 기준 부동 폼 페이지네이션 보정

- PR: [#2045](https://github.com/edwardkim/rhwp/pull/2045)
- 작성자: `jangster77`
- base: `devel`
- head: `task/2019-absolute-extent-pagination`
- 관련 이슈: #2019
- 관련 PR: #2035
- 검토일: 2026-07-08

## 1. 범위

이 self PR 은 #2035 의 전면 revert 대신, #2019 부분 완화를 유지하면서 MCP/Hancom 2020 PDF 를 기준으로 실제 시각 오정렬을 보정한다.

핵심 변경:

- Paper/Page 기준 비-TAC 글상자 내부 stored `LINE_SEG.vertical_pos` 에서 object origin 을 빼 이중 vertical offset 을 제거한다.
- 단나누기 뒤 Paper overlay tail 이 page-local body 하단을 넘는 경우에만 새 page 로 넘긴다.
- 부동 overlay 사이 ColumnDef-only 구분자를 컬럼 진행에서 제외한다.
- 빈 앵커에 매달린 Paper/Page 기준 Square 표는 선언 y 에 절대 배치하고 flow cursor 를 전진시키지 않는다.
- 해당 절대 Square 표 앞뒤의 쪽나누기/ColumnDef/Bookmark, ColumnDef-only 스캐폴드 문단은 PDF 기준 별도 빈 줄로 렌더하지 않는다.

추가 기준 자료:

- `samples/hwpx/issue2019_floating_form_74312.hwpx`
  - #2035 에서 삭제 대상으로 제시된 동일 HWPX fixture. `devel` 보존 파일이며 sha256 `092651a9cd41b9ece41abf0b4431a0e5c0a07d811ebb88375d208a1c92ba3d54`.
- `pdf/issue2019/issue2019_floating_form_74312-2020.pdf`
- `mydocs/plans/task_m100_2019_impl_v3.md`
- `mydocs/working/task_m100_2019_stage3.md`
- `mydocs/report/task_m100_2019_report.md`

## 2. #2035 판단

#2035 는 `8c46ca2` 전면 revert 를 제안한다. 검토 결과 기존 수정은 81쪽 산란을 18쪽으로 줄인 부분 완화였으므로, 그대로 revert 하면 더 큰 회귀가 되살아난다.

메인터너 판단은 다음과 같다.

- #2035 의 문제 제기는 맞다. 18쪽만 맞고 PDF 기준 시각 정합이 남아 있었다.
- 그러나 전면 revert 는 적절하지 않다.
- 이 PR 로 PDF 기준 시각 오정렬을 v3 보정하고, #2035 는 superseded 처리하는 것이 맞다.

## 3. 검증

로컬 검증:

```text
cargo build --bin rhwp
CARGO_INCREMENTAL=0 cargo test --test issue_2019_floating_form_overpagination -- --nocapture
target/debug/rhwp dump-pages samples/hwpx/issue2019_floating_form_74312.hwpx
cargo test --profile release-test --test visual_roundtrip_baseline visual_baseline_all_samples -- --nocapture
CARGO_INCREMENTAL=0 cargo test --profile release-test --tests
cargo fmt --check
git diff --check
wasm-pack build --target web --out-dir pkg
node WASM smoke: wasm pageCount=18
```

PDF visual sweep:

```text
python3 scripts/task1274_visual_sweep.py \
  --key issue2019 \
  --hwp samples/hwpx/issue2019_floating_form_74312.hwpx \
  --pdf pdf/issue2019/issue2019_floating_form_74312-2020.pdf \
  --out output/task2019_v3_visual_scaffold_skip2
```

결과:

- `dump-pages`: 18페이지
- `render-diff`: PASS, 18→18, 구조 불일치 0, 최대 변위 0px
- SVG/PDF/render-tree pages: 18/18/18
- visual metrics: `flagged=0/18`
- 대표 육안 확인: `output/task2019_v3_visual_scaffold_skip2/issue2019/compare/compare_005.png`
- p5 좌표: `pi78` bottom 279.5px, table top 281.8px → 본문과 Paper 기준 Square 표 상단 겹침 없음

## 4. 리스크

- 한컴 PDF 와 rhwp 렌더의 글꼴/래스터 굵기 차이는 남아 있다.
- 이번 PR 의 통과 기준은 페이지 귀속, 큰 표 위치, 본문 흐름, 자동 visual sweep 후보 0건이다.
- `pages <= 20` 테스트는 계속 smoke guard 이며, PDF 시각 정합 자체를 보장하는 단위 테스트로 승격하지 않는다.

## 5. 권고

권고: 수용.

merge 전 최종 조건:

- PR head 최신 커밋 기준 GitHub Actions 통과
- 이 review 문서와 구현 계획 문서가 PR diff 에 포함됨
- 작업지시자 최종 승인
