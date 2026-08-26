---
kind: pr-review
status: local-review-complete-pending-push
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-20
---

# PR #5772 검토: HWPX 그림 참조·위첨자 폭·TAC 축소 행 하한

## 판정

- 검토 source head: `81027c78f6bb62a91562016370e008b88d147d60`
- 범위: #5747 HWPX binary item 참조의 단일 패스 정규화, #5756 위·아래첨자 전진 폭 0.7배,
  #5748 TAC 축소 행의 저장 높이 하한 보존
- 코드·focused 검증·실물 HWP 2020 PDF 기반 시각 검토에서 차단 결함을 찾지 못했다.
- 권고: 작업지시자의 시각 판정 승인과 push 뒤 최신 trailing head의 fast-pass aggregate 확인을 전제로 수용 가능.
  Codex의 이미지 확인만으로 시각 판정을 최종 통과로 단정하지 않는다.

## 접수와 최신성

- 작성자: `planet6897` (`fix/bughunt-batch-r3` -> `devel`)
- 검토 시작 및 최종 source SHA는 모두 `81027c78f6bb62a91562016370e008b88d147d60`이다.
- 기준 `upstream/devel`: `053ac6984206e91911bfae014069d5a7b30fc830`.
- 외부 기여자 source commit은 rebase·amend·force-push하지 않는다. 증적은 source head 뒤의
  collaborator trailing commit으로만 추가하며, 최신 `devel` 호환성은 merge simulation으로 확인한다.

## 코드 검토

### #5747: binary item 참조 정규화

`src/parser/hwpx/mod.rs`의 `canonicalize_bin_item_refs`는 원본 binary item id와 순번 기반 image id의
대응표를 먼저 만든 뒤 각 `binaryItemIDRef`를 한 번만 치환한다. 연쇄 치환으로 뒤 image가 다시 바뀌는
기존 위험을 제거했고, `issue_5747_bin_item_ref_single_pass`는 세 image payload가 각각 자기 marker를
보존하는지 확인한다.

### #5756: 위·아래첨자 advance

`src/renderer/layout/text_measurement.rs`는 위·아래첨자 glyph advance에만 0.7배를 적용하고 tab은
기본 font 폭을 유지한다. `issue_5756_superscript_advance_scale`는 우측 경계가 714.5px 이내인지 확인한다.

### #5748: TAC 축소 행 높이

`src/renderer/height_measurer.rs`는 저장된 lineseg와 padding으로 행별 하한을 만들고 slack 안에서만
부족분을 이동한다. 균일 축소 fallback도 남아 있어 하나의 행이 다른 행의 최소 높이를 침범하지 않는다.
`issue_5748_tac_shrink_row_floor`는 제목 clip 123.81px, 부제 57.93px 및 bottom baseline의 행 내부 위치를
확인한다.

## 로컬 검증

| 검증 | 결과 |
| --- | --- |
| `cargo fmt --all -- --check` | 통과 |
| `node scripts/rust-test-suite-manifest.mjs --prepare` 및 `--check` | 통과 (820 sources, 3,998 attrs) |
| `node scripts/rust-unit-test-tiers.mjs --check` | 통과 (4,225 tests, 299 modules) |
| #5747 focused nextest | 1 passed, 134 skipped |
| #5756 focused nextest | 1 passed, 129 skipped |
| #5748 focused nextest | 1 passed, 115 skipped |
| `ir_field_sweep_baseline` | 4 passed, fixture와 일치 (81.936s) |
| `overflow_cell_baseline` | 1 passed (22.33s), line growth 없음 |
| `cargo build --locked --profile release-test --target-dir target/pr-review` | 통과 |

같은 source SHA의 GitHub [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/32373764028/job/96443258555),
[Lint](https://github.com/edwardkim/rhwp/actions/runs/32373764028/job/96440170990),
[Native Skia](https://github.com/edwardkim/rhwp/actions/runs/32373764028/job/96440170953),
[Canvas visual diff](https://github.com/edwardkim/rhwp/actions/runs/32373763533/job/96440188327),
[CodeQL Rust](https://github.com/edwardkim/rhwp/actions/runs/32373763625/job/96440211019)를 포함한 required check가
성공했다. 따라서 full release-test와 native-Skia 재실행은 workflow 3.2.2의 동일 code candidate 재사용
조건에 따라 생략했다.

Docker는 이 검토 호스트에 없어 표준 Docker WASM 경로는 실행하지 못했다. 대신
`wasm-pack build --target web --out-dir pkg --no-opt`의 native 진단 build는 4분 44초에 통과했고,
실물 #5747의 0·1·7쪽, #5748의 0쪽, #5756의 2쪽 native-WASM SVG parity는 모두 mismatch 0이었다.
이는 Docker 표준 검증 통과를 뜻하지 않는 대체 근거다.

## 시각·fidelity 증적

비공개 기준 HWP/HWPX 원본은 저장소에 넣지 않았다. 아래 SHA-256은 재현에 사용한 원본 식별값이며,
MCP endpoint와 인증 정보는 기록하지 않는다.

| 대상 | 기준 원본 SHA-256 | HWP 2020 PDF | 결과 |
| --- | --- | --- | --- |
| #5747 | `f7bd70a3ec5a81a6d4a35eea22062c154d0ef171d11f762b6438ec546c6b932d` | [`pdf/pr_5772_issue5747_156532835_hancom2020.pdf`](../../../pdf/pr_5772_issue5747_156532835_hancom2020.pdf), 20쪽, `c2e24ab892c5dbe3b30ff50dfd98fef8070fa6cfdae1d85298530f3ed6240bd2` | MCP `success`/`run_status=0`/`validation=ok`; 20/20쪽 대조 |
| #5748 | `3f39620d8427fe0f28feeb440437549c8ae175b73ebef784e961ed5542169062` | [`pdf/pr_5772_issue5748_156682735_hancom2020.pdf`](../../../pdf/pr_5772_issue5748_156682735_hancom2020.pdf), 3쪽, `41d182a0a46880dc8614508de9a5a35158fc7d3fa48a6566226990e2e0e5b3a0` | MCP `success`/`run_status=0`/`validation=ok`; 3/3쪽 대조 |
| #5756 | `f354fdc246a0b334c34af1bd1916185a23eb7cc729c74e57407a7ea35f65b422` | 기준 PDF 미채택 | MCP 결과가 1쪽 호환성 경고만 포함해 시각 기준으로 쓰지 않았다. |

임시 출력은 `output/visual_sweep_pr5772`와 `output/fidelity_pr5772_issue5747`에 남겼고, 결론에 사용한
대표 PNG와 ledger만 아래 안정 경로로 보관했다.

| 검토 대상 | 대표 asset과 사람 검토 결론 |
| --- | --- |
| #5747 p1 | [`p001 PNG`](../assets/pr_5772_issue5747_p001_review.png): 자동 `line_order_overlap` 1건은 font/position 차이 후보였고, government logo와 그림 참조 오배선은 보이지 않았다. |
| #5747 p2 | [`p002 PNG`](../assets/pr_5772_issue5747_p002_review.png): page sequence와 그림 배치가 유지됐다. pixel match 96.62248%, visual accuracy proxy 65.94612%. |
| #5747 p8 | [`p008 PNG`](../assets/pr_5772_issue5747_p008_review.png): 다수 그림이 서로 바뀌지 않았고, visual accuracy proxy 47.86624%는 글꼴·layout 차이 후보로 기록했다. |
| #5748 p1 | [`p001 PNG`](../assets/pr_5772_issue5748_p001_review.png): 제목 `수출 개척 지원`의 세 번째 줄이 표 cell 내부에 남았다. 자동 후보 0건. |
| #5756 p3 | [`p003 rHWP PNG`](../assets/pr_5772_issue5756_p003_rhwp.png): self-render에서 39.3%→78.8% 위치가 cell border 안에 있다. HWP 2020 기준 PDF가 유효하지 않아 독립 fidelity 판정은 보류한다. |

fidelity page-count ledger는 #5747 20/20, #5748 3/3으로 기준 PDF와 일치한다. #5747의 generic table/text 후보와
#5748의 p3 table footer 후보는 이 PR의 그림 참조·TAC p1 행 하한 주장과 직접 연결되지 않아 차단하지 않았으며,
후속 전면 fidelity 평가는 [#3820](https://github.com/edwardkim/rhwp/issues/3820)에서 다룬다.

- [`#5747 page-count ledger`](../assets/pr_5772_issue5747_page_count_ledger.tsv)
- [`#5747 layout candidates`](../assets/pr_5772_issue5747_layout_candidates.tsv)
- [`#5747 cell boundary candidates`](../assets/pr_5772_issue5747_table_cell_boundary_candidates.tsv)
- [`#5747 SVG text-band candidates`](../assets/pr_5772_issue5747_svg_text_band_clip_candidates.tsv)
- [`#5747 overlay metrics`](../assets/pr_5772_issue5747_overlay_metrics.json)
- [`#5748 page-count ledger`](../assets/pr_5772_issue5748_page_count_ledger.tsv)
- [`#5748 layout candidates`](../assets/pr_5772_issue5748_layout_candidates.tsv)
- [`#5748 overlay metrics`](../assets/pr_5772_issue5748_overlay_metrics.json)

## 남은 절차

이 commit은 source code를 바꾸지 않는 review·증적 trailing commit이다. push 직전 source SHA를 다시 확인하고,
최신 `upstream/devel` merge simulation과 Markdown link 검사를 통과시킨다. push 뒤에는 fast-pass preflight와
branch protection aggregate의 최신 head 결과를 확인한 뒤에만 merge 판단으로 진행한다.
