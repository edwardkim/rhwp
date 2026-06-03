# Task #412 Stage 1 — 진단 결과

## 1. 진단 코드

`layout.rs:1422-1430` vpos 보정 분기에 `RHWP_VPOS_DEBUG` 환경변수 기반 진단 로그 추가. 출력 형식:

```
VPOS_CORR: path={page|lazy} pi=N prev_pi=N prev_vpos=… vpos_end=… base=… col_y=… y_in=… end_y=… applied={true|false}
```

## 2. 샘플별 path/applied 분포

| 샘플 | page_true | page_false | lazy_true | lazy_false |
|------|-----------|-----------|-----------|-----------|
| exam_eng.hwp | 127 | 15 | 131 | 3 |
| exam_kor.hwp | 284 | 16 | 363 | 11 |
| exam_math.hwp | 0 | 0 | 184 | 30 |
| k-water-rfp.hwp | 240 | 0 | 50 | 0 |
| aift.hwp | 442 | 27 | 227 | 47 |
| kps-ai.hwp | 320 | 4 | 234 | 7 |

## 3. page_false 의 base 분포

| 샘플 | base=0 | base ≈ 7000 (수천대) | base ≥ 60000 (대) |
|------|--------|------|------|
| exam_eng | 2 | 13 (= 7060) | 0 |
| exam_kor | 4 | 11 (= 7085) | 1 (= 83187) |
| aift | 7 | 0 | 20 (= 64005, 66275, 69900) |
| kps-ai | 4 | 0 | 0 |

## 4. 카테고리 분류

### A. **본 task 의 타깃 (수정 대상)**: base ≈ 7000 의 다단 우측 단

- **exam_eng**: pi=34~46 (item 7~12 옵션) base=7060
- **exam_kor**: pi=13~20 등 base=7085

→ end_y = col_y + (vpos_end - 7060) × scale ≈ col_y + 80~150 px (작음)
→ y_in (sequential) > end_y → **부당하게 스킵됨**

### B. **vpos reset 합법 스킵**: base 가 매우 큰 값 (60000+)

- aift pi=223 base=69900, vpos_end=3840 → end_y = -805 (음수, 컬럼 밖)
- pi=237~ base=64005, vpos_end=7000~13000 → end_y 음수 또는 작음

→ vpos 가 컬럼 경계에서 0 으로 reset 되어 base 와 vpos_end 의 선후 관계가 역전된 케이스. 보정 스킵이 올바름.

### C. **base=0 미세 drift**: 보정값이 sequential 보다 1~5 px 작음

- 누적 정렬 오차가 임계값을 넘긴 케이스. 본 fix 영향 없음.

### D. **lazy_false 미세 drift**: 1~3 px 차이

- lazy_base 경로 자체는 정상 작동. base 차감 의미 있음.

## 5. fix 영향 범위 검증

제안 fix: `vpos_page_base` 경로에서 `end_y = col_y + vpos_end × scale` (base 차감 제거).

| 카테고리 | 영향 | 결과 |
|---------|------|------|
| A (target) | end_y 가 ≈ 80px 증가 → applied=true | **버그 수정** |
| B (vpos reset) | base=70000 → end_y = col_y + vpos_end × scale (양수, 작음). 그래도 y_in (큼) 보다 작아 SKIP 유지 | 영향 없음 |
| C (base=0) | 공식 동일 (base=0 차감은 무의미) | 영향 없음 |
| D (lazy_false) | lazy 경로는 변경 없음 | 영향 없음 |

추가 확인: B 카테고리는 `applied=true` 로 바뀌면 안 됨. 새 공식으로 end_y 재계산:

- aift pi=194: 새 end_y = 75.57 + 14080 × 0.01334 = 263.40 (기존 263.31). y_in=452.77 > 263.40 → SKIP 유지 ✓
- aift pi=223: 새 end_y = 75.57 + 3840 × 0.01334 = 126.80. y_in=142.77 > 126.80 → SKIP 유지 ✓
- aift pi=237: 새 end_y = 75.57 + 65925 × 0.01334 = 955.01. y_in=241.88 < 955.01 → end_y in column? col_height 보다 큰지 확인 필요. 페이지 높이 ≈ 1587 px 이므로 955.01 ≤ col_y + col_height (약 1500) 이라면 APPLY 가능. **잠재적 회귀 후보.**

→ Stage 2 에서 aift pi=237 등 base 70000 케이스의 fix 후 동작 시각 검증 필요.

## 6. exam_kor pi=13~20 변환 시뮬레이션

기존 (page_false 스킵):
- pi=13 y_in=333.32 → 그대로
- pi=14 y_in=373.16 → 그대로 (line_height만 누적)

새 공식 (applied):
- pi=13 end_y = 211.65 + 9165 × 0.01334 = 333.91 → APPLY
- pi=14 end_y = 211.65 + 13581 × 0.01334 = 392.82 → APPLY (sequential 보다 19.66 px 위로 보정)

→ 우측 단 옵션이 절대 vpos 위치로 정렬됨 = 의도한 fix.

## 7. Stage 2 진행 가능 여부

- **A 카테고리 (목적)**: fix 적용으로 정상화 ✓
- **B 카테고리 잠재 회귀**: aift pi=237 등 base 가 매우 큰 케이스에서 새 공식이 col_height 안에 들어와 잘못 APPLY 될 가능성 → Stage 2 / 3 에서 시각 비교 검증 (특히 aift)

→ Stage 2 진행 권고. 단, Stage 3 에서 aift 변동 페이지 정밀 시각 검증 필요.

## 8. 잠재 회귀 보완 옵션

만약 B 카테고리에서 회귀 발견 시:
- **fix 적용 조건 추가**: `base > 0` 인 경우에만 page_path fix 적용. base=0 는 기존 (vpos_end - 0 = vpos_end 동일이라 무의미하므로 영향 없음). base 가 매우 큰 vpos reset 케이스는 별도 가드 추가.
- 또는: `vpos_end >= base` 인 경우에만 fix 적용 (vpos reset 케이스는 vpos_end < base 가 됨 → 기존 공식 유지).

## 9. 결론

Stage 2 진행. fix 후 회귀 검증에서 aift 의 base 가 큰 케이스를 우선 시각 검사한다.
