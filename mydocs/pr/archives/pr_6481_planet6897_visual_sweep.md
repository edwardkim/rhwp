---
kind: visual-sweep-record
status: superseded-after-6471-removal
canonical: mydocs/manual/verification/visual_sweep_guide.md
last_verified: 2026-08-31
pr: 6481
source_prs: [6413, 6422, 6445, 6447, 6455, 6470, 6471, 6479]
---

# PR #6481 planet6897 통합 visual sweep 기록

> **역사 기록**: PR #6481은 #6471의 CMYK JPEG 증적 부족으로 2026-08-31에 닫혔다. 이 문서는
> 당시 8개 체리픽 후보의 증적을 보존하며, 새 replacement 후보의 수용 근거로 재사용하지 않는다.
> replacement 후보는 #6471의 두 체리픽을 제외하고 별도 PR 번호의 review 기록을 새로 작성한다.

## 범위와 판정 원칙

이 기록은 `review/planet6897-batch-20260830`의 code candidate `de5209d52d20749ec413a996f0c89da0e7af1362`에서 직접 만들었다. 실행 파일은 review 전용 `target/review-planet6897-batch-20260830/release-test/rhwp`이며, 모든 기준 PDF는 HWP MCP client로 수신했다. endpoint, IP, token, 환경 파일은 기록하지 않는다.

일반 fixture 6건은 [PDF/SVG visual sweep 가이드](../../manual/verification/visual_sweep_guide.md#github-merge-comment)의 SVG/PDF compare, overlay, review 절차를 썼다. `#6300`과 `#6312`는 `printMethod=4` N-up이므로 표준 도구의 page-by-page 비교를 강제하지 않았다. 대신 private 원본을 파일명과 문서 식별자로 확인하고, Hancom physical sheet의 실제 slot에 해당 logical SVG를 직접 대조했다.

`pixel_match`와 `visual_accuracy_proxy_percent`는 글꼴/텍스트 raster 차이를 포함한다. 따라서 전체 fidelity 합격 수치가 아니라 이번 PR의 표 경계, row wrap, cell image, 카드, 우측 cell, forced break, footer line claim에 한정해 사람이 review PNG를 열어 판정했다. review PNG의 도구 라벨과 overlay legend는 tofu 또는 잘림 없이 판독됐다.

## 원본 및 기준 PDF provenance

| PR | fixture 또는 private original, SHA-256 | engine | 기준 PDF, SHA-256 | MCP job / PDF pages |
|---|---|---|---|---|
| #6413 | `samples/issue6298/copay_cap_tac_table_leading.hwpx` `fa00fae4aec5f80175898734ea4d06b82f4c80df7e32b69a7370328c770ab60f` | 2020 | `pdf/pr6481-visual/pr6481-issue6298-copay-cap-tac-table-leading-2020.pdf` `3c661ccd22d30c7a50e1e7e2f95a6c56b15606e2d2c614bb6391776947f480c0` | `e0cdaade-135d-4a2b-bce6-478480bb97c8` / 12 |
| #6422 | `samples/issue6299/forest_press_wrap_seg_pairs.hwpx` `783476f41f6ba5f79779567cb9882c7ffea953c24d5ea211ba10bb2d365453d5` | 2020 | `pdf/pr6481-visual/pr6481-issue6299-forest-press-wrap-seg-pairs-2020.pdf` `32ed12fac84f7f83155afa37235df23b29cd5c88103072b34d722df64c0c8545` | `93e6e681-e930-4924-a8ee-2f284819bad1` / 1 |
| #6447 | private original `156464313`, repo fixture와 동일 `6eae8426f82676bee40019fa85f86552fed9e07a95672171fef2ed4979c17861` | 2020 | `pdf/pr6481-visual/pr6481-issue6300-trade-report-forced-break-object-2020.pdf` `a5e20780244dba528173ce3b35c85657e4c10ccf3751276d203db1b386360437` | `7057cb0b-9248-46ad-a385-d232161bd411` / logical 40, physical 20 |
| #6471 | `samples/issue6310/press_release_cell_logo.hwpx` `67cf6aa2f6042cd0c743c7ae03d46cc97a0c398df331606c98b26dacc6c62f1e` | 2020 | `pdf/pr6481-visual/pr6481-issue6310-press-release-cell-logo-2020.pdf` `3b6ffbfd48889687076514b0fa367f547b1d73858c63e513277d906ff9562626` | `dfa7b4a3-962f-4a78-99ca-9829a6c523e0` / 51 |
| #6445 | private original `156721992` `ae51668bed34f06822ef83e35f97a7fcb54daf7256ab9213e224244276d33893`; repo fixture는 image-only slice `aa804ca85dba392fad80dd78e3b61e668771984720a0a321a0c0ed694233a918` | 2020 | `pdf/pr6481-visual/pr6481-issue6312-hidden-original-2020.pdf` `b30e6fe82756af99091768c7597453eae47ced437112eed4475b45cd50a67c03` | `403e7868-fc8b-49ca-a44d-07b99308482d` / logical 4, physical 2 |
| #6455 | `samples/issue6442/access_pass_form.hwp` `c00794514373a323749cf5354a572748666d84b15c39172cc9069ff35753603a` | 2020 | `pdf/pr6481-visual/pr6481-issue6442-access-pass-form-2020.pdf` `b32a32fbe04bdeca83b702bea27317deed6fc1eb8c6b131969acabfbd38fdf58` | `24f41fe2-52bb-4bfe-8233-10e42faa09d5` / 3 |
| #6470 | `samples/issue6443/research_project_design_form.hwpx` `d3e22d6d0b0c587692244e5a2fbe7de7fa2aa26302607acda977e59e02b78329` | 2020 | `pdf/pr6481-visual/pr6481-issue6443-research-project-design-form-2020.pdf` `37f5057fb78108c195a0437a2f9887be1da628272dcbb729c449f93dd8319421` | `4c0eb986-7ba3-4917-a24e-051d5823326e` / 8 |
| #6479 | `samples/issue6465/press_release_footer_logos.hwpx` `89ec9065432547ae141328dfb957981a26f9dd1b17b9bba9d721e5ae4fbbf5fa` | 2020 | `pdf/pr6481-visual/pr6481-issue6465-press-release-footer-logos-2020.pdf` `d5a3db9320675907e92a3407d75ce0726fbe605e5bf19fe9bdc46befd07d1487` | `1bbb1b6c-04ea-4922-9a5c-a1b6e7da891f` / 13 |

## 일반 physical-page sweep

공통 명령은 다음과 같다. 실제 temporary root는 `/tmp/rhwp-pr6481-visual/<issue>/`이고, `--rhwp-bin`을 명시해 다른 build target이 섞이지 않게 했다.

```bash
venv/bin/python scripts/visual_sweep.py \
  --rhwp-bin /Users/tsjang/rhwp/target/review-planet6897-batch-20260830/release-test/rhwp \
  --key <key> --hwp <fixture> --pdf <baseline-pdf> --page <page> \
  --out /tmp/rhwp-pr6481-visual/<issue>
```

| PR | reviewed scope / 자동 후보 | `pixel_match` / proxy | 사람 판정 | temporary review / stable asset |
|---|---|---|---|---|
| #6413 | p12; column line-band 2, claim 후보 0 | 87.44435 / 78.80165 | 2 후보는 table/text band raster grouping이다. TAC 표 좌측선과 본문 우측 경계에 clipping/overflow가 없다. | `/tmp/rhwp-pr6481-visual/issue6298/pr6481-issue6298/review/review_012.png`; `mydocs/pr/assets/pr_6481_issue6298_p012_review.png` |
| #6422 | p1; 0 | 91.23962 / 33.36546 | wrap fragment가 겹치거나 한 행을 이중 소비하지 않는다. | `/tmp/rhwp-pr6481-visual/issue6299/pr6481-issue6299/review/review_001.png`; `mydocs/pr/assets/pr_6481_issue6299_p001_review.png` |
| #6471 | p1; 0 | 95.54450 / 20.79271 | Zoom image는 cell 안에 남고 tile/collapse가 없다. CMYK JPEG visual proof는 아니다. | `/tmp/rhwp-pr6481-visual/issue6310/pr6481-issue6310/review/review_001.png`; `mydocs/pr/assets/pr_6481_issue6310_p001_review.png` |
| #6455 | all 3, p2 reviewed; p2 0 | 89.70238 / 40.26297 | 앞·뒷면 카드 4개가 모두 내용이 있고, 이번 결함인 뒷면 카드 공백이 없다. | `/tmp/rhwp-pr6481-visual/issue6442/pr6481-issue6442/review/review_002.png`; `mydocs/pr/assets/pr_6481_issue6442_p002_review.png` |
| #6470 | all 8, p8 reviewed; p8 0 | 90.08795 / 25.08413 | 비용 상세 텍스트가 우측 cell line을 넘거나 clip되지 않는다. | `/tmp/rhwp-pr6481-visual/issue6443/pr6481-issue6443/review/review_008.png`; `mydocs/pr/assets/pr_6481_issue6443_p008_review.png` |
| #6479 | all 13, p13 reviewed; p13 0 | 93.64131 / 20.50308 | footer logo와 설명 block이 자체 줄에 있어 inline object와 같은 줄을 공유하지 않는다. | `/tmp/rhwp-pr6481-visual/issue6465/pr6481-issue6465/review/review_013.png`; `mydocs/pr/assets/pr_6481_issue6465_p013_review.png` |

## N-up original physical-sheet 판정

N-up 비교에는 `visual_sweep.py`를 쓰지 않았다. `export-svg --compat 2022 --font-style`로 candidate logical page를 만든 뒤 144dpi A4 portrait canvas에 rasterize하고, Hancom N-up PDF physical sheet에서 텍스트로 확인한 slot을 잘라 compare/overlay/review PNG로 만들었다. temporary root는 `/tmp/rhwp-pr6481-visual/nup-direct-20260831/`이다.

| PR | logical page -> Hancom physical slot | `pixel_match` | 사람 판정과 한계 | temporary review / stable asset |
|---|---|---:|---|---|
| #6447 | `export-svg -p 17` -> physical sheet 9 left; `농수산식품` 텍스트로 slot 확인 | 86.17842 | 대상 문단이 4개 줄과 주석 block으로 Hancom slot과 같이 분리돼 있고, 빈 줄 뒤 과긴 line 또는 우측 clip이 없다. candidate footer `-17-`과 Hancom footer `-16-`의 차이는 이 PR이 명시한 남은 전체 pagination 1쪽 차이이며 forced-break claim과 분리했다. | `/tmp/rhwp-pr6481-visual/nup-direct-20260831/issue6300/review_nup_claim_page.png`; `mydocs/pr/assets/pr_6481_issue6300_nup_claim_review.png` |
| #6445 | `export-svg -p 0` -> hidden original physical sheet 1 left | 85.33588 | 제목 아래 앵커 문단이 별도 줄을 보존하며 아래 흐름에 붙지 않는다. repo fixture는 BinData를 placeholder로 치환한 slice라 header/footer image fidelity는 이 판정 범위 밖이다. | `/tmp/rhwp-pr6481-visual/nup-direct-20260831/issue6312/review_nup_claim_page.png`; `mydocs/pr/assets/pr_6481_issue6312_nup_claim_review.png` |

## 최종 PR별 판정

| 원 PR | 판정 | 근거와 처리 방식 |
|---|---|---|
| #6413 | 승인 | TAC 표 경계의 계약과 p12 직접 시각 증적에 차단 finding이 없다. #6481 통합 결과로만 수용한다. |
| #6422 | 승인 | line-wrap/행 소비 계약과 p1 직접 시각 증적에 차단 finding이 없다. #6481 통합 결과로만 수용한다. |
| #6445 | 메인터너 보정 후 수용 가능 | 원 head는 충돌하지만 `de5209d52d20749ec413a996f0c89da0e7af1362` 보정의 N-up physical-sheet 증적과 focused 회귀가 충족됐다. |
| #6447 | 승인 | forced-break 대상 문단의 N-up physical-sheet 직접 판정과 계약 검증에 차단 finding이 없다. #6481 통합 결과로만 수용한다. |
| #6455 | 승인 | 카드 공백 결함의 p2 직접 시각 증적과 계약 검증에 차단 finding이 없다. #6481 통합 결과로만 수용한다. |
| #6470 | 승인 | 비용 상세 열의 p8 직접 시각 증적과 계약 검증에 차단 finding이 없다. #6481 통합 결과로만 수용한다. |
| #6471 | 머지 보류 | CMYK JPEG 정규화는 독립 Hancom PDF 증적이 없다. 이를 보완한 메인터너 검증 뒤에만 `메인터너 보정 후 수용 가능`으로 재분류한다. |
| #6479 | 승인 | footer/logo line claim의 p13 직접 시각 증적과 계약 검증에 차단 finding이 없다. #6481 통합 결과로만 수용한다. |

따라서 #6481 전체는 #6471이 머지 보류인 동안 merge 권고를 내리지 않았고, 해당 PR은 닫혔다.
replacement 후보에서는 #6471을 체리픽하지 않으며, 새 후보의 source PR 목록·commit SHA·검증 결과는
새 PR 번호의 review 기록으로 다시 작성한다.

대표 asset과 기준 PDF는 아직 local review branch에만 있고 원격 PR #6481에는 push하지 않았다. 승인 뒤 asset을 포함한 commit을 push한 경우에만 merge 후 contributor PR comment에 [Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)과 merge-SHA 고정 raw asset URL을 `--body-file`로 한 번 게시하고 API로 재조회한다.
