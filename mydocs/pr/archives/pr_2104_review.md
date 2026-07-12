# PR #2104 리뷰 — #2098 쪽-하단 고정 틀 앵커 vpos=0 리셋 오독 수정

## 메타

| 항목 | 내용 |
|---|---|
| PR | #2104 |
| 작성자 | `planet6897` |
| 제목 | Issue #2098: 쪽-하단 고정 틀 앵커 vpos=0 리셋 오독 수정 |
| base / head | `devel` / `planet6897:fix/2098-page-bottom-anchor-vpos0` |
| 관련 이슈 | #2098 |
| 규모 | 4 files, +90 / -5 |
| 상태 | 문서 작성 시점 참고값: `MERGEABLE`, `CLEAN`, head `496bda60bbf66f3264ad6ae3762abd013eab84c5` |
| merge | 2026-07-09 21:12 KST, merge commit `f4ab0e2abb1a23eff82d66a7fc224706aea36157` |
| 이슈 상태 | #2098 자동 close 확인: 2026-07-09 21:13 KST |
| 증적 보존 | 옵션 2: 코드 PR merge 후 review 문서, MCP 기준 PDF, visual asset을 docs-only PR #2127로 보존 |

## 변경 범위

- `src/renderer/typeset.rs`
  - `para_is_page_bottom_fixed_table_anchor` 추가.
  - 빈 문단이 `vert=쪽 + valign=Bottom + wrap=자리차지` 표를 안는 경우, 저장 `vpos=0`을 흐름 리셋 신호에서 제외.
  - page-bottom footer fit 경로에서 앵커 `vpos<=0`이면 `prev_body_bottom_vpos`로 본문 흐름 하단을 복원해 배타 영역 fit을 판정.
- `samples/task2098/page_bottom_fixed_anchor_vpos0.hwpx`
  - #2098 합성 재현 샘플.
- `tests/issue_2098_page_bottom_fixed_anchor_vpos0.rs`
  - 전체 1쪽과 표가 1쪽에 남는지 고정.
- `samples/task2098/README.md`
  - 재현 구조와 기대 결과 설명.

## 리뷰 결론

Blocking finding 없음.

PR의 핵심 주장은 확인됐다. PR fixture는 수정 후 rhwp 1쪽이며, HWP 2020 MCP 기준 PDF도 1쪽이다. page-bottom footer frame이 별도 2쪽으로 밀리지 않고 1쪽 하단에 남는 동작도 visual sweep에서 확인했다.

Non-blocking 한계:

- PR 본문에서 언급한 실문서 `36358528`, `36376848` 원본은 PR diff와 현재 로컬 저장소에 없어서 직접 재현하지 못했다.
- 다만 PR에 합성 fixture가 포함되어 있고, 반대 방향 게이트 `36387725` 및 기존 #1611/#1624/#1658 테스트가 통과하므로 이 PR의 merge blocker로 보지는 않는다.
- merge 후 원 PR 코멘트에는 다음부터 페이지 수/시각 검증이 필요한 PR에는 실문서 원본 HWP/HWPX와 기준 PDF를 함께 첨부해 달라는 요청을 남기는 것이 좋다.

## MCP 기준 PDF

원본:

- `samples/task2098/page_bottom_fixed_anchor_vpos0.hwpx`
- SHA-256: `2ad146168e567bae1d10653c08d32f201bd27bf2ed528c07cb484fffbd526cb5`

MCP 출력:

- `pdf/task2098/page_bottom_fixed_anchor_vpos0-2020.pdf`
- SHA-256: `ebebecf1cc6f70d4d1eba03c48adb8e0d50dbee97ef29b84fb709aab670c3e3c`
- MCP job id: `91581299-f9a5-40e6-b815-55cb31b604f1`
- server `run_status`: `0`
- server `validation`: `ok`
- `pdfinfo`: 1 page, A4, PDF 1.7

## 검증

로컬 merge 시뮬레이션:

```bash
git switch -c codex/pr2104-review-20260709 upstream/devel
git merge --no-commit --no-ff local/pr2104
```

결과: 충돌 없음. 검증 후 `git merge --abort`로 정리.

로컬 검증:

```bash
CARGO_INCREMENTAL=0 cargo test --profile release-test \
  --test issue_2098_page_bottom_fixed_anchor_vpos0 \
  --test issue_1611_footer_page_bottom_pagination \
  --test issue_1624_footer_overpush_pagination \
  --test issue_1658_page_bottom_fixed_exclusion
```

결과: 6 tests passed.

```bash
cargo fmt --check
git diff --check
CARGO_INCREMENTAL=0 cargo build
CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings
```

결과: 모두 통과.

PR 본문 회귀값 확인:

- `samples/hwpx/opengov/36387725_footer_page_bottom.hwpx`: 2 pages
- `samples/byeolpyo1.hwp`: 4 pages
- `samples/byeolpyo4.hwp`: 26 pages

GitHub Actions:

- 최신 head `496bda60bbf66f3264ad6ae3762abd013eab84c5`
- Build & Test: pass
- CodeQL: pass
- Render Diff / Canvas visual diff: pass

## Visual Sweep

명령:

```bash
python3 scripts/task1274_visual_sweep.py \
  --key pr2104-2098 \
  --hwp samples/task2098/page_bottom_fixed_anchor_vpos0.hwpx \
  --pdf pdf/task2098/page_bottom_fixed_anchor_vpos0-2020.pdf \
  --page 1 \
  --out output/pr2104_visual
```

결과:

- SVG pages: 1
- PDF pages: 1
- selected pages: `[1]`
- flagged: `0/1`
- pixel match: `99.55488%`
- visual accuracy proxy: `4.79731%`
- frame overflow 후보: 없음
- content bottom delta: `0.0px`

산출물:

- compare: `output/pr2104_visual/pr2104-2098/compare/compare_000.png`
- overlay: `output/pr2104_visual/pr2104-2098/overlay/overlay_000.png`
- review: `output/pr2104_visual/pr2104-2098/review/review_000.png`
- 보존 asset: `mydocs/pr/assets/pr_2104_issue2098_visual_review.png`

주의: 이 synthetic sample은 잉크가 매우 적고, 텍스트 위치/폰트 차이 때문에 `ink_match` 기반 proxy가 낮게 나온다. 자동 후보는 0건이며, PR 판단은 PDF와 rhwp 모두 1쪽이고 footer frame이 1쪽 하단에 남는지에 둔다.

## 후속 처리

- PR #2104는 최신 head 기준 GitHub Actions 통과 상태에서 merge 완료.
- #2098은 PR merge 직후 자동 close 확인.
- review 문서, MCP 기준 PDF, visual sweep 대표 asset은 옵션 2 docs-only PR #2127로 보존한다.
- 원 PR 후속 코멘트에는 검증 결과와 함께, 다음부터 페이지 수/시각 검증이 필요한 PR에는 실문서 원본 HWP/HWPX와 기준 PDF를 함께 첨부해 달라는 요청을 남긴다.
