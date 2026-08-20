---
kind: pr-review
status: local-validation-complete-pending-github-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-20
---

# PR #5770 검토 - renderer bughunt r2 (#5720, #5721, #5727)

## 판정

- 검토 시점 source head는 `182b94007b6840b009d1b00aac6088006be3a9ce`이며, contributor remote
  `fix/bughunt-batch-r2`도 같은 SHA를 가리켰다. `MERGEABLE/CLEAN`과 current `upstream/devel`
  merge-tree clean은 작성 시점 참고값이며, merge 직전에 다시 확인해야 한다.
- reviewer `jangster77`를 assign했다. 외부 contributor PR이고 `maintainerCanModify=true`다.
- 코드의 세 renderer 보정과 추가 회귀 test는 차단 결함을 발견하지 못했다. overflow-cell 원장의
  확인된 감소분은 maintainer commit `b2a5f4920`으로 `tests/fixtures/overflow_cell_baseline.tsv`에
  래칫 반영했다. 이 새 code head는 아직 원격에 push하지 않았으므로, GitHub 전체 CI 상태는 확인 전이다.

## 범위와 CI

- contributor 변경은 renderer layout 4개 파일, HWP/HWPX fixture 3개, regression test 3개, IR
  field-sweep baseline 2행이다. maintainer 보정은 overflow-cell baseline 1개 파일의 감소분 래칫이다.
- #5720은 행 선언 폭이 표 폭과 1% 이내로 어긋나는 경우를 정규화하고, 선언 폭보다 넓어진 전역 fallback
  row를 축소해 표가 용지 밖으로 나가지 않게 한다.
- #5721은 글상자 vpos stream의 첫 유효 line segment가 page origin 이상일 때만 origin 재기저화를 유지해,
  box-relative stream 안의 표 순서 역전을 막는다.
- #5727은 TAC picture가 앞선 빈 composed line에 이미 배정되었을 때 다음 text line이 같은 picture를
  다시 소유하지 않게 하고, cell 경로의 중복 picture emit을 막는다.
- GitHub의 exact source head에서 Full Build & Test, Lint (fmt, clippy, WASM check), Native Skia,
  CodeQL Rust/JavaScript/Python, Canvas visual diff, Proptest, adapter inter-diff와 모든 archive shard가
  성공했다. renderer PR의 Full CI 재사용 조건을 충족하므로 로컬 전체 nextest와 Native Skia 전체는
  중복 실행하지 않았다.

## 로컬 검증

- `cargo fmt --all`, `cargo fmt --all -- --check`, `git diff --check`, Rust suite manifest 및
  unit-test tier check를 통과했다.
- `issue_5720_column_grid_declared_width`, `issue_5721_textbox_vpos_origin_gate`,
  `issue_5727_inline_tac_own_line`, 기존 #5590/#1921/#4287 regression filter를 모두
  `--locked --cargo-profile release-test --target-dir target/pr-review`로 통과했다.
- `ir_field_sweep_baseline`은 generated suite helper로 4/4 통과했고(75.027s), 현재 TSV와 PR의
  `ir_field_sweep_baseline.tsv` diff는 비어 있었다. 보관본은
  `mydocs/pr/assets/pr_5770_ir_field_sweep_current.tsv`다.
- `overflow_cell_baseline`은 보정 전 1/1 통과했다(19.08s). 현재 원장은 기존 baseline 대비
  `hwp3-table-caption.hwp 28 -> 0`, `issue1891/86712_regulatory_analysis.hwpx 2 -> 1`,
  `issue1892_hwp3_drawing_group_roundtrip.hwp 20 -> 0`으로 감소했다. local validation 4.3.1의
  감소/해소 래칫 규칙에 따라 해당 행을 삭제 또는 1로 갱신했다. 보정 후 동일 test는 1/1 통과했고
  (18.92s), 갱신 baseline과 현재 dump의 diff는 비어 있다. 원장은
  `mydocs/pr/assets/pr_5770_overflow_cell_current.tsv`에 보관했다.
- maintainer 보정 뒤 `cargo nextest run --locked --cargo-profile release-test --target-dir
  target/pr-review --tests --test-threads 12 --no-fail-fast`는 214.604s에 8,046/8,046 통과했다
  (slow 3개, skipped 39개). 이 전체 실행은 새 test-data head의 로컬 검증 근거다.
- `wasm-pack build --target web --out-dir pkg`와 시각 대조용
  `cargo build --locked --profile release-test --target-dir target/pr-review`를 통과했다.

## 시각 증적

기준 PDF는 PR에 PDF가 없어 HWP 2020 MCP `PrintToPDFEx`로 만들었다. 세 변환은 모두 CLI status success,
server `run_status=0`, validation ok와 `pdfinfo` 페이지 수를 확인했다. MCP endpoint와 인증 정보는 기록하지
않는다. 픽셀 diff는 후보 검출값이며 최종 판단은 사람 대조다.

| Issue | 입력 SHA-256 | 기준 PDF SHA-256 | MCP job / PDF pages | 검토 | 최종 asset |
| --- | --- | --- | --- | --- | --- |
| #5720 | `7486fa2484fd2370ca27c456ba405bc4153b9d6191a52c9e1f01f5266f8e6a0b` | `0d02b74893774ddb83e69338ca3cf3ef1b5a40e60224d75f62207899b1d8389d` | `86bbea39-807c-4e23-9282-b7cc480ecf92` / 1 | p1, diff 9.83%, reference-only 0; 선언 폭 표가 용지 안에 남음 | `mydocs/pr/assets/pr_5770_issue5720_p001_review.png` |
| #5721 | `d842dffbfdd54fdb18c311d693b49be2ad72bc82c714de86e55972e6dd714c6a` | `9da4dc31d88562779f23a20745488ac0007f3837f992b7f0aacb206fbf98e27a` | `f8b7c70c-0db6-4114-8283-441075f9f2cc` / 1 | p1, diff 5.19%, reference-only 0; 발신처 표가 제목 표보다 위에 유지됨 | `mydocs/pr/assets/pr_5770_issue5721_p001_review.png` |
| #5727 | `4cd2916a4c12c1067b88ce78a7fc019ed3522cc646b671276e820d796056b319` | `ccf287ac8d1c3183abf3ef6f7b41afac55bd4f651032cf342fd0e11c98723c6b` | `9f7c7a0e-7c81-4bf1-8b98-9a734fc2fca3` / 4 | p1, diff 16.45%, reference-only/SVG-only 0; 로고 자기 줄과 다음 text line이 분리됨 | `mydocs/pr/assets/pr_5770_issue5727_p001_review.png` |

- 기준 PDF는 각각 `pdf/pr_5770/issue5720_2734559_hancom2020.pdf`,
  `pdf/pr_5770/issue5721_2568129_hancom2020.pdf`,
  `pdf/pr_5770/issue5727_156732636_hancom2020.pdf`에 보관했다.
- final fidelity report, text report, run state는 각 issue별 `mydocs/pr/assets/pr_5770_issue*_*` TSV에
  보관했다. 임시 `pdf/pr_5770/fidelity/` SVG/PNG export는 대표 asset 복사 뒤 경로 한정으로 제거했다.
- Snap Chromium은 `/tmp`와 hidden cache path에 PNG를 쓸 수 없어 `pdf/pr_5770/`의 사용자 홈 경로에서
  캡처했다. venv의 `pypdf`, `pypdfium2`, `Pillow`를 사용했고, Snap의 DBus 경고는 캡처 결과에 영향을
  주지 않았다.

## 후속 처리

1. 작업지시자 승인 뒤 동일 visibility branch의 maintainer code/test commit `b2a5f4920`을 원 PR source
   branch에 push한다.
2. 새 code head의 focused overflow test와 GitHub Full CI, CodeQL, 필요한 Render Diff가 녹색인지 확인한다.
3. 이 증적 commit은 로컬에 준비된 상태다. 원격 code head가 녹색이 된 뒤 trailing review-only head의
   aggregate 상태를 다시 확인한다.
4. 최종 mergeability와 작업지시자 승인을 다시 확인한 뒤에만 merge 및 contributor/issue comment를 게시한다.
