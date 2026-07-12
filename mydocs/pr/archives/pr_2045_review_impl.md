# PR #2045 self-review 구현 계획 — #2019 PDF 기준 부동 폼 페이지네이션 보정

- PR: [#2045](https://github.com/edwardkim/rhwp/pull/2045)
- base: `devel`
- head: `task/2019-absolute-extent-pagination`
- 관련 이슈: #2019
- 관련 PR: #2035
- 작성일: 2026-07-08

## Stage 1. 정정과 기준 PDF 보존

- 기존 #2019 보고서의 "완전 일치" 결론을 정정한다.
- `tests/issue_2019_floating_form_overpagination.rs` 를 81쪽 산란 재발 방지 smoke guard 로 명확히 한다.
- #2035 에서 삭제 대상으로 제시된 `samples/hwpx/issue2019_floating_form_74312.hwpx` 를 #2045 기준 원본으로 명시한다.
- MCP/Hancom 2020 변환 PDF 를 `pdf/issue2019/issue2019_floating_form_74312-2020.pdf` 에 보존한다.

## Stage 2. v3 보정 구현

- Paper/Page 기준 글상자 내부 vpos origin 을 정규화한다.
- Paper overlay tail overflow 를 page-local body 기준으로 판단한다.
- 부동 overlay 사이 ColumnDef-only 구분자를 컬럼 진행에서 제외한다.
- Paper/Page 기준 Square 표는 빈 앵커의 flow 를 전진시키지 않고 절대 위치에 그린다.
- 절대 Square 표 주변의 비가시 스캐폴드 문단을 숨긴다.

## Stage 3. 검증

완료된 검증:

```text
cargo build --bin rhwp
CARGO_INCREMENTAL=0 cargo test --test issue_2019_floating_form_overpagination -- --nocapture
target/debug/rhwp dump-pages samples/hwpx/issue2019_floating_form_74312.hwpx
cargo test --profile release-test --test visual_roundtrip_baseline visual_baseline_all_samples -- --nocapture
CARGO_INCREMENTAL=0 cargo test --profile release-test --tests
python3 scripts/task1274_visual_sweep.py --key issue2019 ... --out output/task2019_v3_visual_scaffold_skip2
cargo fmt --check
git diff --check
wasm-pack build --target web --out-dir pkg
node WASM smoke: wasm pageCount=18
```

검증 결과:

- 74312 page count: 18
- render-diff: PASS, 18→18, 구조 불일치 0, 최대 변위 0px
- visual sweep: SVG/PDF/render-tree 18/18/18, `flagged=0/18`
- 대표 5쪽 본문/표 겹침 해소

## Stage 4. PR 처리

1. 이 self-review 문서를 archive 경로에 포함한다.
2. upstream head branch 를 갱신한다.
3. base `devel` 로 Open PR 을 생성한다.
4. PR 번호 확정 후 review 문서 파일명을 `pr_2045_review*.md` 로 정리한다.
5. #2035 에는 이 PR 로 superseded 처리한다는 코멘트 초안을 별도 승인 후 게시한다.

## Merge 전 조건

- GitHub Actions 최신 head 기준 통과
- PR diff 에 review 문서, 기준 PDF, v3 계획/보고 문서 포함
- 작업지시자 최종 승인
