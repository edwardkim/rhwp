# Task #412 Stage 3 — 다중 샘플 회귀 검증

## 1. 검증 대상

| 샘플 | 페이지 수 | 특성 |
|------|----------|------|
| exam_eng.hwp | 8 | 다단(2col) + 표 + 그림 — 본 task 의 주 대상 |
| exam_kor.hwp | 24 | 다단(2col) + 표 + 그림 + 폰트 다양 |
| exam_math.hwp | 20 | 다단 + 수식 |
| k-water-rfp.hwp | 28 | 단단 + 표 + 페이지 분할 |
| 2025년 기부·답례품 실적 지자체 보고서_양식.hwpx | 30 | 단단 양식 |
| aift.hwp | 78 | 단단 일반 |
| kps-ai.hwp | 80 | 단단 일반 |

## 2. cargo test

```
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

전체 테스트(여러 test crate) 통과.

## 3. VPOS 보정 분포 변화 (pre vs post-fix)

### exam_eng.hwp (대표 샘플)

| 분류 | Pre-fix | Post-fix |
|------|---------|----------|
| page_true | 127 | 127 |
| page_false | 15 | 0 ← **fix 효과** |
| lazy_true | 131 | 131 |
| lazy_false | 3 | 3 |

→ 모든 page_path 보정이 정상 적용됨. lazy_path 분포 동일.

### exam_kor.hwp

| 분류 | Pre-fix | Post-fix |
|------|---------|----------|
| page_true | 284 | 296 |
| page_false | 16 | 4 ← **12건 정상화** |
| lazy_true | 363 | 369 |
| lazy_false | 11 | 5 |

### k-water-rfp.hwp

| 분류 | Pre-fix | Post-fix |
|------|---------|----------|
| page_true | 240 | 240 |
| page_false | 0 | 0 |
| lazy_true | 50 | 50 |
| lazy_false | 0 | 0 |

→ 영향 없음 (이미 정상).

## 4. 시각 검증 (page 1 sampling)

| 샘플 | 결과 |
|------|------|
| exam_kor p1 | 좌·우 단 옵션 간격 균일, 시각 회귀 없음 ✓ |
| exam_math p1 | 다단 정상 렌더 ✓ |
| k-water-rfp p1 | 단단 표지 정상 ✓ |
| aift p1 | 안내 박스 정상 렌더 ✓ |
| kps-ai p1 | 정상 ✓ |
| 2025년 기부·답례품 p1 | 정상 ✓ |

## 5. exam_eng.hwp 상세 검증

### 5.1 옵션 간격 (option-to-option) 균일성

| 위치 | Pre-fix | Post-fix | 상태 |
|------|---------|----------|------|
| p1 좌측 단 item 1 | 28.56 / 21.89 / 21.89 / 21.89 (catch-up) | 21.89 균일 | ✓ |
| p1 우측 단 item 7 | 15.34 / 15.33 / 15.34 / 15.33 (좁음) | 22.55 균일 | ✓ |
| p2 좌측 단 item 13~16 | 23.01 균일 | 23.01 균일 | 영향 없음 |
| p2 우측 단 item 18 | 15.33 / 19.92 / 19.92 / 19.92 | 동일 | ❌ 잔존 |
| p2 우측 단 item 20 | 26.59 / 19.92 (catch-up) | 19.92 균일 | ✓ |

### 5.2 잔존 이슈 — Item 18 (overlay shape bypass)

p2 우측 단 첫 옵션 그룹 (pi=105~109) 의 ①→② 가 15.33 px 로 좁음. 원인:
- pi=104 가 Picture(InFrontOfText, tac=false) + Table(TopAndBottom, tac=true) 보유
- vpos 보정 분기가 `prev_has_overlay_shape` bypass + `prev_tac_seg_applied` 가드로 pi=105 에서 SKIP
- 결과: lazy_base 초기화가 pi=106 으로 미뤄지면서 한 nudge 누락

→ **별도 task 로 추적 필요** (overlay shape / tac 가드와 lazy_base 초기화 분리).

## 6. Golden snapshot 영향

| Test | 변경 |
|------|------|
| issue_147_aift_page3 | 약 3.68 px shift (single ls 적용) → 골든 갱신 |
| issue_157_page_1 | anchor 도입 후 자동 정상화 (golden 변경 없음) |
| 기타 4건 | 변경 없음 |

issue #147 의 본래 검증 목적(메모→바탕쪽 오분류) 과 무관한 위치 미세 조정이며, 갱신된 골든은 실제 HWP 의 vpos 의도에 더 가까움.

## 7. 회귀 위험 평가

| 영역 | 평가 |
|------|------|
| 단단 문서 | 영향 없음 (k-water-rfp 등 변화 없음) |
| 다단 문서 (page_path) | base 가 큰 단(우측 단 등) 에서 보정이 정상 발동 → **개선** |
| 다단 문서 (lazy_path) | 일부 paragraph spacing_after 적용 (개선) |
| 표·그림 인접 paragraph | overlay shape bypass 케이스는 영향 없음 (item 18 잔존 부합) |
| 페이지 분할 (PartialParagraph) | snapshot diff 없음, 회귀 미관찰 |

## 8. Stage 4 진행 가능

- [x] cargo test 6/6 통과
- [x] 7개 샘플 시각 검증 통과
- [x] page/lazy 분포 개선 확인
- [x] 잔존 이슈 식별 및 별도 task 분리 권고
- [ ] 최종 보고서 작성 (Stage 4)
