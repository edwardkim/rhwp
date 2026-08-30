---
kind: visual-sweep-record
status: accepted-integration-candidate
canonical: mydocs/manual/verification/visual_sweep_guide.md
last_verified: 2026-08-31
pr: 6485
source_prs: [6413, 6422, 6445, 6447, 6455, 6470, 6479]
code_candidate: f47d5b3586d470c99ed38f155af18175801f3c85
---

# PR #6485 planet6897 통합 visual sweep 기록

## 범위와 기준

이 기록은 #6471을 명시적으로 제외한 PR #6485 code candidate `f47d5b3586d470c99ed38f155af18175801f3c85`에서 직접 만든 시각 증적이다. 기준 PDF와 대표 PNG는 이 통합 PR에 포함돼 있다. PDF는 HWP MCP client의 2020 engine으로 산출했으며 endpoint, token, 환경값은 기록하지 않는다.

일반 fixture 5건은 `scripts/visual_sweep.py`로 claim page를 대조했다. #6300과 #6312는 N-up physical page이므로 logical SVG를 Hancom physical sheet의 확인된 slot과 직접 대조했다. `pixel_match`와 proxy는 전체 fidelity 합격 점수가 아니라 이번 수정 claim의 보조값이며, PNG를 직접 열어 clipping, overflow, 행/줄 분리 결과를 판정했다.

## 기준 PDF와 PNG

| PR | fixture 또는 private original | 기준 PDF SHA-256 | claim scope | stable PNG |
| --- | --- | --- | --- | --- |
| #6413 | `samples/issue6298/copay_cap_tac_table_leading.hwpx` `fa00fae4aec5f80175898734ea4d06b82f4c80df7e32b69a7370328c770ab60f` | `pr6485-issue6298-copay-cap-tac-table-leading-2020.pdf` `3c661ccd22d30c7a50e1e7e2f95a6c56b15606e2d2c614bb6391776947f480c0` | p12 | [PNG](../assets/pr_6485_issue6298_p012_review.png) |
| #6422 | `samples/issue6299/forest_press_wrap_seg_pairs.hwpx` `783476f41f6ba5f79779567cb9882c7ffea953c24d5ea211ba10bb2d365453d5` | `pr6485-issue6299-forest-press-wrap-seg-pairs-2020.pdf` `32ed12fac84f7f83155afa37235df23b29cd5c88103072b34d722df64c0c8545` | p1 | [PNG](../assets/pr_6485_issue6299_p001_review.png) |
| #6447 | private original `156464313`; fixture SHA `6eae8426f82676bee40019fa85f86552fed9e07a95672171fef2ed4979c17861` | `pr6485-issue6300-trade-report-forced-break-object-2020.pdf` `a5e20780244dba528173ce3b35c85657e4c10ccf3751276d203db1b386360437` | logical p17, physical sheet 9 left | [PNG](../assets/pr_6485_issue6300_nup_claim_review.png) |
| #6445 | private original `156721992`; fixture SHA `aa804ca85dba392fad80dd78e3b61e668771984720a0a321a0c0ed694233a918` | `pr6485-issue6312-hidden-original-2020.pdf` `b30e6fe82756af99091768c7597453eae47ced437112eed4475b45cd50a67c03` | logical p1, physical sheet 1 left | [PNG](../assets/pr_6485_issue6312_nup_claim_review.png) |
| #6455 | `samples/issue6442/access_pass_form.hwp` `c00794514373a323749cf5354a572748666d84b15c39172cc9069ff35753603a` | `pr6485-issue6442-access-pass-form-2020.pdf` `b32a32fbe04bdeca83b702bea27317deed6fc1eb8c6b131969acabfbd38fdf58` | p2 | [PNG](../assets/pr_6485_issue6442_p002_review.png) |
| #6470 | `samples/issue6443/research_project_design_form.hwpx` `d3e22d6d0b0c587692244e5a2fbe7de7fa2aa26302607acda977e59e02b78329` | `pr6485-issue6443-research-project-design-form-2020.pdf` `37f5057fb78108c195a0437a2f9887be1da628272dcbb729c449f93dd8319421` | p8 | [PNG](../assets/pr_6485_issue6443_p008_review.png) |
| #6479 | `samples/issue6465/press_release_footer_logos.hwpx` `89ec9065432547ae141328dfb957981a26f9dd1b17b9bba9d721e5ae4fbbf5fa` | `pr6485-issue6465-press-release-footer-logos-2020.pdf` `d5a3db9320675907e92a3407d75ce0726fbe605e5bf19fe9bdc46befd07d1487` | p13 | [PNG](../assets/pr_6485_issue6465_p013_review.png) |

모든 PDF는 `pdf/pr6485-visual/`에 보존했다.

## 직접 판정 결과

| PR | automated result | 사람 판정과 명시한 한계 |
| --- | --- | --- |
| #6413 | p12, 후보 0, `pixel_match=87.92468`, proxy `79.37219` | TAC 표 좌측선과 본문 우측 경계에 clipping/overflow가 없다. |
| #6422 | p1, 후보 0, `91.23962`, `33.36546` | wrap fragment가 겹치거나 행을 이중 소비하지 않는다. |
| #6447 | N-up, `86.17842` | 쟁점 문단의 4개 줄과 주석 block이 보존되고 우측 clip이 없다. footer `-17-`/`-16-` 차이는 forced-break claim 밖의 남은 pagination 차이다. |
| #6445 | N-up, `85.33588` | 앵커 문단과 아래 흐름이 분리됐다. fixture의 BinData placeholder에 따른 header/footer image 차이는 anchor-line claim 밖이다. |
| #6455 | p2, 후보 0, `89.70238`, `40.26297` | 앞·뒷면 카드 4개에 모두 내용이 있고 back-side blank가 없다. |
| #6470 | p8, 후보 0, `90.08795`, `25.08413` | 비용 상세 텍스트가 우측 cell line을 넘거나 clip되지 않는다. |
| #6479 | p13, 후보 0, `93.64131`, `20.50308` | footer logo와 설명 block이 자체 줄에 남아 inline object와 같은 줄을 공유하지 않는다. |

## 로컬 검증

- lint bundle: success
- focused contract tests: #6298 2, #6299 2, #6300 2, #6312 2, #6442 3, #6443 2, #6465 1 passed
- `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 8 --no-fail-fast`: 8,785 passed, 43 skipped
- Native Skia: library 165 passed, focused 7 passed
- WASM package build: success

## 최종 판정

| 원 PR | 판정 | 통합 commit |
| --- | --- | --- |
| #6413 | 승인 | `9fbc0c092` |
| #6422 | 승인 | `bc695cd6c` |
| #6445 | 메인터너 보정 후 수용 가능 | `cd9412d7e` + `698d17e56` |
| #6447 | 승인 | `03c1d494c` |
| #6455 | 승인 | `8f384e51b` |
| #6470 | 승인 | `e476c9625` |
| #6479 | 승인 | `3d8109708` |

#6471은 이 PR의 source, PDF 판정, 수용 및 후속 처리 대상이 아니다. 위 판정은 PR #6485의 최신 trailing head CI와 명시적 merge 승인까지를 조건으로 한다.
