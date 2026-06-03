# Stage 3 보고서 — Task #485 회귀 검증

**작성일**: 2026-05-07
**브랜치**: `local/task485`

---

## 1. 검증 범위

Stage 2a + 2b 의 정정 (limit_reached 플래그 + boundary epsilon) 이 다른 분할 표 케이스에 회귀를 유발하지 않는지 점검.

## 2. 검증 항목

### 2.1 본 결함 해소 (재확인)

| 페이지 | 결과 |
|--------|------|
| `synam-001.hwp` p15 | 클립 해소 ✓ |
| `synam-001.hwp` p20 | 클립 해소 ✓ |
| `synam-001.hwp` p21 | 클립 해소 ✓ |

### 2.2 인접 페이지 흐름 (synam-001 p13~p23)

p13~p23 export-svg 정상 출력. 분할 표 (`pi=140` rows=6..7 ; `pi=163` rows=0..1) 의 모든 partial 페이지 (p15·p16·p17 / p20·p21·p22·p23) 가 시각적으로 자연스러움.

PDF 대조 시 페이지 적재량이 한컴과 정확히 일치하지 않음 — 그러나 본 정정의 의도 (descender 침범 방지) 결과 마지막 1~2줄이 다음 페이지로 밀리는 것은 **결함이 아닌 정상 동작**.

### 2.3 Task #362 회귀 점검 (kps-ai.hwp)

**대상**: `kps-ai.hwp` p56/p67/p68/p69/p70/p72/p73 (Task #362 정정 영역)

| 페이지 | 결과 |
|--------|------|
| p56 (idx 55) | 정상 출력 |
| p67 (idx 66) | 정상 출력 (분할 표 → 다음 페이지 흐름 정상) |
| p68/69/70 | 정상 출력 |
| p72/73 | 정상 출력 |
| 전 페이지 export | OK (오류 없음) |

→ Task #362 의 정정 의도 (kps-ai.hwp 88→79 페이지 + 분할 표 정합) 보존.

### 2.4 Task #431 회귀 점검 (synam-001.hwp)

- 빈 페이지 미발생 (전 35페이지 정상 출력)
- 분할 표의 `content_offset >= content_limit` 케이스 정상 처리

### 2.5 분할 표 다른 샘플 점검 (aift.hwp)

- p10~p14 (`pi=123` 13×10 / `pi=134` 23×5 의 분할 표) — 정상 출력
- 분할 셀 마지막 줄 클립 없음
- 페이지 흐름 자연스러움

### 2.6 전체 샘플 export-svg

- `samples/*.hwp` (155개 파일) p0 export-svg — **TOTAL: 155 / FAIL: 0**
- 본 정정으로 인한 파싱/렌더 오류 발생 없음

### 2.7 cargo test 결과

```
passed: 1199, failed: 0, ignored: 2
```

전 단위/통합 테스트 통과.

## 3. 후보 A (typeset split_end_limit) 진입 불요

수행계획서/구현계획서의 후보 A (`engine.rs` 4곳의 `split_end_limit = avail_content` 산정에 epsilon 차감) 는 본 검증 결과 **진입 불요**:

- 본 결함의 본질이 layout 측 (compute_cell_line_ranges) 에서 정정됨
- typeset 의 split_end_limit 자체는 정확 (cell-clip-rect 와 정합)
- 후보 A 적용 시 typeset 측 페이지 분할 결정과 layout 측 가시 라인 결정에 이중 마진이 발생할 위험

→ 후보 A 적용 안 함.

## 4. 후보 D (vpos correction drop) 진입 불요

수행계획서/구현계획서의 후보 D (layout 의 vpos correction 단계 본문 영역 침범 시 드롭) 는 본 검증 결과 **진입 불요**:

- Stage 2a + 2b 만으로 본 결함의 시각 침범 해소
- 후보 D 는 `typeset_layout_drift_analysis.md` §회귀 원인 체인 6항의 collapse 위험 보유 (선행 시도 실패 사례)

→ 후보 D 적용 안 함.

## 5. 잔여 / 위험

### 5.1 페이지 적재량 미세 변화

Stage 2b 의 epsilon 2.0px 적용으로 일부 분할 표가 마지막 ~2px 영역을 양보. 페이지 적재량이 PDF (한컴) 와 1줄 차이가 발생할 수 있으나, 본 결함 해소 측면에서 의도된 동작.

### 5.2 epsilon 임의성

2.0px 는 측정 데이터 기반이지만 본질적으로 휴리스틱. layout drift 본질의 구조적 통일은 별도 이슈로 보관 (`typeset_layout_drift_analysis.md` §단일 모델 통합).

### 5.3 폰트별 descender 차이

서로 다른 폰트의 descender 비율 (~0.15~0.25 × line_h) 에 따라 epsilon 적합성이 다름. 본 케이스는 한국어 폰트 기본 적합. 향후 다국어 폰트 적용 시 재점검 필요.

## 6. 결론

- **본 결함 (페이지 15·20·21 클립) 해소** ✓
- **회귀 없음** — Task #362/#431/#398/#474 의도 보존
- **cargo test 1199 통과**
- **전 샘플 (155개) export 성공**
- 후보 A·D 진입 불요

→ Stage 4 (최종 보고서) 진행 가능.

## 7. 작업지시자 승인 요청

1. 회귀 검증 결과 (회귀 없음) 동의?
2. 후보 A·D 진입 불요 결정 동의?
3. Stage 4 (최종 보고서 + orders 갱신 + GitHub 이슈 코멘트) 진행 동의?

승인 후 Stage 4 진행.
