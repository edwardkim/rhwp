# Stage 3 보고서 — #1046: Class D 수정 (문단 뒤 trailing 줄간격 overflow 오검출 정정)

- 타스크: #1046 (M100), 브랜치 `local/task1046`
- 단계: 측정 통일(B) Stage 3 — Class D(문단 분할 줄단위)
- 작성일: 2026-05-21
- 대상: pi=361/429/781 (sec0), 268/406 (sec1)

## 1. 진단 — 문단 trailing 줄간격 overflow 오검출

PartialParagraph/FullParagraph 의 LAYOUT_OVERFLOW 가 **마지막 텍스트 줄이 아니라 그 뒤
trailing 줄간격(line_spacing)·spacing_after** 에서 발생. 페이지네이터는 이미 마지막 줄
trailing_ls 를 허용해 배치(#359/#404)하는데, 렌더러 검출은 trailing 을 포함한 y_offset 으로
판정해 페이지네이터 정책과 어긋난 false-positive.

예 pi=429 (FullParagraph): cur_h=908.9, sb=9.3, 텍스트 줄 18.7 → 콘텐츠 바닥 ≈936.9 <
본문 941.1(들어감), trailing_ls 14.9 가 초과 → 검출 10.7px. 텍스트는 본문 안.

## 2. 수정 — Class B/C 메커니즘을 문단으로 확장 (렌더 불변)

`last_table_content_bottom` → `last_item_content_bottom` 으로 일반화. 본문 문단(셀 밖)
렌더 시 매 줄에서 `y + line_height`(=텍스트 줄 바닥, trailing 줄간격/spacing_after 가산
전)를 기록 → 마지막 렌더 줄 값이 남는다. overflow 검출이 FullParagraph/PartialParagraph
에서도 이 콘텐츠 바닥으로 비교. **렌더링 출력 불변** — 검출만 정정.

## 3. 결과 (대상 샘플)

| 지표 | Stage 3 B+C 후 | Stage 3 D 후 |
|------|----------------|--------------|
| LAYOUT_OVERFLOW 총건 | 9 | **5** |
| in-scope (page-larger 제외) | 5 | **1** |
| 해소 | — | 361/429/268/406 (순수 trailing) |
| pi=781 | 15.8 | **4.6** (genuine 줄 초과로 축소) |
| page-larger (323/567) | 2 | 2 (불변) |
| 신규 overflow | — | 0 |

## 4. 회귀 검증
- `cargo test --release`: **1516 passed / 0 failed** (렌더 불변 → 골든 무영향).
- 대상 185p / aift 74p 불변.

## 5. 잔여
- **pi=781 (4.6px)**: trailing 아님 — 마지막 줄 자체가 본문을 4.6px 넘어 그려지는 genuine
  초과(LAYOUT_OVERFLOW_DRAW baseline 부터 존재). 페이지네이터 마지막-줄 허용 오차 계열,
  별도 검토.
- page-larger 2건(pi=323 단독 표, pi=567 nested) — 단일 항목이 본문보다 큼, 범위 외.
