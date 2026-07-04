# PR #1767 리뷰 - #1663 co-anchored RowBreak 표 orphan control

## 메타

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1767 |
| 작성자 | @kkyu8925 |
| 작성자 구분 | FIRST_TIME_CONTRIBUTOR |
| base/head | `devel` / `fix/1663-orphan-rebased` |
| 문서 작성 시점 head | `3dd698ce73ff2fc12079ddc7b57c41bf9cddf3b0` |
| 문서 작성 시점 상태 | `MERGEABLE`, `BEHIND` |
| 관련 이슈 | #1663 |

작성자가 first-time contributor 이므로, 후속 코멘트는 환영 인사와 함께 세심한 피드백 톤으로 남긴다.

## 변경 범위

- `src/renderer/typeset.rs`
  - 같은 host 문단에 co-anchored 된 후속 `TopAndBottom` + `RowBreak` 표가 fresh page 에 통째로 들어가면 행 단위 분할 대신 다음 페이지로 이월한다.
  - co-anchored 자리차지 표 뒤 문서 말미 빈 문단이 새 빈 페이지를 만들지 않도록 trailing overflow 로 흡수한다.
- `tests/issue_1663.rs`
  - 후속 표 B가 page 0에 orphan row 로 남지 않는지 확인한다.
  - 표 뒤 말미 빈 문단이 blank page 를 추가하지 않아 최종 2페이지인지 확인한다.
- `samples/issue1663_coanchored_float_orphan.hwpx`
  - 비공개 실문서 대신 최소 합성 fixture 를 추가했다.
- `mydocs/plans/task_m100_1663.md`, `mydocs/report/task_m100_1663_report.md`
  - 조사/검증 기록을 추가했다.

## 로컬 검증

최신 `upstream/devel` 기준 별도 브랜치 `local/pr1767-review` 에 PR 실제 커밋을 cherry-pick 했다.

```text
dbf4cf39b Task #1663: 빈-host co-anchored 자리차지 float 표 orphan control 및 표 뒤 말미 빈 문단 흡수
```

검증 결과:

| 항목 | 결과 |
|---|---|
| cherry-pick onto latest `upstream/devel` | 통과, 충돌 없음 |
| `git diff --check upstream/devel...HEAD` | 통과 |
| `cargo fmt --check` | 통과 |
| `env CARGO_INCREMENTAL=0 cargo test --test issue_1663 --test issue_1686` | 통과, 6 passed |
| `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings` | 통과 |

## 시각 검증

메인터너 로컬 기준 PDF:

- `pdf/issue1663_coanchored_float_orphan-2024.pdf`
- PDF 생성 정보: Hwp 2024 13.0.0.3622 / Hancom PDF 1.3.0.550
- PDF page count: 2

실행:

```bash
python3 scripts/task1274_visual_sweep.py \
  --file-target pr1767-issue1663 \
  samples/issue1663_coanchored_float_orphan.hwpx \
  pdf/issue1663_coanchored_float_orphan-2024.pdf \
  --out output/pr1767-visual-review
```

결과:

| target | SVG/PDF pages | flagged | frame | line | column | order | question |
|---|---:|---:|---|---|---|---|---|
| `pr1767-issue1663` | 2/2 | 0/2 | `[]` | `[]` | `[]` | `[]` | `[]` |

페이지별 판정 자료:

| page | 임시 review PNG | 최종 asset PNG | visual_accuracy_proxy_percent | 사람 판정 |
|---:|---|---|---:|---|
| 1 | `output/pr1767-visual-review/pr1767-issue1663/review/review_001.png` | `mydocs/pr/assets/pr_1767_issue1663_visual_review_p1.png` | 0.84914 | page 1에는 선행 표 A만 남아 orphan row 가 없음 |
| 2 | `output/pr1767-visual-review/pr1767-issue1663/review/review_002.png` | `mydocs/pr/assets/pr_1767_issue1663_visual_review_p2.png` | 2.48337 | 후속 표 B가 통째로 page 2에 있고 추가 blank page 가 없음 |

후속 코멘트용 asset 링크 후보:

- `https://raw.githubusercontent.com/edwardkim/rhwp/devel/mydocs/pr/assets/pr_1767_issue1663_visual_review_p1.png`
- `https://raw.githubusercontent.com/edwardkim/rhwp/devel/mydocs/pr/assets/pr_1767_issue1663_visual_review_p2.png`

자동 일치율 보조값은 낮지만, 이 synthetic fixture 는 PDF/rhwp 간 표 위치와 raster 차이가 커서 픽셀 일치율이 낮게 나온다. PR 의 검증 목표는 pixel-perfect geometry 가 아니라 `표 B orphan 방지`와 `trailing blank page 제거`이므로, visual sweep 자동 후보 0/2와 사람 판정 기준으로 통과로 본다.

## 코드 검토 메모

- orphan control 은 `has_preceding_coanchored_float` 조건으로 같은 host 의 두 번째 이후 자리차지 표에 한정되어 있다.
- `table_total <= available` 조건이 있어 한 페이지보다 큰 표를 무한히 이월하는 경로는 피한다.
- trailing empty absorption 은 page 마지막 항목이 co-anchored 자리차지 표인 경우로 좁혀져 있어 일반 빈 줄 pagination 과 단독 anchored 표 흐름을 직접 건드리지 않는다.
- 영향 영역이 `typeset.rs`의 페이지네이션 핵심부이므로, merge 전 최신 PR head 기준 required checks 통과는 계속 필요하다.

## 결론

PR 내용과 #1663 이슈 목표 기준으로는 merge 후보로 판단한다.

다만 문서 작성 시점 PR head 는 `BEHIND` 상태였으므로, maintainer 권한으로 최신 `devel` 기준 충돌 없는 적용을 확인했고 로컬 검증도 통과했다. GitHub PR head 를 최신화한 뒤 required checks 가 통과하면 merge 가능하다.

first-time contributor 에게는 좁은 fixture 와 테스트까지 포함해 범위를 잘 줄여준 점을 먼저 감사 인사로 남기고, pagination 핵심부라 CI 완료를 기다린다는 점을 부드럽게 안내한다.

## 후속 코멘트 요청 사항

merge 또는 review 완료 코멘트에는 다음 내용을 포함한다.

- first-time contributor 이므로 환영/감사 인사를 먼저 둔다.
- 이번 PR은 최소 합성 fixture와 회귀 테스트 범위가 잘 좁혀져 있어 merge 후보로 판단했다는 점을 설명한다.
- 이번 검증에는 메인터너 로컬의 한컴 2024 PDF(`pdf/issue1663_coanchored_float_orphan-2024.pdf`)를 사용했음을 밝힌다.
- 다음부터 렌더링/페이지네이션/시각 정합 PR을 올릴 때는, 시각 대조가 가능하도록 한컴 2020 또는 한컴 2024 등 기준 프로그램에서 저장한 PDF도 함께 업로드해 달라고 정중히 요청한다.
- visual asset 링크는 `devel` merge 후 아래 두 PNG를 사용한다.
  - `https://raw.githubusercontent.com/edwardkim/rhwp/devel/mydocs/pr/assets/pr_1767_issue1663_visual_review_p1.png`
  - `https://raw.githubusercontent.com/edwardkim/rhwp/devel/mydocs/pr/assets/pr_1767_issue1663_visual_review_p2.png`
