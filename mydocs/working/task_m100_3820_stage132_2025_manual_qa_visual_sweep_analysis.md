# Stage 132 - 2025 행정업무운영 편람 Q&A visual sweep 분석

## 목표

Stage 131에서 native HWP의 page count를 Hancom PDF와 같은 383쪽으로 맞춘 뒤, Q&A `RowBreak` 경계의 실제 표 paint와 다음 문단 owner가 PDF와 같은 physical page에 남는지 검증한다. 쪽수만 같아도 빈 spacer 행, 표 bottom margin, 빈 guide line이 눈에 보이게 달라질 수 있으므로 이번 Stage는 시각 근거를 먼저 남긴다.

## 고정 기준선

- Hancom PDF: `pdf/2025 행정업무운영 편람(최종)-hwp-2020.pdf` (383쪽, 사용자가 이미 확보한 oracle)
- native HWP 입력: `samples/2025 행정업무운영 편람(최종).hwp`
- Stage 131 native HWP renderer: 383쪽
- Stage 131 HWPX renderer: 386쪽
- Stage 131 commit: `85e5579db`

## 분석 범위

1. `pi=037`, `pi=056`, `pi=074` 주변에서 native HWP의 page item owner와 PDF physical page를 대응한다.
2. 마지막 빈 spacer 행이 별도 continuation-only page를 만들지 않고 직전 표의 bottom tail로 paint되는지 확인한다.
3. `pi=039`가 0-height guide로 남아도 첫 빈 줄과 다음 Q&A 표 사이의 가시 간격을 줄이지 않는지 확인한다.
4. PDF와 rhwp SVG를 같은 해상도로 rasterize해 overlay 또는 side-by-side 증적을 남긴다.

## 보존 계약

- 기준 PDF를 새로 생성하거나 교체하지 않는다. 위의 기존 Hancom PDF를 oracle로 사용한다.
- Stage 131의 383쪽 native HWP / 386쪽 HWPX regression을 유지한다.
- PDF physical 278의 `PageHide` blank page와 기존 Stage 128~131의 별도 저장 계약은 변경하지 않는다.
- 시각 evidence가 없으면 renderer 코드를 변경하지 않는다.

## 수용 기준

1. 각 Q&A 경계의 PDF/RHWP page 대응과 paint 차이를 결과 문서에 기록한다.
2. visual divergence가 있으면 source owner, SVG node 또는 layout metric으로 원인을 특정한다.
3. 수정이 필요할 때만 source, focused regression, 결과 evidence를 하나의 Stage 커밋으로 고정한다.

## owner 재분석 결과

- Stage 130과 Stage 131의 `dump-pages`를 같은 global page로 비교했다.
- 383쪽 달성의 세 source owner는 `pi=030`, `pi=056`, `pi=074`의 마지막 빈 spacer 행이다. 세 표는 모두 native HWP 6행×5열/15-cell `RowBreak`, `outer_margin_bottom=566 HU` 구조다.
- Stage 130에는 `pi=030`의 `37.3px` continuation-only page가 있었고, Stage 131에서는 이 행이 앞 page에 포함되면서 이후 global page가 하나 앞당겨진다. `pi=056`, `pi=074`도 같은 방식으로 각각 한 page를 제거한다.
- `pi=039`는 Stage 130과 Stage 131 모두 일반 `FullParagraph`로 남았고, Stage 131의 duplicate-guide predicate는 source의 실제 shape identity에서 발동하지 않았다.
- 따라서 `pi=039` 규칙은 383쪽 달성에 기여하지 않으며, 이를 강제로 발동하면 oracle보다 한 쪽 적은 382쪽이 될 위험이 있다. Stage 132에서는 해당 dead predicate를 제거하고, 증명된 terminal-spacer 규칙만 남긴다.

## visual sweep 결과

- current native HWP와 Hancom PDF의 physical 294, 295, 297, 301쪽을 각각 144dpi PNG로 rasterize해 side-by-side로 비교했다. intermediate evidence는 `/tmp/rhwp-3820-stage132-qa-visual/`에 둔다.
- 쪽수는 모두 383쪽이지만, 동일 printed page에서 Q&A owner가 일치하지 않는다.

| physical page | Hancom PDF | rhwp | 판정 |
| --- | --- | --- | --- |
| 294 (printed 286) | Q24-Q26 | Q20-Q21 | rhwp가 4개 질문 뒤처짐 |
| 295 (printed 287) | Q27-Q29 | Q22-Q23 | rhwp가 5개 질문 뒤처짐 |
| 297 (printed 289) | Q32-Q33 | Q27-Q29 | rhwp가 3개 질문 뒤처짐 |
| 301 (printed 293) | Q40-Q41 | Q36-Q37 | rhwp가 4개 질문 뒤처짐 |

- SVG/PDF renderer는 같은 분석 중 `pi=036 -> pi=037`에 14.5px, `pi=055 -> pi=056`에 28.8px의 `LAYOUT_TABLE_OVERLAP`도 보고했다. page count만 맞추려 마지막 spacer를 앞 page에 overflow시키는 것은 visual fidelity의 해결이 아니다.
- `pi=039` duplicate-guide helper는 actual source에서 한 번도 `HiddenEmptyPara`로 발동하지 않았고, 383쪽 결과에도 기여하지 않았다. 이번 Stage에서 제거했다.

## 검증 결과

- 실행: `CARGO_TARGET_DIR=target/stage124-3820 cargo test --profile release-test --test issue_3930_hwpx_hwp_save_layout --quiet`
- 결과: 3 passed, 0 failed (0.90초).
- native HWP 383쪽, HWPX 386쪽의 focused regression은 유지한다.
- 그러나 Q&A physical page owner와 표 paint가 PDF와 다르므로 Issue #3820은 종료하지 않는다. 다음 Stage는 첫 divergence부터 per-row/table height와 overlap을 분석한다.
