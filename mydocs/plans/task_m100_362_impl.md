# Task #362 구현계획서 — kps-ai p56 외부 표 안 콘텐츠 클립 결함 정정

## 사전 조사 (수행계획 후 추가)

### SVG 비교 — 정량 확인

kps-ai p56 의 SVG 출력 비교 (마지막 텍스트 "(평가점수)" y 좌표):

| 항목 | v0.7.3 | main |
|---|---|---|
| 외부 셀 clipPath height | 865.05 px | 865.05 px (동일) |
| 마지막 텍스트 y | **1001.18** | **1020.65** |
| 차이 | — | +19.5 px |

외부 셀 clipPath 의 y 끝 ≈ 155.48 + 865.05 = **1020.53 px**.

→ main 의 텍스트 y=1020.65 가 clipPath 끝 (1020.53) 을 0.12 px 초과 → **클립 발생**.
→ v0.7.3 는 19.5 px 위에 있어 클립 없음.

### 핵심 진단

차이 19.5 px = **셀 안 콘텐츠 누적 높이가 main 에서 19.5 px 더 크게 측정**됨.

외부 표 자체 크기는 동일 (865.05 px). 셀 padding 도 데이터상 동일. 차이는:
1. 셀 안 paragraph 측정 (line_height / line_spacing / spacing)
2. 셀 안 내부 표 (7x6) 측정
3. paragraph 와 내부 표 사이 간격 처리

### 의심 origin

`git diff v0.7.3..local/devel -- src/renderer/height_measurer.rs` 결과:
- 셀 padding 처리 변경 (line 478-498) — Task #279 의 cell padding aim 호환

Task #279 의 변경 시멘틱:
- v0.7.3: `cell.apply_inner_margin` 만 체크 → 셀별 padding 적용
- main: `prefer_cell_axis` 함수로 padding 0 도 명시값으로 존중 + cell vs table padding 비교

본 case 의 외부 셀 padding=(510,510,141,141), table.padding=(510,510,141,141) 동일 → padding 자체로는 차이 없음.

내부 7x6 표의 셀 padding=(141,141,141,141), table.padding=(141,141,141,141) 동일 → 동일.

→ padding 자체가 origin 이 아닐 수 있음. 다른 origin (셀 내 paragraph height, 내부 표 측정, scale-down 로직) 추가 점검 필요.

### Stage 1 진단 항목

본 결함은 측정 단계 문제로 추정. Stage 1 에서 정확한 19.5 px 차이의 origin 식별:
1. 셀 안 11 paragraphs 의 측정 높이 비교 (v0.7.3 vs main)
2. 내부 7x6 표의 측정 높이 비교
3. scale-down 발동 여부 + scale 값
4. table_layout 의 셀 안 콘텐츠 렌더링 시 사용 좌표 비교

## 단계 (4 단계, 각 단계는 보고서 + 승인 게이트)

### Stage 1 — 결함 origin 정량 분석

**Task 1.1 — 셀 안 paragraph 측정**
- HeightMeasurer::measure_paragraph 의 11 paragraphs 측정 결과 (v0.7.3 vs main)
- 임시 디버그 출력 추가: pi=535, ci=0 의 셀 paragraphs 측정 시 vpos / line_height / line_spacing / total_height

**Task 1.2 — 내부 7x6 표 측정**
- measure_table_impl 의 내부 표 측정값
- row_heights, raw_table_height, common_h, scale 값

**Task 1.3 — scale-down 발동 여부**
- 외부 표 (pi=535) 의 raw_table_height vs common_h
- scale 발동 시 row_heights 변화

**Task 1.4 — 측정값 origin 비교**
- v0.7.3 vs main 의 어느 측정값이 19.5 px 차이 나는지

**산출물**: `mydocs/working/task_m100_362_stage1.md`

### Stage 2 — 수정 방안 설계

**Task 2.1 — 수정 위치 + 변경 내용**
- Stage 1 결과 기반 수정 위치 확정
- 후보:
  - A: 셀 padding 처리 정정 (Task #279 의 회귀 정정)
  - B: TAC 클램프 시 셀 콘텐츠도 동일 비율로 scale-down
  - C: 외부 표 안에 표가 있는 경우 클램프 동작 변경
  - D: 다른 origin (line_spacing / spacing_after / paragraph height 등)

**Task 2.2 — 영향 범위 분석**
- form-002 (TAC 표) 회귀 점검
- k-water-rfp (다양한 표) 회귀 점검

**산출물**: `mydocs/working/task_m100_362_stage2.md`

### Stage 3 — 코드 수정 + 자동 회귀

1. Stage 2 확정 수정 적용
2. `cargo build --release`
3. `cargo test --lib` (1008+) / svg_snapshot (6/6) / issue_301 / clippy / wasm32 통과
4. 7 핵심 샘플 + form-002 + k-water-rfp + kps-ai 페이지 수 + LAYOUT_OVERFLOW 회귀 0
5. kps-ai p56 의 마지막 텍스트 y 좌표가 v0.7.3 와 동일 (1001.18) 확인

**산출물**: `mydocs/working/task_m100_362_stage3.md`

### Stage 4 — WASM 빌드 + 시각 검증 + 최종 보고서

1. WASM Docker 빌드
2. 작업지시자 시각 판정:
   - kps-ai p56 외부 표 안 콘텐츠 정상 표시 (클립 없음)
   - 7 핵심 샘플 회귀 0
3. 최종 보고서 + 트러블슈팅 + orders 갱신
4. 타스크 브랜치 커밋 + local/devel merge

## 참고

- 수행계획서: `mydocs/plans/task_m100_362.md`
- 이슈: [#362](https://github.com/edwardkim/scrhwp/issues/362)
- 비교 기준: v0.7.3 의 SVG y=1001.18 (외부 셀 clipPath 안)
