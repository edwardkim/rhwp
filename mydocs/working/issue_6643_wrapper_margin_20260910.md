# #6643 블록 래퍼 표 수평 여백 보정

## 현재 상태

- 작업일: 2026-09-10, Ubuntu Linux, 96 DPI, webfont rasterizer.
- 기준 commit: `a3cd825c23d550e0c5f46b4e2eb3735a537f23f2` (`upstream/devel` 동기화 기준).
- 브랜치: `fix/6643-wrapper-margin-20260910`.
- 코드 commit: `7415b6f1d3931ea212a719232a3404401ad2d88c`. `Column | Para` 기준의 원본 HWP 블록 래퍼에 왼쪽 바깥 여백 적용. 재귀에서 부모가 이미 적용한 여백은 재가산하지 않는다. 기존 inline/depth/profile/offset 가드는 유지한다.
- 판정: 수평 보정의 직접 실측 및 기존 회귀 통과. 세로 오차와 페이지 내용 배치 차이는 남아 있어 #6643 전체 해결 판정은 아니다.
- 코드 commit과 upstream push 후 [PR #6979](https://github.com/edwardkim/rhwp/pull/6979)를 생성했다. 검토 기록과 대표 PNG는 같은 PR의 문서 후속 commit으로 포함한다. GitHub CI 완료, 작업지시자의 최종 시각 승인, merge, GitHub comment, issue 종료는 아직 완료하지 않았다.
- [PR #6979 self-review](../pr/archives/pr_6979_review.md)에 검증 범위, 잔여 문제와 merge 후 코멘트 계획을 기록한다.

## 이슈 선택 기록

jeong-sik의 열린 이슈를 오래된 순서로 확인했고, 실제 선행 PR이 처리 중인 범위는 중복 구현하지 않았다.

| 이슈 | 선행 처리 | 이번 처리 |
| --- | --- | --- |
| #6389 | #6412/#6484 및 통합 #6486/#6490 | 선행 구현 제외, 잔여 문제 해결을 의미하지 않음 |
| #6611 / #6623 | 열린 #6670이 같은 원인 처리 | 중복 구현 제외 |
| #6643 | 관련 언급은 있으나 실제 수정 PR은 없음 | 첫 보정 대상 |

## 검증본 식별

아래는 commit 전 최종 검증에 사용한 값이며, 동일한 source를 코드 commit `7415b6f1d3931ea212a719232a3404401ad2d88c`으로 고정했다. 기준 commit과 검증 바이너리를 구분하며, GitHub CI 결과는 로컬 검증과 별도로 기록한다.

| 대상 | SHA-256 |
| --- | --- |
| `src/renderer/layout/table_layout.rs` | `a41a159979a49fddc70f63208cfb870ef3e1acb48d4828bf619dec9e4b56b7c9` |
| `target/pr-review/debug/rhwp` (최종 workspace 빌드 / sweep) | `7d3d9d57dc648d21c1ce2ea8162823cbfe1156f3c997c3995a9223fcfdaba578` |
| `samples/80168_regulatory_analysis.hwp` | `c8ad10fe9f07be5119cd804278017aefa46e555bbee4f05f0f5132fe4f591a22` |
| `pdf/80168_regulatory_analysis-2022.pdf` | `7af457d9ec502132b1035582c16b1ba783e7faff71da0feef202690382bb1b95` |
| `scripts/visual_sweep.py` | `d367dba28c3620d5d2bba6d236a443597c2710c3b7c2efa06611f087708b94e8` |

최초 단독 CLI 빌드의 바이너리 SHA-256은 `0ece2438b3bd6fe81fd44322e04fbccf65e41d8bb9660eec69b5ce6dfededa44`였다. 최종 workspace 빌드와 구분한다.

## 원본과 기준 PDF

- 출처: [이슈 #6643](https://github.com/edwardkim/rhwp/issues/6643)이 지목한 기존 저장소 fixture와 한컴 기준 PDF.
- 원본: `samples/80168_regulatory_analysis.hwp`, SHA-1 `bebce726906daadaef21d27df0de1c6178c6ae5d`.
- 기준: `pdf/80168_regulatory_analysis-2022.pdf`, SHA-1 `ee43260d2512898ab746fbdcefab8490e8f4831c`.
- `Creator: Hwp 2022 12.0.0.4547`, `Producer: Hancom PDF 1.3.0.550`, PDF 1.4.
- 157쪽, 595 x 841pt (A4). 기존 파일을 재사용했으며 새 PDF 변환이나 동일 PDF 복사는 하지 않았다.
- 원본과 기준 PDF는 이번 정리에서 삭제하거나 이동하지 않았다.

## 실행 명령과 결과

같은 `target/pr-review`를 재사용하며 순차 실행했다.

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
cargo fmt --all -- --check
cargo clippy --locked --target-dir target/pr-review -- -D warnings
cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown --target-dir target/pr-review -- -D warnings
cargo build --locked --workspace --target-dir target/pr-review
cargo clippy --locked --workspace --all-targets --target-dir target/pr-review -- -D warnings
node scripts/rust-test-suite-manifest.mjs --check
cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast
```

- 모든 단계 exit 0. prepare/manifest: 1,241 sources, 48 integration targets.
- 전체 회귀: **9,383 passed, 46 skipped, 5 slow**, 테스트 실행 506.946초 (컴파일 제외).
- nextest의 `profile.ci-duration-observation.junit.report-skipped` 알 수 없는 키 경고는 있었으나 exit 0이었다.
- 기존 #6648 중첩 표 여백 테스트 통과. 테스트 source나 baseline 기대값은 수정하지 않았다.
- #6378 계약도 CLI로 직접 대조했다. `samples/tac-img-02.hwp`는 66쪽, `.hwpx`는 67쪽이며 첫 Table bbox는 둘 다 `(79.4, 119.6, 631.2, 898.2)px`다.
- CLI 대조: 각 입력의 `export-svg --page 0 --json` 및 `export-render-tree --page 0` 결과에 기존 페이지 수/원점 계약을 검사했다. exit 0.

## 시각 비교 명령과 범위

정본: [PDF/SVG visual sweep 가이드](../manual/verification/visual_sweep_guide.md).
아래 output 경로는 실행 당시 경로이며, 중간 산출물 정리 후에는 남아 있지 않다.

```bash
RHWP_BIN=target/pr-review/debug/rhwp timeout 600 venv/bin/python tools/fidelity_compare/fidelity_compare.py 0 156 \
  --source samples/80168_regulatory_analysis.hwp \
  --reference-pdf pdf/80168_regulatory_analysis-2022.pdf \
  --label issue-6643-para-fix --reference-grade 'Hancom 2022 original PDF' \
  --text-only --export-all-svg --layout-ledger \
  --out-dir pdf/issue_6643_20260910/para-fix/fidelity

timeout 600 venv/bin/python scripts/visual_sweep.py \
  --hwp samples/80168_regulatory_analysis.hwp \
  --pdf pdf/80168_regulatory_analysis-2022.pdf --key issue-6643 \
  --pages 6,23,37,50,63,79,93,111,127,146 \
  --rhwp-bin target/pr-review/debug/rhwp \
  --out pdf/issue_6643_20260910/para-fix/sweep --resume
```

- fidelity: PDF/SVG/render tree 모두 157쪽, 요청 157쪽 완료, 누락 0, exit 0.
- sweep: 먼저 6/23/50/63/79쪽을 실행한 뒤 같은 provenance에서 10쪽으로 확장하며 기존 5쪽 checkpoint를 재사용했다. 모두 완료, 누락 0, exit 0.
- 자동 후보 0쪽. 10쪽 평균 pixel match 92.03058%, 평균 visual_accuracy_proxy_percent 8.25289%.
- 이 수치는 폰트/raster 차이와 아래 페이지 내용 불일치도 포함한 자동 결과다. 전체 fidelity 통과나 정확도로 해석하지 않는다.
- 실행 당시 compare/overlay/review 경로: `pdf/issue_6643_20260910/para-fix/sweep/issue-6643/{compare,overlay,review}/`.

## 직접 확인 결과와 한계

| 동일 표 대조 | 기준 x | 수정 전 x | 수정 후 x | 남은 y 차이 |
| --- | --- | --- | --- | --- |
| rhwp 6쪽 / PDF 6쪽 | 79.32 | 77.47 | 79.35 | 881.76 vs 882.71, 약 -0.95px |
| rhwp 146쪽 / PDF 145쪽 | 79.32 | 77.47 | 79.35 | 396.49 vs 404.04, 약 -7.55px |

- 6쪽 수평 오차는 약 1.85px에서 0.03px로 줄었다. 대표 review PNG를 직접 열어 본문과 비교 패널을 확인했다.
- 이전 Column 한정 후보와의 157쪽 SVG 비교에서 선 좌표 변경은 6/23/37/50/63/79/93/111/127/146쪽의 수평 +1.88px뿐이었다. 선 개수, 선의 세로 좌표, SVG text 내용은 유지됐다.
- before는 이전 Column 한정 후보이며, 별도로 빌드한 upstream 기준본이라고 부르지 않는다.
- 146쪽 회귀 의심은 서로 다른 내용을 같은 물리 쪽 번호로 비교한 오판이었다. 동일 표는 PDF 145쪽에 있고, 해당 PDF PNG도 직접 열어 확인했다. 가드 추가 승인 요청은 철회했으며 **LINE_SEG 유무로 보정 대상을 제한하는 코드는 추가하지 않았다**.
- 146쪽 same-number review PNG는 동일 표 대조 증적이 아니므로 대표 asset에서 제외했다.
- 최초 이슈 본문의 y=883.6px는 최신 코멘트에서 정정됐다. 세로 바깥 여백의 일괄/절반 가산은 하지 않았다.
- 자동 후보 0건이나 기존 회귀 통과만으로 #6643 전체 해결 또는 최종 시각 승인을 선언하지 않는다.

## 최종 증적과 정리

정책: [시각·fixture 증적](../manual/pr_review/visual_fixture_evidence.md).

- 대표 PNG는 아래 **1개**만 보존한다. 6쪽 수평 개선과 남은 세로 차이를 직접 보여 주는 패널이다.
- [#6643 6쪽 대표 review PNG](../pr/assets/pr_6979_20260910/pr6979-p006-review.png)
- 원래 경로: `pdf/issue_6643_20260910/para-fix/sweep/issue-6643/review/review_006.png` (정리 후 삭제).
- 최종 경로: `mydocs/pr/assets/pr_6979_20260910/pr6979-p006-review.png`.
- `pdf/issue_6643_20260910/` 및 `output/issue_6643_20260910/`의 이번 작업용 임시 산출물을 제거했다. SVG, render-tree/metric/manifest JSON, TSV, raw PNG, 중복 compare/overlay/review, contact sheet, 실패 후보, 원시 로그가 대상이다.
- 원본 fixture, 기존 기준 PDF, 다른 작업의 자료, `target/pr-review`는 보존한다.
- 종료 코드, pass/skip 수, SHA, 비교 매핑, 지표, 판정과 한계는 위 Markdown에 남겨 임시 JSON/로그에 의존하지 않는다.
- 실제 PR #6979 생성 후 PR 번호를 포함한 안정 파일명으로 이동했다. issue 번호 경로에 중복 PNG를 남기지 않는다.
- 향후 코멘트에는 이 대표 PNG만 사용하고 전체 미해결 범위와 페이지 대응 한계를 함께 설명한다. merge/comment 승인 및 구체적인 게시 계획은 별도이며, 지금 게시하거나 이슈를 닫지 않는다.

## 관련 근거

- [이슈 #6643](https://github.com/edwardkim/rhwp/issues/6643)
- [세로 목표 좌표 정정](https://github.com/edwardkim/rhwp/issues/6643#issuecomment-5514248418)
- [래퍼 중복 여백 실험](https://github.com/edwardkim/rhwp/issues/6643#issuecomment-5514468345)
- [HWPX 계약 및 일괄 가산 실험](https://github.com/edwardkim/rhwp/issues/6643#issuecomment-5515093183)
