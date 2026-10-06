---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-22
---

# PR #7320 검토 기록 — 단 맨 위 TAC 표의 저장 `vertical_pos` 보존

- 원 PR: [#7320](https://github.com/edwardkim/rhwp/pull/7320)
- 관련 이슈: [#7203](https://github.com/edwardkim/rhwp/issues/7203) (`Refs`; 자동 종료 대상 아님)
- 작성자: `planet6897` (기존 기여자)
- 원 code head: `7a285a7034449b2ca7d189a33c2851c21effa485`
- 통합 검토 branch: `review/open-prs-20260921`
- 적용 commit: `99c135022a82d2452a617f2d40c76c78ed2df7d5` (`cherry-pick -x`, 충돌 없음)
- 작성 시점 원 PR 상태: `MERGEABLE / CLEAN`, 원 head CI 성공

## 변경과 검토

빈 앵커 문단의 TAC 표가 단 상단에 배치될 때, 첫 `LINE_SEG.vertical_pos`가 실제 쪽-상대
여백을 뜻하는 경우에만 표 시작 y에 더한다. 합성 line segment, 0 이하 값, `spacing_before`를
넘는 누적축 값은 제외한다. 기존 column-top 문단 계약과 같은 상한을 적용하며 일반 표·본문
문단을 건드리지 않는다.

호출 지점은 TAC·비인라인·HWP5 stored-pagination profile·column top 조건으로 제한되어 있고,
테스트는 `vpos=500`의 16쪽과 `vpos=0`의 90쪽 반례를 함께 고정한다. `issue_598`의 hit-test
좌표 세 개는 이 실제 위치 이동에 맞춰 갱신했다.

## 검증 입력과 Visual Sweep

| 역할 | 저장소 경로 | SHA-256 | 확인 |
| --- | --- | --- | --- |
| HWP 원본 | `samples/hwpctl_API_v2.4.hwp` | `d11dd1331083be4e8c989dfbd587777626b3d77686d3436c35a2c20da9494603` | 추적됨, `rhwp info --json`: HWP5, 105쪽 |
| 기준 PDF | `pdf/hwpctl_API_v2.4-hwp-2020.pdf` | `1d289727dd40ed35e48135bf16df06fe4cd080d967441ff464fb0e0b205fae74` | 추적됨, 105쪽 |
| Native overlay 16·90쪽 | `mydocs/pr/assets/pr7320_review/native_overlay_016.png`, `native_overlay_090.png` | `a61a611a…`, `9cbb155c…` | 이번 integration head에서 생성 |
| fresh WASM overlay 16·90쪽 | `mydocs/pr/assets/pr7320_review/wasm_overlay_016.png`, `wasm_overlay_090.png` | `999bdd56…`, `a9705c29…` | 이번 integration head에서 생성 |

`CARGO_TARGET_DIR=target/review-open-prs-20260921 scripts/wasm-pack-locked.sh --target web
--out-dir /tmp/rhwp-open-prs-wasm-20260921`로 새 WASM package를 만들고, 동일 integration head
`bd974e19b`에서 다음을 각각 실행했다.

```text
python3 scripts/visual_sweep.py --file-target pr7320-tac-top \
  samples/hwpctl_API_v2.4.hwp pdf/hwpctl_API_v2.4-hwp-2020.pdf \
  --rhwp-bin target/review-open-prs-20260921/release-test/rhwp --pages 16,90 --dpi 96 \
  --out /tmp/rhwp-pr7320-native-visual-20260921
# fresh WASM은 위 명령에 --wasm-pkg /tmp/rhwp-open-prs-wasm-20260921 추가
```

| backend / 페이지 | 핵심 표 top | 기준 / 판정 |
| --- | ---: | --- |
| Native 16쪽 | `142.7px` | PDF `142.56px`, 차이 `0.14px` |
| fresh WASM 16쪽 | `142.7px` | 동일 geometry |
| Native·fresh WASM 90쪽 | `136.0px` | PDF `136.01px`, 반례 유지 |

두 실행 모두 Visual Sweep 구조 flag 0건, frame overflow 0건이었다. 선 밴드 최대 차이는
16쪽 1px, 90쪽 2px이다. 전체 raster의 ink match는 16쪽 44.46%, 90쪽 25.20%로 낮지만, 이는
기존 글꼴 raster 차이이며 overlay와 render-tree에서 이번 표·테두리 geometry의 회귀는 보이지
않는다. 이를 전체 PDF 픽셀 일치로 해석하지 않는다.

## 공통 조판 원칙과 입력 보존

| 항목 | 판정 | 근거 |
| --- | --- | --- |
| 구현 근거와 일반성 | 충족 | `vpos=500`, 0, 합성·누적축 제외를 분리하고 반례까지 검사 |
| 측정·배치 일관성 | 충족 | 저장 line segment를 같은 HWP unit→px 변환으로 TAC table y에 한 번만 반영 |
| 분할·이어받기 | 비해당 | 표 분할·예약·이월 분기를 변경하지 않음 |
| 줄 소속과 점유 높이 | 충족 | 빈 앵커가 paragraph layout을 건너뛰는 경로와 column-top 조건을 직접 대조 |
| 사례와 증거의 독립성 | 충족 | 기준 PDF와 저장소 HWP 및 Native/fresh WASM을 통합 head에서 재실행 |
| 기준값 변경 | 비해당 | baseline/golden 갱신 없음 |
| 검증 입력 commit 포함 | 충족 | HWP·PDF·대표 overlay 4개가 모두 이 branch에 추적됨 |

## Merge 후 contributor PR comment 계획

통합 PR이 merge된 뒤에만 원 PR에 UTF-8 body file로 한국어 comment를 게시한다. 실제 merge SHA,
CI URL, 위 두 페이지의 수치와 "글꼴 raster 잔차는 남지만 TAC 표 geometry는 통과"라는 판정을
기록한다. Visual Sweep 정본 링크와 함께 다음 네 representative image를
`https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr7320_review/...`
형식으로 넣고, 게시 뒤 API로 image URL·UTF-8 본문·merge SHA를 다시 확인한다.

- `native_overlay_016.png`
- `native_overlay_090.png`
- `wasm_overlay_016.png`
- `wasm_overlay_090.png`

## 최종 판정

**승인.** 남은 글꼴 raster 차이는 이번 TAC 저장 `vpos` 보정의 범위를 넘으며, 표 top 위치·반례와
구조 검사는 Native와 fresh WASM 모두 통과했다. 통합 PR 생성·merge 전에는 최신 integration head의
CI와 작업지시자 승인을 다시 확인한다.
