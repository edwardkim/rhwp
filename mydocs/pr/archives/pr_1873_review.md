# PR #1873 Review — HWP3 LINE_SEG rewind 페이지 경계 반영

## 메타

| 항목 | 내용 |
|---|---|
| PR | #1873 |
| 제목 | `HWP3 LINE_SEG rewind 페이지 경계 반영` |
| 작성자 | `jangster77` |
| base | `devel` |
| head | `task/m100-1695-vpos-reset` |
| 관련 이슈 | #1695 |
| review 방식 | 옵션 1: 현재 PR 에 review 문서, visual asset, 오늘할일 포함 |
| update branch 후 코드 head | `6fd39bb70076fc85e7d7445f357edcc7a478d147` |
| 최종 PR head | `3e64bbbfdc1fb3d378ae398906880ca76dc91941` |
| CI 상태 | 통과 |
| merge commit | `e5ff8ab9311ff0a22c127f5a5081c4f1cacc421d` |
| 관련 이슈 close | #1695 auto-close 완료 (`2026-07-03T16:32:57Z`) |

## 변경 범위

- `src/renderer/typeset.rs`
  - 원본 HWP3 source 여부를 `TypesetState` 에 전달한다.
  - 원본 HWP3 텍스트 문단의 저장 `LINE_SEG` 가 페이지 하단부에서 near-top 으로 되감기는 경우만 내부 page break 후보로 본다.
  - 기존 sample16 전용 reset 예외는 유지한다.
  - HWP3 변환본(`is_hwp3_variant`)에는 새 일반 rewind 규칙을 적용하지 않는다.
- `src/document_core/queries/rendering.rs`
  - `DocumentCore` source format 이 HWP3 인지 typeset 경로로 전달한다.
- `tests/issue_1695.rs`
  - `samples/SO-SUEOP.hwp` 의 page count 46 유지 확인
  - `pi=179`, `pi=634`, `pi=748` rewind 경계 분할 확인
  - 분할 전 페이지의 body bottom overflow 방지와 다음 페이지 near-top continuation 확인
- `mydocs/plans/task_m100_1695.md`
- `mydocs/working/task_m100_1695_stage1.md`

## 로컬 검증

| 항목 | 결과 |
|---|---|
| `env CARGO_INCREMENTAL=0 cargo test --test issue_1695 -- --nocapture` | 통과 |
| `env CARGO_INCREMENTAL=0 cargo test --test issue_1035_alignment` | 통과 |
| `env CARGO_INCREMENTAL=0 cargo test --test issue_1692` | 통과 |
| `env CARGO_INCREMENTAL=0 cargo test --test issue_1375_endnote_rewind_column_overflow` | 통과 |
| `env CARGO_INCREMENTAL=0 cargo test --test issue_1733` | 통과 |
| `env CARGO_INCREMENTAL=0 cargo build` | 통과 |
| `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings` | 통과 |
| `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` | 통과 |
| `git diff --check` | 통과 |

full release-test 첫 실행에서는 새 일반 rewind 규칙이 HWP3 변환본까지 열려 `issue_1035_alignment` 의
sample16-hwp5-2022 page count 가 64 에서 65 로 증가했다. 이후 일반 규칙을 원본 HWP3 source 전용으로
축소했고, focused 회귀 및 full release-test 를 다시 통과했다.

## 시각 검증

대상:

- HWP: `samples/SO-SUEOP.hwp`
- 기준 PDF: `pdf/SO-SUEOP-2024.pdf`
- visual sweep command:

```bash
python3 scripts/task1274_visual_sweep.py \
  --key issue1695-so-sueop-hwp \
  --hwp samples/SO-SUEOP.hwp \
  --pdf pdf/SO-SUEOP-2024.pdf \
  --out output/issue1695_after_visual \
  --rhwp-bin target/debug/rhwp
```

결과:

| 항목 | 결과 |
|---|---|
| SVG pages | 46 |
| render-tree pages | 46 |
| 기준 PDF pages | 46 |
| review pages | 46 |
| 자동 flagged pages | 0 / 46 |
| average visual_accuracy_proxy_percent | 15.28125 |
| worst visual_accuracy_proxy_percent | 9.80461 |

대표 산출물:

| page | 임시 review PNG | PR 기록용 asset | visual_accuracy_proxy_percent | 사람 판정 메모 |
|---:|---|---|---:|---|
| 6 | `output/issue1695_after_visual/issue1695-so-sueop-hwp/review/review_006.png` | `mydocs/pr/assets/pr_1873_issue1695_so_sueop_review_p006.png` | 18.04059 | `pi=179` rewind split 이후 하단 overflow 후보 없음 |
| 24 | `output/issue1695_after_visual/issue1695-so-sueop-hwp/review/review_024.png` | `mydocs/pr/assets/pr_1873_issue1695_so_sueop_review_p024.png` | 15.87482 | `pi=634` rewind split 이후 본문 흐름 유지 |
| 29 | `output/issue1695_after_visual/issue1695-so-sueop-hwp/review/review_029.png` | `mydocs/pr/assets/pr_1873_issue1695_so_sueop_review_p029.png` | 14.75988 | `pi=748` rewind split 이후 본문 흐름 유지 |
| 44 | `output/issue1695_after_visual/issue1695-so-sueop-hwp/review/review_044.png` | `mydocs/pr/assets/pr_1873_issue1695_so_sueop_review_p044.png` | 9.80461 | issue 본문에서 언급된 endnote/하단 영역 참고 확인, merge blocker 후보 없음 |

`visual_accuracy_proxy_percent` 는 사람 판정 정확도가 아니라 내용 픽셀 중심 raster 일치율 보조값이다.
SO-SUEOP 전체는 폰트/래스터 차이 때문에 값이 낮지만, 자동 flagged page 는 0 이고 PR 의 핵심 경계 분할
후보에서는 하단 overflow 재현이 보이지 않았다.

## GitHub Actions / update branch

- 최초 PR head `e937e29ec1ce98539752cd2d73b8128e2ea3c291` 은 base 대비 `BEHIND` 상태였다.
- `upstream/devel` 기준 rebase 후 `6fd39bb70076fc85e7d7445f357edcc7a478d147` 로 force-with-lease push 했다.
- 이전 SHA 의 `CI`, `CodeQL`, `Render Diff` run 은 force-cancel 로 `completed/cancelled` 확인했다.
- option 1 문서/asset/오늘할일 커밋 후 최종 head 는 `3e64bbbfdc1fb3d378ae398906880ca76dc91941` 이다.
- 최종 head 기준 `CI`, `CodeQL`, `Render Diff` 모두 통과했다.

## 오늘할일 기록

- `mydocs/orders/20260704.md` 에 #1873 처리 현황을 기록했다.
- option 1 경로이므로 review 문서, visual asset, 오늘할일을 같은 PR branch 에 포함한다.

## 판단

로컬 검증과 visual sweep 기준으로 #1695 의 핵심 문제인 HWP3 `LINE_SEG` vpos rewind 경계 반영은 의도대로
수정됐다. 최종 head CI 통과 후 PR #1873 은 `e5ff8ab9311ff0a22c127f5a5081c4f1cacc421d` 로 merge 완료됐고,
관련 이슈 #1695 는 auto-close 완료됐다.
