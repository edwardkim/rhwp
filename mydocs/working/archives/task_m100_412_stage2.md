# Task #412 Stage 2 — 코드 수정 + 검증

## 1. 핵심 변경

### 1.1 anchor 도입: `col_anchor_y`

`build_single_column` 진입 시 `body_wide_reserved` 푸시를 적용한 직후의 `y_offset` 을 `col_anchor_y` 로 캡처. 이 값이 첫 PageItem 이 실제 렌더링되는 y 위치 = 첫 항목 vpos(`vpos_page_base`) 에 대응하는 y 좌표.

**왜 필요한가**: `col_area.y` 는 단 영역의 top 으로, 일반적으로 첫 항목의 vpos 와 일치하지 않음. body_wide_reserved (페이지 너비 글상자/표) 푸시가 있는 경우 첫 항목은 col_area.y 보다 아래에서 시작.

### 1.2 vpos_end 결정: 현재 paragraph 의 first vpos 우선

기존: `vpos_end = prev_seg.vpos + lh + ls`
변경: `curr_first_vpos > prev_seg.vpos` 면 `curr_first_vpos`, 아니면 fallback

HWP 가 paragraph spacing_after 를 다음 paragraph 의 first seg vpos 에 인코딩하므로, 현재 paragraph 의 first vpos 가 더 정확.

### 1.3 page_path / lazy_path 분리

| 경로 | anchor | 공식 |
|------|--------|------|
| page_path (vpos_page_base 있음) | `col_anchor_y` | `col_anchor_y + (vpos_end - base) * scale` |
| lazy_path (vpos_lazy_base 사용) | `col_area.y` | `col_area.y + (vpos_end - base) * scale` |

lazy_path 의 base 는 sequential y_offset 으로부터 역산된 값이라 `col_area.y` 가 vpos=base 좌표를 의미하도록 정의되어 있음. page_path 의 base 는 `col_anchor_y` 가 vpos=base 좌표를 의미.

## 2. 변경 파일

`src/renderer/layout.rs`:
- 한 곳 추가: `let col_anchor_y = y_offset;` (body_wide_reserved 푸시 직후)
- 한 곳 수정: vpos 보정 분기에서 `curr_first_vpos` 산출 + `is_page_path` 분기 + anchor 분리

## 3. 검증 결과 (exam_eng.hwp)

### 3.1 Page 1 우측 단 item 7 (원래 보고된 버그)

| 옵션 간 | Pre-fix Δy | Post-fix Δy | 기대 (vpos delta × scale) |
|--------|-----------|-----------|--------------------------|
| ①→② | 15.34 ❌ | **22.54** ✓ | 22.55 |
| ②→③ | 15.33 ❌ | 22.53 ✓ | 22.55 |
| ③→④ | 15.34 ❌ | 22.53 ✓ | 22.55 |
| ④→⑤ | 15.33 ❌ | 22.54 ✓ | 22.55 |

### 3.2 Page 1 좌측 단 item 1

| 옵션 간 | Pre-fix Δy | Post-fix Δy | 기대 |
|--------|-----------|-----------|------|
| ①→② | 28.56 ❌ (catch-up) | **21.89** ✓ | 21.91 |
| ②→③ | 21.89 ✓ | 21.89 ✓ | 21.91 |
| ③→④ | 21.89 ✓ | 21.89 ✓ | 21.91 |
| ④→⑤ | 21.89 ✓ | 21.89 ✓ | 21.91 |

### 3.3 Page 2 우측 단 item 20

| 옵션 간 | Pre-fix Δy | Post-fix Δy | 기대 |
|--------|-----------|-----------|------|
| ①→② | 26.59 ❌ (catch-up) | **19.92** ✓ | 19.93 |
| ②→③ | 19.92 ✓ | 19.92 ✓ | 19.93 |
| ③→④ | 19.92 ✓ | 19.92 ✓ | 19.93 |
| ④→⑤ | 19.92 ✓ | 19.92 ✓ | 19.93 |

### 3.4 Page 8 (B)(C)(D) 분리 라벨

시각 비교 결과 pre/post-fix 위치 패턴 동일. 글자 겹침 미관찰.

## 4. 잔존 이슈

### 4.1 Page 2 우측 단 item 18 ①→② (별도 task 필요)

| 옵션 간 | Post-fix Δy | 기대 |
|--------|-----------|------|
| ①→② | 15.33 ❌ | 19.93 |
| ②~⑤ | 19.92 ✓ | 19.93 |

**원인**: pi=104 (Picture+Table) 가 overlay shape (InFrontOfText) 또는 tac=true 표를 포함해 vpos 보정 분기가 bypass 됨. 결과적으로 lazy_base 초기화가 pi=106 으로 미뤄지면서 pi=105→106 한 nudge 가 누락.

**왜 task #412 에서 해결 안 함**: overlay shape bypass 와 prev_tac_seg_applied 가드가 얽혀 있어 안전한 fix 가 단순하지 않음. 별도 task 로 추적해야 함.

## 5. 회귀 검증 (cargo test)

```
test result: ok. 6 passed; 0 failed
```

snapshot diff 1건 발생:
- `tests/golden_svg/issue-147/aift-page3.svg`: 약 3.68 px 일괄 shift (single ls 적용). Issue #147 의 본래 검증 목적(메모→바탕쪽 오분류)과 무관한 위치 미세 조정 → 골든 갱신.

snapshot diff 1건 자동 해결:
- `tests/golden_svg/issue-157/page-1.svg`: pre-fix 임시 변경(naive base 차감 제거)에서 발생했던 위치 어긋남이 anchor 도입(col_anchor_y) 후 자동 복원되어 추가 갱신 불필요.

## 6. Stage 3 진행 요건

- [x] cargo test 통과
- [x] 골든 snapshot diff 검토 + 의도 확인
- [x] exam_eng 8페이지 시각 검증 (page 8 정상)
- [ ] 다중 샘플 회귀 검증 (Stage 3)

## 7. 다음 단계

Stage 3: exam_kor / exam_math / k-water-rfp / 2025년 기부답례품 / aift / kps-ai 다중 샘플 회귀 검증.
