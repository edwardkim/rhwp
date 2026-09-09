# PR #6591 review - near-fit top-level TAC table body width

## 접수

| 항목 | 값 |
| --- | --- |
| PR | [#6591](https://github.com/edwardkim/rhwp/pull/6591) |
| 작성자 | `davindev` (`kidsnote/rhwp`) |
| base / head | `devel` / `2050166436301f7dfed5a796d5742601f78b85d7` |
| 변경 규모 | 4 files, +184/-72, 3 commits |
| 관련 이슈 | Closes #6590 |
| merge 결과 | `bd72886c02d301ff796b6b5c55a452a870cf317a`, 2026-09-03T08:04:32Z |
| 최종 GitHub 상태 | `MERGED`; 병합 직전 `MERGEABLE`, `CLEAN` 및 required check 충족 |
| reviewer | `jangster77` review request assigned |

## 변경과 검토 범위

- HWP5 stored-pagination layout에서 최상위 `treat_as_char` 표의 선언 폭이 본문 폭보다 크고 축소율이 0.9 이상이면, 기존 `NestedTableWidthProjection`을 본문 폭으로 적용한다.
- 새 `issue_6590_nearfit_tac_table_body_width` regression 2건을 추가하고, 영향받은 text-overlap baseline 1건과 KTX golden SVG를 갱신한다.
- #6590의 직접 표본은 `samples/basic/BlogForm_BookReview.hwp`이며, `rhwp info --json`상 `hancom-office-2010` 저장 HWP5다. 따라서 한/글 2020 MCP PDF를 기준으로 사용했다.

## 최신 devel 정합

- 검토 시작 전 local `devel`을 `upstream/devel` `d770ef80ed5ccc82a834558355b6786213ca2e05`까지 fast-forward했다.
- PR head는 해당 최신 `devel`의 조상이 아니었지만, detached merge simulation tree에서 충돌 없이 결합됐다.
- `git diff --check`를 통과했고, 원 PR head를 merge 직전에 다시 조회해 `MERGEABLE`, `CLEAN`과 동일 SHA를 확인했다.
- 원 PR은 일반 merge commit `bd72886c02d301ff796b6b5c55a452a870cf317a`으로 병합됐고, local `devel`도 해당 `upstream/devel`까지 fast-forward했다.

## 검증 결과

| 검증 | 결과 |
| --- | --- |
| `rust-test-suite-manifest --prepare/--check` | pass, generated state는 검증 전용 |
| `cargo fmt --all -- --check` | pass |
| native, WASM, workspace, all-targets Clippy와 workspace build | pass |
| #6590 focused regression | pass, 전체 nextest에도 포함 |
| `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --no-fail-fast` | pass, 8,974 passed, 46 skipped, 305.148s |
| `scripts/wasm-pack-locked.sh --target web --out-dir pkg` | pass |
| original PR CI | [CI](https://github.com/edwardkim/rhwp/actions/runs/33713684944), [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/33713684906), [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/33713684621), [Adapter](https://github.com/edwardkim/rhwp/actions/runs/33713684876), [Proptest](https://github.com/edwardkim/rhwp/actions/runs/33713684936) success; WASM Build policy skip |
| merge SHA devel CI | [CI](https://github.com/edwardkim/rhwp/actions/runs/33731398995), [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/33731398842), [Adapter](https://github.com/edwardkim/rhwp/actions/runs/33731399092), [Proptest](https://github.com/edwardkim/rhwp/actions/runs/33731398982), [Close Issues](https://github.com/edwardkim/rhwp/actions/runs/33731398851) success |

CodeQL devel run의 JavaScript/TypeScript, Python, Rust 분석 worker와 preflight는 모두 success였다.

## 시각 증적

### #6590 direct pair, BlogForm_BookReview p1

- 한/글 2020 baseline PDF SHA-256: `38a833d5ca3fdb00517d9d9fe4a6399ac2ebc4c0c529b5e24988cb818ff5a7b6`.
- 기준 PDF: `pdf/BlogForm_BookReview-2020.pdf`.
- visual sweep: SVG/PDF `1/1`, `flagged=0/1`, pixel match `95.41523%`, visual accuracy proxy `37.53051%`.
- 최상위 TAC table (`pi=1`, `ci=0`) bbox `x=30.2, w=468.7`; Body bbox도 `x=30.2, w=468.7`이므로 우단 `498.9px`가 일치했다.
- 낮은 proxy는 PDF와 rhwp의 글꼴/glyph raster 차이가 만든 자동 보조값이며, review PNG에서 표 외곽선과 row geometry는 일치했다.

| asset | stable path |
| --- | --- |
| compare | `mydocs/pr/assets/pr_6591_nearfit_tac_table_20260903/blogform/compare_001.png` |
| overlay | `mydocs/pr/assets/pr_6591_nearfit_tac_table_20260903/blogform/overlay_001.png` |
| review | `mydocs/pr/assets/pr_6591_nearfit_tac_table_20260903/blogform/review_001.png` |

### text-overlap baseline 재확인, issue2559 p21

- PR이 `text_overlap_baseline.tsv`를 `27 -> 28`로 올린 실제 후보를 별도로 점검했다.
- 한/글 2020 baseline PDF SHA-256: `e3093cc14d4653a679ad95df612e55d6741128dd9b74d124ca119b9bd7800498`.
- 기준 PDF: `pdf/1341000_research_report_footnotes-2020.pdf`.
- visual sweep: SVG/PDF `1/1`, `flagged=0/1`, pixel match `93.28221%`, visual accuracy proxy `15.90740%`.
- table 외곽선과 행 구조는 기준 PDF와 일치했고, 2.04px heuristic overlap은 glyph/font raster 차이가 큰 표 내부 후보였다. review PNG에서 셀 경계를 넘어 가려지거나 읽히지 않는 실제 text overlap은 확인하지 못했다.
- 이 결과는 p21 한 장의 직접 판정이며, baseline 증가 사실은 숨기지 않는다.

| asset | stable path |
| --- | --- |
| compare | `mydocs/pr/assets/pr_6591_nearfit_tac_table_20260903/issue2559/compare_021.png` |
| overlay | `mydocs/pr/assets/pr_6591_nearfit_tac_table_20260903/issue2559/overlay_021.png` |
| review | `mydocs/pr/assets/pr_6591_nearfit_tac_table_20260903/issue2559/review_021.png` |

KTX golden은 `samples/KTX.hwp`의 `printMethod=4` N-up 출력이라 physical PDF page와 SVG의 1:1 visual sweep 대상이 아니다. 갱신된 golden은 latest-develop merge simulation의 전체 nextest에서 검증했다.

## 옵션 B 문서 PR과 merge 후 contributor PR comment 계획

- [PDF/SVG Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 링크한다.
- #6590 p1의 실제 `flagged=0/1`, pixel match `95.41523%`, visual accuracy proxy `37.53051%`와 사람의 표 우단 일치 판정을 기록한다.
- proxy는 사람의 최종 판정을 대체하지 않는 자동 일치율 보조값임을 명시한다.
- merge commit SHA에 고정한 raw URL로 위 `blogform/review_001.png`를 표시한다.
- 이 archive review·오늘할일·기준 PDF·stable PNG만 담은 옵션 B 문서 PR을 `devel` 대상으로 별도 생성한다.
- 문서 PR 병합 및 그 merge SHA의 devel CI 성공 뒤에만 `--body-file`로 게시하고 API로 comment body를 재조회한다.
- #6590의 실제 자동 close 여부도 API로 확인하며, 자동 종료됐더라도 같은 merge SHA와 검증 증적의 후속 기록이 없을 때만 한 번 게시한다.

## 최종 판정

## 판정: 승인

- #6590의 직접 HWP/Hancom PDF 증적에서 표 우단이 Body 우단과 일치한다.
- 최신 `upstream/devel` 결합 tree의 lint, focused/전체 Rust regression, WASM build가 통과했고 conflict가 없다.
- 새 text-overlap baseline 1건은 실제 p21 시각 대조에서 cell text의 가시 충돌로 확인되지 않았다. 후보와 한계를 위에 분리 기록했다.
- 병합 전 조건인 최신 PR head의 required CI 상태와 `MERGEABLE`/`CLEAN` 재확인, 작업지시자 승인을 충족해 일반 merge commit으로 병합했다.
- merge SHA devel CI·CodeQL·Adapter·Proptest·Close Issues도 모두 success다. contributor PR comment와 #6590 후속 상태는 옵션 B 문서 PR의 병합·devel CI 성공 뒤에 한 번 처리한다.
