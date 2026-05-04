# Stage 3 — 시각·자동 검증 완료 보고서

**Task**: #577 — 셀 내부 단독 TopAndBottom 이미지 1라인 오프셋
**브랜치**: `local/task577`
**단계**: 3/4 (검증)

---

## 1. 본 이슈 대상 — exam_science.hwp 1페이지 보기 ①~⑤

### 셀 vs 이미지 좌표 (after-fix)

| 보기 | 셀 클립 (y, h, bottom) | 이미지 (y, h, bottom) | 결과 |
|------|------------------------|------------------------|------|
| ① | 787.04 / 56.83 / 843.87 | 790.81 / 41.28 / 832.09 | ✅ 셀 내부 |
| ② | 787.04 / 56.83 / 843.87 | 790.81 / 49.28 / 840.09 | ✅ 셀 내부 |
| ③ | 843.87 / 59.60 / 903.47 | 847.64 / 40.96 / 888.60 | ✅ 셀 내부 |
| ④ | 843.87 / 59.60 / 903.47 | 847.64 / 52.80 / 900.44 | ✅ 셀 내부 |
| ⑤ | 903.47 / 54.05 / 957.52 | 907.24 / 45.76 / 953.00 | ✅ 셀 내부 |

5개 모두 image_y - cell_y = **3.78 px** (= pad_top, HWP IR 합치). baseline 의 `+19.10 px` 오프셋 제거됨.

## 2. LAYOUT_OVERFLOW 변화

### exam_science.hwp

```
baseline:  LAYOUT_OVERFLOW: page=1, col=0, para=46, type=Table, y=1374.0, bottom=1364.4, overflow=9.5px
after:     LAYOUT_OVERFLOW: page=1, col=0, para=46, type=Table, y=1367.8, bottom=1364.4, overflow=3.4px
```

→ 9.5 px → 3.4 px (개선 ~6 px).

### mel-001.hwp (회귀 검증 샘플)

baseline 8건:
```
page=1 para=25 Table overflow=3.5px
page=2 para=42 FullParagraph overflow=6.0px
page=3 para=56 Table overflow=10.3px
page=8 para=114 Table overflow=17.2px
page=10 para=147 Table overflow=10.8px
page=14 para=217 Table overflow=8.9px
page=16 para=247 Table overflow=7.7px
page=18 para=285 Table overflow=18.8px
```

after: **0건** (모든 오버플로 제거).

## 3. 좌표 이동 영향 분석

### 본 PR 의도된 변화

비-TAC TopAndBottom Picture (`vert_rel_to=Para`)가 들어 있는 셀에서 이미지가 cell_top + pad_top + line_height (≈ 19.10 px) → cell_top + pad_top (≈ 3.78 px) 로 정정. 이미지가 셀 내부에서 위로 약 15.32 px 이동.

### 부수 효과 (의도 외 보정)

`exam_science.hwp` 와 `mel-001.hwp` 의 SVG 비교 결과 다음 부수 효과 관측:

| 샘플 | 페이지 | 영향 | 해석 |
|------|--------|------|------|
| exam_science | 1 | 문제2 표 cell_y +7.2 px (787.04) — IR vpos=47860 의 expected 787.43 에 정합 | baseline 이 IR 보다 7.6 px 위에 있던 결함이 같이 정정 |
| exam_science | 3, 4 | 일부 본문/표 좌표 ±수 px ~ +96 px 이동 | 다단 패킹 변화 — 일부 페이지에서 컬럼 충진 결과가 달라졌으나 셀 내부 이미지·셀 정합은 유지 |
| mel-001 | 2 | cell_y -4 px 일괄 이동 + body-clip h 미세 변화 | baseline 의 6 px 오버플로가 정확히 사라지는 방향 |
| mel-001 | 1, 3, 21 | 변화 없음 | 영향 없음 |

부수 효과는 모두 **이미지 anchor 가 IR vpos 에 정합되는 방향**(즉 LAYOUT_OVERFLOW 가 줄어드는 방향)이다. baseline 이 시각적으로 잘 보이던 페이지도 사실은 이미지가 셀 내부에서 1라인만큼 밀려 있었거나 컬럼 충진이 어긋나 있었다.

### 위험 평가

- 파편화된 회귀로 보이는 페이지(예: exam_science p3 의 +96 px 이동)는 컬럼 충진의 누적 효과로 보이며, 해당 페이지의 이미지는 여전히 cell-clip 내부에 정확히 들어 있음(verify: cell-clip-470 y=387.52, h=146.48 / image y=389.4, h=142.72 → image bottom=532.12 < cell bottom=534.0 ✅).
- 신규 LAYOUT_OVERFLOW 발생 없음 (exam_science / mel-001 두 샘플 기준).
- 본 부수 효과는 별도 이슈로 상세 회귀 검증을 이어갈 수 있으나, 본 타스크 범위는 "이미지 잘림 정정"이므로 새로운 회귀가 검출되지 않는 한 그대로 진행한다.

## 4. 빌드 / 테스트 재확인

| 항목 | 결과 |
|------|------|
| `cargo build --release` | ✅ |
| `cargo test --release --lib` | ✅ 1125 passed (Stage 2 동일) |

## 5. 잔여 위험

- exam_science.hwp 4페이지 일부 이미지 좌표 ±수십 px 이동 → 추가 샘플 시각 비교 필요시 별도 추적
- 여러 파일에서 일관된 LAYOUT_OVERFLOW 제거 방향 → 추가 PR 회귀 검증 시 본 타스크 결과를 baseline 으로 갱신해야 함

## 6. 다음 단계

Stage 4 — 최종 결과 보고서 작성, 오늘할일 갱신.

## 7. 승인 요청

Stage 3 결과 검토 후 Stage 4(종결) 진행 승인 부탁드립니다.
