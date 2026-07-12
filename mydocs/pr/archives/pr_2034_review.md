# PR #2034 리뷰 - 용지 앵커 글뒤로 표 절대배치

- PR: #2034 `Issue #1994: 용지-앵커 글뒤로 표 절대배치 (겹침 소거, 5p->4p)`
- URL: https://github.com/edwardkim/rhwp/pull/2034
- 작성자: `planet6897`
- 작성자 관계: `CONTRIBUTOR`
- 기준 브랜치: `devel`
- head branch: `planet6897:fix/1994-behindtext-table-overlap`
- 문서 작성 시점 참고 head: `a11997ea444736d654db3e8a7cffa41c62a0f610`
- 관련 이슈: #1994
- reviewer: `jangster77` 지정 완료
- merge commit: `e5f9e00864f2b0423c9292fc170a2477b2882504`
- 처리 방식: 옵션 2. 원 코드 PR merge 후 review 문서, 기준 PDF, 대표 visual asset 을 별도 docs-only 후속 PR 로 보존

## 결론

승인 및 merge 완료.

PR 은 #1994 교회 주보 샘플에서 `Paper` 기준 `BehindText` 표가 `RowBreak` 흐름 분할 경로로 빠져
3쪽에서 다른 글뒤로 표와 겹치고 페이지 수가 5쪽으로 늘어나는 문제를 직접 고친다. 새 분기는
비-TAC `Paper` 기준 `BehindText`/`InFrontOfText` 표를 본문 흐름을 밀지 않는 절대 배치 대상으로
처리하며, 기존 #1271 계열의 "Paper 기준 배경성 표는 out-of-flow" 판단과 같은 방향이다.

코드 검토와 로컬 검증, 기준 PDF visual sweep 결과에서는 merge blocker 를 찾지 못했다. PR #2034 는
merge commit `e5f9e00864f2b0423c9292fc170a2477b2882504` 로 병합되었고, #1994 는 GitHub closing keyword 로
자동 close 되었다.

이 PR 의 판단 근거인 issue 댓글 첨부 PDF 는 원 PR diff 에 없으므로, maintainer review artifact 로
저장소에 보존한다.

## 변경 요약

- `src/renderer/typeset.rs`
  - 기존 `TopAndBottom + Paper` 절대 배치 게이트를 `BehindText`/`InFrontOfText`까지 확장한다.
  - `BehindText`/`InFrontOfText` 는 `current_height` 를 sync 하지 않고 `table_total_height=0.0` 으로
    배치해 RowBreak 분할과 흐름 advance 를 피한다.
  - 같은 host 문단에 선행 Paper float 가 있는지 보는 조건도 세 wrap 타입으로 확장한다.
- `tests/issue_1994_behindtext_table_overlap.rs`
  - 페이지 수가 한컴 기준 4쪽인지 확인한다.
  - pi=34 `BehindText + Paper + RowBreak` 표가 `PartialTable` 로 분할되지 않는지 확인한다.
- `samples/basic/issue1994_behindtext_table_20200830.hwp`
  - #1994 재현 HWP fixture.

## 기준 PDF 보존

이슈 댓글 https://github.com/edwardkim/rhwp/issues/1994#issuecomment-4903928794 에 첨부된
`issue_1994.pdf` 를 review 기준본으로 사용했다.

- 원본 첨부 URL: https://github.com/user-attachments/files/29746262/issue_1994.pdf
- PDF 메타: Hwp 2020 11.0.0.9083 / Hancom PDF 1.3.0.550 / 4쪽 / A4 landscape / 302237 bytes.
- SHA-256: `aba4e44b839de113a867e130a01d966bd3bfaf5e1eb048aebbed9ba45365f29e`
- 원본 첨부 보존 경로: `samples/issue1994/issue_1994.pdf`
- visual sweep 기준 PDF 경로: `pdf/issue1994/issue_1994.pdf`
- 임시 다운로드 경로: `output/pr2034/issue_1994.pdf`

`samples/issue1994/issue_1994.pdf`, `pdf/issue1994/issue_1994.pdf`,
`output/pr2034/issue_1994.pdf` 는 같은 SHA-256 이다.

## 시각 검증

`mydocs/manual/visual_sweep_guide.md` 에 따라 PR fixture 와 issue 댓글 첨부 PDF 를 비교했다.

```bash
python3 scripts/task1274_visual_sweep.py \
  --key pr2034-issue1994 \
  --hwp samples/basic/issue1994_behindtext_table_20200830.hwp \
  --pdf output/pr2034/issue_1994.pdf \
  --out output/pr2034_visual
```

| 항목 | 결과 |
|------|------|
| SVG/PDF 페이지 수 | 4 / 4 |
| 자동 후보 | 0 / 4 |
| 평균 pixel match | 85.90012% |
| 평균 visual proxy | 20.27623% |
| 대표 review PNG | `mydocs/pr/assets/pr_2034_issue1994_review_p003.png` |
| 임시 review PNG | `output/pr2034_visual/pr2034-issue1994/review/review_003.png` |

사람 확인: 문제 페이지인 3쪽에서 예배 스케줄 표가 기준 PDF 와 같은 좌측 하단 영역으로 내려가며,
교역자 명단 표와 겹치지 않는다. #1994 댓글에서 언급된 1쪽 하단 문단과 2쪽 오른쪽 단도 대표
review 이미지 기준으로 구조적 이탈은 보이지 않았다. 자동 분석은 flagged page 0건이다.

visual proxy 값은 한컴 PDF 와 rhwp PNG 의 폰트/래스터 차이 때문에 낮게 나온다. 이 PR 의 직접 판단은
페이지 수 4쪽 정합, pi=34 `PartialTable` 미발생, 3쪽 겹침 소거, 1-2쪽 구조 유지 여부를 기준으로 삼았다.

## 검증

- reviewer 지정: `gh pr edit 2034 --repo edwardkim/rhwp --add-reviewer jangster77` 통과.
- target cleanup: cargo 검증 전 `/Users/tsjang/rhwp/target/*` 삭제 완료.
- `cargo fmt --check`: 통과.
- `git diff --check upstream/devel...HEAD`: 통과.
- `CARGO_INCREMENTAL=0 cargo test --test issue_1994_behindtext_table_overlap -- --nocapture`: 통과, 1 passed.
- `CARGO_INCREMENTAL=0 cargo test --test issue_1858_bottom_anchor_flush -- --nocapture`: 통과, 1 passed.
- `CARGO_INCREMENTAL=0 cargo test --test issue_2019_floating_form_overpagination -- --nocapture`: 통과, 1 passed.
- `CARGO_INCREMENTAL=0 cargo test --test issue_2015_saved_bounds_rowbreak -- --nocapture`: 통과, 2 passed.
- `CARGO_INCREMENTAL=0 cargo test --lib test_typeset_703_behind_text_table_no_flow_advance -- --nocapture`: 통과, 1 passed.
- `CARGO_INCREMENTAL=0 cargo build --bin rhwp`: 통과.
- `wasm-pack build --target web --out-dir pkg`: 통과.
- GitHub Actions: PR head `a11997ea444736d654db3e8a7cffa41c62a0f610` 기준 CI, CodeQL, Render Diff 통과.
- 이전 head `c5f6f727804366e4c96285156e2076ef40a46ba3` 의 CI/CodeQL/Render Diff run 은 모두 completed/success 로
  남은 취소 대상 없음.

## 코드 리뷰 메모

- `is_paper_behind_infront` 가 `can_sync` 없이 절대 배치되는 점은 의도된 변화다. `BehindText`/`InFrontOfText`
  Paper 표는 본문 흐름을 밀지 않는다는 기존 #1271 계열 판단과 맞고, 이번 샘플의 `RowBreak` 분할 회귀를
  직접 막는다.
- 새 테스트가 페이지 수와 `PartialTable` 미발생을 고정하므로 이번 회귀의 핵심 증상은 잡힌다.
- 더 넓은 일반화, 예를 들어 host 문단 자체가 다음 페이지로 넘어가는 Paper overlay 표의 anchoring 회귀는
  별도 synthetic fixture 가 있으면 더 강해질 수 있다. 현재 PR 범위에서는 blocker 로 보지 않는다.

## 공개 approve review 기록

PR merge 전에 다음 내용으로 approve review 를 게시했다.

```text
@planet6897 검토했습니다. 이번 PR은 merge 후보로 봅니다.

#1994 샘플에서 `Paper` 기준 `BehindText` 표가 `RowBreak` 흐름 분할로 빠지던 경로를 막아, 기준 PDF와 같은 4쪽 페이지 수로 돌아오고 3쪽 표 겹침이 사라지는 것을 확인했습니다.

확인한 내용은 다음과 같습니다.

- GitHub Actions: CI, CodeQL, Render Diff 통과
- 로컬 검증: `cargo fmt --check`, `git diff --check`, #1994 표적 테스트, #1858/#2019/#2015/#703 관련 회귀 테스트, `cargo build --bin rhwp`, `wasm-pack build --target web --out-dir pkg`
- visual sweep: issue 댓글 첨부 `issue_1994.pdf` 기준 4쪽 모두 비교, 자동 후보 0/4

기준 PDF와 대표 visual review 이미지는 maintainer review artifact 로 별도 후속 PR에 보존하겠습니다.
```

## 후속 코멘트 초안

docs-only PR merge 후 다음 내용으로 #1994 및 PR #2034 에 후속 코멘트를 남긴다.

```text
PR #2034 머지로 #1994 를 처리했습니다.

- merge commit: e5f9e00864f2b0423c9292fc170a2477b2882504
- GitHub Actions: CI, CodeQL, Render Diff 통과
- 로컬 검증: #1994 표적 테스트, #1858/#2019/#2015/#703 영향권 테스트, cargo build, WASM build 통과
- 기준 PDF: issue 댓글 첨부 `issue_1994.pdf` 를 `samples/issue1994/issue_1994.pdf` 와 `pdf/issue1994/issue_1994.pdf` 로 보존
- visual sweep: 4쪽 비교, 자동 후보 0/4

대표 visual review:
![PR #2034 visual review](https://raw.githubusercontent.com/edwardkim/rhwp/devel/mydocs/pr/assets/pr_2034_issue1994_review_p003.png)
```
