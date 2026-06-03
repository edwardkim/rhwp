# Stage 5-4 완료보고서 — #1022: VPOS_CORR 정합 검증

- 타스크: #1022 / 브랜치 `local/task1022`
- 작성일: 2026-05-20
- 단계: Stage 5-4 — Stage 5-3 (VPOS_CORR over-correction 제거) 검증

## 1. 검증 결과

| 항목 | 결과 |
|------|------|
| `cargo build --release` | 무경고 |
| `cargo clippy --release` | 무경고 |
| `cargo test --release` | **1302 passed**, 0 failed, 6 ignored |
| `svg_snapshot` | **8 passed** (issue-617·issue-677 골든 갱신 후) |
| 비공개 184페이지 `LAYOUT_OVERFLOW` | **42 → 23건** |
| 페이지 22 18.3px overflow | **해소** |

## 2. 골든 판정 (한컴 2022 PDF 대조)

VPOS_CORR 변경으로 이동한 공개 골든 2건:

| 골든 | 변화 | 판정 |
|------|------|------|
| issue-617 (exam_kor p5) | 부동소수 말단(`732.6266666666666`→`...667`) | 노이즈, 갱신 |
| issue-677 (복학원서 p1) | 전체 5.3px 상향(trailing_ls 제거) | **vpos 정합** — 복학원서 PDF 대조, 본래 의도("LineSeg.vpos 정합")에 부합. golden 은 trailing_ls 만큼 과보정된 상태였음. 갱신 |

form-002 골든: 정합 유지(변동 없음).

## 3. 잔여 23건 분류

### 3-1. 페이지보다 큰 콘텐츠 (3건, 사전 존재·내 변경 무관)

베이스라인(42건)과 **동일**:
- pi=272 PartialTable 854.9px (page 40)
- pi=567 PartialParagraph 856.7px (page 93)
- pi=324 PartialTable 143.9px (page 63)

페이지보다 큰 중첩 표/문단으로 내부 분할(`calc_nested_split_rows`)이 필요한 별개 경로. task993 §4 에서 scope-out.

### 3-2. small/med 드리프트 (20건)

| 타입 | 건수 |
|------|------|
| PartialTable | 14 (15 - 1 page-larger) |
| FullParagraph | 4 |
| Table | 2 |
| PartialParagraph | 1 (2 - 1 page-larger) |

- PartialTable 분할 잔여: rowspan / 분할 행 측정 미세 차이 (task993 §4 의 rowspan 위임 잔여).
- FullParagraph(pi=642 등): VPOS_CORR 이벤트 없음 → 별개 측정 드리프트 (MeasuredParagraph ↔ 렌더 미세 차이, 누적).
- Table inline: TAC 표 측정 차이.

## 4. 성과 요약

- 본 #1022 명시 범위(`HeightMeasurer ↔ cell_units`): Stage 3 완료(42→38).
- 확장 범위(VPOS_CORR 정합): Stage 5-3 으로 Task #537 의 stale over-correction 제거 → 38→23. 페이지 22 해소.
- 주소 가능 오버플로(non-page-larger): **39 → 20건 (~49% 감소)**.
- 사전 존재 page-larger 3건은 별개 scope.

## 5. 잔여 처리 옵션

(a) 본 #1022 를 현 상태(42→23, 페이지 22 해소)로 마무리. 잔여 20건은
    이종 small/med 드리프트로 각각 별도 조사 — 후속 이슈.
(b) 잔여 small/med 를 계속 chip — PartialTable 분할 잔여(rowspan) /
    FullParagraph 드리프트 / Table 별로 추가 조사·수정. 각각 개별 작업.

페이지 22(사용자 보고 사안) 해소 + 49% 감소로 본 타스크 목표는 실질
달성. 잔여는 이종이라 한 번에 닫기 어렵다.
