# PR #1756 리뷰 — Task #1755 지연 자리차지 표 host 제목 줄 이월 전 쪽 잔류

## PR 메타

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1756 |
| 작성자 | @planet6897 |
| base / head | `devel` / `planet6897:pr/devel-1755` |
| 관련 이슈 | #1755 |
| 문서 작성 시점 참고값 | draft=false, mergeable=CONFLICTING, mergeStateStatus=DIRTY |
| maintainer 수정 | `maintainerCanModify=true` |

## 변경 의도

#1753/#1754에서 남은 한계 항목이다. `samples/task1753/deferred_takeplace_fill_ahead.hwpx`
재현 문서에서 visible-host RowBreak 표의 host 제목 줄이 마지막 fragment 뒤쪽에 렌더되어,
한컴 PDF 기준의 "이월 전 쪽 하단" 흐름과 달라지는 문제를 다룬다.

PR 설명 기준 핵심 목표는 pi=51 host 제목 줄을 9쪽 하단에 먼저 배치하고, 11쪽 fragment 뒤
이중 렌더를 막는 것이다.

## 충돌 원인과 해소

PR #1756은 #1754 위에 스택된 단일 커밋으로 올라와 있다. #1754는 이미 #1810으로 `devel`에
반영되었기 때문에, 현재 PR head를 그대로 merge하면 `src/renderer/typeset.rs`에서 충돌한다.

로컬에서는 `upstream/devel` 기준 새 브랜치에서 PR 단일 커밋을 cherry-pick하여 conflict를
재현했고, 다음 방식으로 해소했다.

- 최신 `devel`에 이미 들어간 #1753 prefill 흐름은 유지했다.
- #1755가 추가한 `pre_emitted_host_paras` 신호만 `TypesetState`, `PaginationResult`,
  `LayoutEngine` 전달 경로에 얹었다.
- `prefill_before_deferred_table` 진입 시 host 텍스트 줄을 `PartialParagraph`로 먼저 배치하고,
  layout fragment 쪽 host 렌더를 억제하도록 유지했다.

## 로컬 검증

- `rm -rf target/*`
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1755_host_heading_pre_emit -- --nocapture`
  - 1 passed
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1753_deferred_table_fill_ahead -- --nocapture`
  - 1 passed
- `cargo fmt --check`
- `git diff --cached --check`
- `env CARGO_INCREMENTAL=0 cargo build`

## 시각 검증

`mydocs/manual/verification/visual_sweep_guide.md` 기준으로 PR 핵심 페이지를 한컴 2024 PDF와 비교했다.

```bash
python3 scripts/task1274_visual_sweep.py \
  --key pr1756-host-heading \
  --hwp samples/task1753/deferred_takeplace_fill_ahead.hwpx \
  --pdf samples/task1753/deferred_takeplace_fill_ahead-2024.pdf \
  --pages 9,11 \
  --out output/pr1756-visual
```

결과:

- SVG/PDF 페이지 수: 21 / 21
- 선택 페이지: 9, 11
- 자동 후보: `0/2`
- review PNG:
  - page 9: `output/pr1756-visual/pr1756-host-heading/review/review_009.png`
  - page 11: `output/pr1756-visual/pr1756-host-heading/review/review_011.png`
- PR 기록 asset:
  - page 9: `mydocs/pr/assets/pr_1756_visual_review_p9.png`
  - page 11: `mydocs/pr/assets/pr_1756_visual_review_p11.png`
- visual_accuracy_proxy_percent:
  - page 9: 12.70671
  - page 11: 6.72881

보조 일치율은 폰트/잉크 픽셀 중심 값이라 낮게 나왔지만, 자동 후보는 없고 review PNG 상
11쪽 host 제목 중복 렌더 후보는 보이지 않는다.

## PR 코멘트 처리 시 PNG 위치

- 코멘트 처리 시 제시할 PNG:
  - `mydocs/pr/assets/pr_1756_visual_review_p9.png`
  - `mydocs/pr/assets/pr_1756_visual_review_p11.png`
- 코멘트 요지: #1754 스택 충돌은 메인터너 권한으로 해소했고, p9에서 host 제목 줄이
  이월 전 쪽에 배치되며 p11에서 마지막 fragment 뒤 host 제목 중복 렌더가 보이지 않는다.
  visual sweep 자동 후보는 p9/p11 기준 `0/2`이다.

## 리스크

- PR 원문에는 #1755 작업 문서가 포함되어 있다. 코드 변경과 테스트는 타당하지만, merge 전 최종
  diff에 문서 포함 범위를 유지할지 별도 정리할지 확인이 필요하다.
- full CI는 conflict-fix push 이후 최신 head 기준으로 다시 확인해야 한다.

## 결론

메인터너 권한으로 conflict 해소 가능한 PR이다. 로컬 focused test와 visual sweep 기준으로는
#1755 핵심 조건이 맞는다. conflict-fix 후보 diff와 검증 결과를 작업지시자에게 공유했고,
승인 후 contributor 브랜치에 push한 뒤 최신 CI 기준으로 merge 여부를 판단한다.
