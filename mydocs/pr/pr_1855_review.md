# PR #1855 리뷰 — same-para float 과잉 포착 회귀 보정

## 메타

| 항목 | 내용 |
|------|------|
| PR | https://github.com/edwardkim/rhwp/pull/1855 |
| 작성자 | @planet6897 |
| base / head | `devel` / `task1853-float-stack-overcapture-fix` |
| 작성 시점 규모 | 8 files, +454 / -5 |
| 검토 기준 head | `ce7b11a3da43c1e172e83ef104e907e1991faf78` |
| 상태 | Draft 아님, mergeable 참고값 `MERGEABLE` |
| reviewer assign | @jangster77 지정 완료 |

## 관련 이슈

- #1853: PR #1844의 같은 문단 float 스택 통째-이월 규칙이 tac 캡션·페이지-절대 앵커까지
  선행 float 로 오인하여 표 본체를 통째 다음 쪽으로 밀고 +1쪽 회귀를 만든 문제.

## 변경 범위

- `src/renderer/typeset.rs`
  - `preceded_by_same_para_float` 판정에서 단순 `para_index` 비교를 제거하고, 선행 `PageItem`의
    `control_index`로 원본 `para.controls[ci]`를 조회한다.
  - `is_para_topbottom_float(&t.common)`인 표만 선행 flow stack float 로 인정한다.
  - tac 캡션 표와 `vert=용지` 페이지-절대 앵커 표는 선행 float 에서 제외한다.
- `tests/issue_1853.rs`
  - `samples/issue1853_caption_precedes_body_split.hwpx` 기준으로 tac 캡션 페이지에서 본체 표가
    같은 쪽에 split-start 되는지, 총 페이지 수가 52쪽인지 검증한다.
- 문서
  - #1853 계획/구현/보고 문서와 #1844 사후 리뷰 기록을 추가한다.

## 기준 PDF 요청 원칙

페이지 수 검증이나 시각 검증이 PR 판단에 필요한 경우, PR review 문서에는 한컴 2020/2024 등 기준
프로그램에서 저장한 PDF 첨부 요청 여부를 반드시 기록한다.

이번 검증에서는 사용자가 다음 기준 PDF를 추가 제공했다.

- 기준 PDF(repo-relative 기대 경로): `pdf/issue1853_caption_precedes_body_split-2024.pdf`
- PR 샘플: `samples/issue1853_caption_precedes_body_split.hwpx`

기준 PDF는 review worktree 의 `pdf/issue1853_caption_precedes_body_split-2024.pdf` 로 포함했다. 후속
문서/asset PR 또는 PR comment 에도 환경 의존 절대 경로가 아니라 위 repo-relative 경로로 기록한다.

## PR 내용 기준 판단

PR의 핵심 주장은 "tac 캡션을 선행 same-para float 로 오인해 본체 표가 통째 다음 쪽으로 이월되고,
총 페이지 수가 53쪽으로 늘어나는 회귀를 52쪽으로 복원한다"이다.

코드 변경은 이 주장에 직접 대응한다. `PageItem::Table`/`PartialTable`이 같은 paragraph 에 있다는 사실만으로
선행 float 로 보던 기존 판정을, 원본 control 이 실제 `!tac && TopAndBottom && vert=Para`인 경우로 좁힌다.
따라서 tac 캡션과 페이지-절대 앵커는 제외되고, PR #1844에서 의도한 진짜 flow stack 표는 유지된다.

## 로컬 검증

검토 worktree: `/private/tmp/rhwp-pr1855-review`

- `git diff --check upstream/devel...HEAD` 통과
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1853` 통과
  - `body_float_splits_on_caption_page_not_deferred_whole`
  - `caption_over_deferral_does_not_add_a_page`
- `env CARGO_INCREMENTAL=0 cargo build --bin rhwp` 통과
- 기준 PDF 페이지 수:
  - `pdfinfo pdf/issue1853_caption_precedes_body_split-2024.pdf` → 52쪽
- rhwp 페이지 수:
  - `target/debug/rhwp dump-pages samples/issue1853_caption_precedes_body_split.hwpx | rg -c '^=== 페이지'` → 52쪽
- dump-pages 핵심 지점:
  - p44: `Table pi=371 ci=0 tac=true` 캡션과 `PartialTable pi=371 ci=1 rows=0..2` 본체 시작이 같은 쪽에 존재
  - p45: `PartialTable pi=371 ci=1 rows=1..3 cont=true` continuation 존재

## 시각 검증

명령:

```bash
python3 scripts/task1274_visual_sweep.py \
  --key pr1855-issue1853 \
  --hwp samples/issue1853_caption_precedes_body_split.hwpx \
  --pdf pdf/issue1853_caption_precedes_body_split-2024.pdf \
  --pages 44-45 \
  --out output/pr1855_visual \
  --rhwp-bin target/debug/rhwp
```

결과:

- SVG pages: 52
- PDF pages: 52
- selected pages: 44, 45
- visual sweep: `flagged=0/2`
- p44 review: `/private/tmp/rhwp-pr1855-review/output/pr1855_visual/pr1855-issue1853/review/review_044.png`
- p45 review: `/private/tmp/rhwp-pr1855-review/output/pr1855_visual/pr1855-issue1853/review/review_045.png`
- p44 asset: `mydocs/pr/assets/pr_1855_issue1853_review_p044.png`
- p45 asset: `mydocs/pr/assets/pr_1855_issue1853_review_p045.png`
- p44 `visual_accuracy_proxy_percent`: 약 24.27%
- p45 `visual_accuracy_proxy_percent`: 약 11.79%

사람 판정:

- p44에서 기준 PDF와 rhwp 모두 tac 캡션과 본체 표 시작이 같은 쪽에 나타난다.
- p45에서 본체 표 continuation이 이어진다.
- PR의 핵심 회귀인 "본체 표가 캡션 쪽에서 사라지고 통째 다음 쪽으로 밀리는 문제"는 해소된 것으로 판단한다.
- 다만 p44~45의 표 내부 행 경계와 하단/상단 배분은 기준 PDF와 완전히 같지 않다. 이는 본 PR의
  핵심 주장인 +1쪽 회귀 해소와 본체 split-start 복원과는 별도 잔여 시각 차이로 기록한다.

## GitHub CI

update branch 후 검토 기준 head `ce7b11a3da43c1e172e83ef104e907e1991faf78` 문서 작성 시점 참고값:

- CI preflight: 통과
- Render Diff preflight: 통과
- CodeQL preflight: 통과
- CodeQL wrapper: neutral
- WASM Build: skipped
- Analyze python/javascript-typescript: 통과
- Canvas visual diff / Analyze rust / Build & Test: 진행 중

review 문서/PDF/오늘할일 push 후 PR head 가 다시 바뀌므로 최종 merge 전에는 최신 head 기준 상태를 다시
확인해야 한다.

## 결론

PR #1855는 PR이 주장한 #1853 회귀, 즉 tac 캡션/페이지-절대 앵커 과잉 포착으로 본체 표가 통째 이월되어
페이지 수가 +1 되는 문제를 해소한 것으로 판단한다. 기준 PDF와 rhwp 모두 52쪽이고, p44에서 캡션과 본체
표 split-start가 함께 나타난다.

단, p44~45의 세부 행 분배는 기준 PDF와 완전 정합하지 않으므로, 이 차이를 별도 후속 후보로 남긴다.
이 잔여 차이를 본 PR의 merge blocker 로 보지는 않는다.
