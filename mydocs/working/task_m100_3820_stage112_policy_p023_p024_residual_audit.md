---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-10
---

# Task #3820 Stage 112 — 정책연구 p23–p24 그림·caption 잔여 감사

## 목적

Stage 12는 정책연구 p23–p24의 그림 21–24 이미지·caption flow와 raster 차이를
별도 source contract로 남겼다. 최신 renderer에서 이 차이가 실제 그림·caption
소유권 또는 기하 결함으로 남아 있는지 한컴 PDF와 직접 재판정한다.

- 시작 commit: `14a11b6e9`
- 입력 HWP SHA-256:
  `50094a3db2b2003b293c5cbf43014d001aa97929acb488cef0cb7ea0e16b3113`
- 기준 PDF SHA-256:
  `7879ffee6313575132187c44c0090cd2e62c32c12c29b7eabd989181acf27b3a`
- 기준: 한컴오피스 2020 PDF
- 검증 바이너리: `target/pr-review/release-test/rhwp`

## 실행과 원장

`fidelity_compare.py`를 새 output에서 p23–p24에 실행하되 SVG와 render tree는 전체
215쪽을 다시 생성했다.

- 기준 PDF / rhwp SVG / render tree: `215/215/215`쪽
- 요청 / 완료 / 누락: `2/2/0`
- p23·p24 PDF-only / SVG-only 문자: 모두 0
- owner-shift / owner-sequence / page-boundary: 모두 0건
- visible text excess: 0건
- image outside frame / table-footer / body-footnote overlap: 모두 0건
- text-band clip: 0건

## PDF 직접 판정

p23과 p24의 원본 크기 compare를 확대해 다음을 확인했다.

- p23의 그림 21·22는 PDF와 같은 1×2 배치이며 두 그림과 각 5줄 caption이 같은
  페이지를 소유한다.
- 그림 21 caption 첫 줄과 그림 22 caption 첫 줄의 수직 위치가 PDF와 일치하고,
  caption이 서로 또는 후속 본문과 겹치지 않는다.
- p24의 그림 23은 페이지 상단을 소유하고 caption이 그림 바로 아래에 유지된다.
- p24의 표·그림 24·후속 본문은 PDF와 같은 순서와 페이지를 소유하며 소실·중복이
  없다.

픽셀 diff는 p23 `11.56%`, p24 `14.72%`다. 차이는 chart raster와 본문 글꼴 획,
antialiasing이 주로 차지하며 그림·caption bbox나 페이지 flow가 틀린 증거는 아니다.
자동 flag 0만으로 닫지 않고 위 직접 시각 판정과 문자·layout 원장을 함께 사용했다.

## 기존 실물 회귀

그림 21과 caption의 핵심 source contract를 다음 exact 회귀로 재확인했다.

```text
CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=target/pr-review \
cargo test --profile release-test \
  --test issue_3738_hwp_caption_cell_alignment
```

결과는 `1/1` 통과다. 회귀는 그림 본체 `image_y ≈ 148.3px`와 caption 첫 줄
`caption_y ≈ 495.2px`를 한컴 PDF 위치에 대해 ±3px로 고정한다.

## 결론

Stage 12의 p23–p24 잔여 기록은 최신 renderer에서 재현되지 않는다. 그림 21–24와
caption flow는 PDF와 일치하므로 코드 변경 없이 stale 잔여를 폐기한다. 다음 stage는
최신 exact 회귀가 없는 장대 분할 표 p160–p165를 직접 감사한다.

## 증적

- [p23 비교](../pr/assets/task_m100_3820_stage112_policy_p023_p024_residual_audit/compare_p023.png)
- [p24 비교](../pr/assets/task_m100_3820_stage112_policy_p023_p024_residual_audit/compare_p024.png)
- [픽셀 순위](../pr/assets/task_m100_3820_stage112_policy_p023_p024_residual_audit/report.tsv)
- [문자 원장](../pr/assets/task_m100_3820_stage112_policy_p023_p024_residual_audit/text-report.tsv)
- [layout 원장](../pr/assets/task_m100_3820_stage112_policy_p023_p024_residual_audit/layout-candidates.tsv)
