# Stage 3 보고서 — #1046: Class B+C 수정 (표 뒤 trailing 간격 overflow 오검출 정정)

- 타스크: #1046 (M100), 브랜치 `local/task1046`
- 단계: 측정 통일(B) Stage 3 — Class B(통째 표) + Class C(연속 PartialTable)
- 작성일: 2026-05-21
- 대상: pi=266/308 (sec0), 354/357 (sec1) [B] · pi=218/600 (sec0) [C]

## 1. 진단 — overflow 오검출(레이아웃은 정확)

통째/분할 표 항목의 LAYOUT_OVERFLOW 가 **표 콘텐츠 자체가 아니라 표 뒤에 더해지는
host 문단 trailing 간격**(tac 줄간격 / spacing_after / outer_margin_bottom)에서 발생.

확정 예 (WHOLE_TABLE_Y 진단, page 40 TER-003):
```
pi=266 table_y_start=580.7 table_y_end=1044.2 table_h=463.6   # 표는 본문(1046.9) 안
LAYOUT_OVERFLOW para=266 type=Table y=1054.1 → trailing +9.9px 가 margin 초과
```
- pi=266 표는 1044.2 에서 끝나 본문에 정상 진입, 다음 내용(pi=267/268)은 다음 페이지로
  정상 분리. 한컴 PDF 도 동일(TER-003 한 페이지 통째). 즉 **시각적 초과 없음** — overflow
  검출이 페이지 바닥의 후행 간격을 콘텐츠 초과로 오판한 false-positive.
- pi=308 동일(표 1042.8 < 1046.9, trailing +15.7). C(218/600)도 분할 표 동일 패턴(~2-3px).

## 2. 수정 — 검출이 표 콘텐츠 하단을 사용 (렌더 불변)

`last_table_content_bottom: Cell<f64>` 추가. 표 렌더가 trailing 간격을 더하기 **직전**의
y(=실제 콘텐츠 하단)를 기록:
- 통째 표: `layout_table_item` 의 `table_y_end` 기록.
- 분할 표: `layout_partial_table` 반환 직후 기록(spacing_after/outer_margin 가산 전).

항목 디스패치마다 `NaN` 리셋(표 렌더에서만 설정 → stale 누수 없음). overflow 검출은
Table/PartialTable 항목에서 이 값으로 비교(미기록 시 종전 y_offset). **렌더링 출력은
일절 불변** — 검출(LAYOUT_OVERFLOW 기록)만 정정. 문단 trailing_ls 정책(#359/#404)의 표 대응.

genuine page-larger(콘텐츠 자체가 본문 초과: pi=323 등)는 content_bottom 이 본문을
넘으므로 그대로 검출된다(미억제).

## 3. 결과 (대상 샘플)

| 지표 | Stage 3 A 후 | Stage 3 B+C 후 |
|------|-------------|----------------|
| LAYOUT_OVERFLOW 총건 | 15 | **9** |
| in-scope (page-larger 제외) | 11 | **5** |
| 해소 | — | B: 266/308/354/357 · C: 218/600 |
| page-larger (323/567) | 2 | 2 (불변) |
| 신규 overflow | — | 0 |

## 4. 회귀 검증
- `cargo test --release`: **1516 passed / 0 failed** (렌더 불변 → 골든 무영향).
- 대상 185p / aift 74p 불변.

## 5. 잔여 (in-scope 5건, 전부 Class D 문단 분할)
pi=361(4.3, PartialPara), 429(10.7, FullPara), 781(15.8, FullPara), 섹션1 268(12.3,
PartialPara), 406(3.1, FullPara). 표 경로 무관 — partial-paragraph 줄단위 배치 드리프트.
