# Task #1007 구현 계획서 — HWP5 변환본 페이지 강제 나눔 한컴 정합

수행 계획서: [`task_m100_1007.md`](task_m100_1007.md)

## 1. 구현 대상

`samples/hwp3-sample16-hwp5.hwp` (HWP3 → HWP5 변환본) 의 page 3 사업개요 페이지 분할이 한컴 정합되도록 fix:
- **현재**: Page 3 에 Section 1 (Ⅰ. 사업개요) + Section 2 시작 ("(2) 주전산센터...") 까지 packed
- **목표**: Page 3 = Section 1 만, Page 4 시작 = "(2) 주전산센터..."

## 2. 진단 도구

기존 인프라:
- `RHWP_DEBUG_LAYOUT=1` (Task #936)
- `rhwp dump-pages -p N`
- `rhwp dump -s 0 -p N`
- `ir-diff` (HWP3 vs HWP5 변환본 비교)

신규 (Stage 1 도입):
- `RHWP_DEBUG_PAGE_FILL=1` — paragraph 누적 시 page 가득찼는지 / page break 결정 logic 추적

## 3. 구현 단계 (5 stage)

### Stage 1 — 정밀 진단

**1-1. paragraph 간격 정확 측정**
- sample16-hwp5 page 3 의 모든 paragraph 의 actual y position (SVG) 측정
- 한컴 viewer 에서 같은 paragraph 의 y position 사용자 시각 측정 (가능 시)
- paragraph 별 spacing_before / spacing_after / line_height / line_spacing 비교

**1-2. page-fill 결정 위치 식별**
- pi=87 (마지막 section 1 paragraph) → pi=88 ("(2)...") 사이의 page break 결정 코드 위치
- `src/renderer/pagination/engine.rs` 의 page packing logic 분석
- 어느 조건에서 한컴은 break 하고 rhwp 는 안 하는지

**1-3. 격차 양 (px) 정확 측정**
- 한컴 page 3 의 마지막 content y vs rhwp page 3 의 마지막 content y
- 차이를 px 단위로 정량화

**Stage 1 산출물**: `mydocs/working/task_m100_1007_stage1.md` — 진단 보고서

### Stage 2 — Fix 후보 평가

**Stage 1 진단 결과로 후보 결정**. 시나리오:

**시나리오 X — paragraph spacing 누적 격차**
- 어느 paragraph type 의 spacing 이 한컴 보다 작은지 식별
- 후보 X1: ParaShape `/4` 보정 + 변환본 시 추가 buffer (예: `* 1.1`)
- 후보 X2: line_segs.line_height 변환본 시 추가 buffer
- 후보 X3: paragraph 사이 minimum gap 강제 (한컴 mimic)

**시나리오 Y — empty paragraph 처리 격차**
- 한컴이 빈 paragraph (pi=87) 직후 section 시작 시 page break 트리거
- 후보 Y1: 빈 paragraph + 다음 paragraph 가 새 section heading 패턴 시 page break

**시나리오 Z — page-fill 정책 차이**
- 한컴이 page 의 특정 y position (예: 90%) 도달 시 다음 paragraph 를 새 페이지로 분리
- 후보 Z1: page packing 시 threshold 조정 (paragraph 가 page 하단 근처에 fit 가능해도 새 페이지로 분리)

**Stage 2 산출물**: `mydocs/working/task_m100_1007_stage2.md` — Fix 후보 평가 + 선정

### Stage 3 — Fix 적용

선정 후보 구현:
- 변경 파일 예상: `src/renderer/pagination/engine.rs` 또는 `src/renderer/typeset.rs` 또는 `src/renderer/style_resolver.rs`
- 변경 줄 수 예상: 10-30 줄
- 단위 검증: sample16-hwp5 page 3 한컴 정합 (page 4 시작 = "(2)...")

**Stage 3 산출물**: `mydocs/working/task_m100_1007_stage3.md` — Fix 보고서

### Stage 4 — 회귀 검증

**4-1. 단위 / 정적 검증**
- `cargo test --release --lib` 전체 (현재 baseline: 1301 passed)
- `cargo clippy --release -- -D warnings`

**4-2. SVG sweep (회귀 측정)**
| Sample | 검증 항목 |
|--------|----------|
| `hwp3-sample16-hwp5.hwp` | page 3-4 한컴 정합 ✓ |
| `hwp3-sample16.hwp` (HWP3 원본) | 페이지 분할 유지 (회귀 0) |
| `aift.hwp` | 회귀 0 |
| `biz_plan.hwp` | 회귀 0 |
| `통합재정통계(2014).hwp` | 회귀 0 |
| `exam_kor/math/eng.hwp` | 시험지 회귀 0 |
| 다른 HWP3 sample (sample10/12/14/18/19) | 회귀 0 |

**4-3. WASM 빌드**
- `wasm-pack build --release --target web --out-dir pkg`
- rhwp-studio 실제 렌더링 확인

**Stage 4 산출물**: `mydocs/working/task_m100_1007_stage4.md` — 회귀 검증 보고서

### Stage 5 — 작업지시자 시각 판정 + 최종 보고서

**5-1. rhwp-studio 시각 판정**
- `samples/hwp3-sample16-hwp5.hwp` page 3-4 한컴 정합 확인
- 다른 sample 회귀 확인

**5-2. 최종 보고서**
- `mydocs/report/task_m100_1007_report.md`
- root cause + fix + 회귀 검증 결과 + 작업지시자 판정 결과

**Stage 5 산출물**: 최종 보고서

## 4. Risk 매트릭스

| 변경 영역 | 영향 sample | 회귀 risk | 완화책 |
|----------|-----------|----------|--------|
| pagination engine | 모든 HWP3/HWP5/HWPX | 높음 | sweep + sample sweep 회귀 0 |
| ParaShape spacing buffer | 변환본만 (`is_hwp3_variant=true`) | 낮음-중간 | variant 식별 휴리스틱 정확도 |
| page-fill threshold | 모든 페이지 분할 | 매우 높음 | sweep + 작업지시자 시각 판정 |

## 5. 잔존 / 분리 (Out of Scope)

- 한컴 의 정확한 layout 공식 알 수 없으므로 100% 정합 어려움 — 휴리스틱 best effort
- Cover page 머릿말 overlap (Task #1006) 별도
- Shape gradient simplify (Task #1008) 별도

## 6. 단계별 산출물 / 커밋 단위

각 stage 완료 시:
- `mydocs/working/task_m100_1007_stage{N}.md` 단계 보고서 + 관련 소스/문서 → 1 커밋
- 작업지시자 승인 후 다음 stage 진행

## 7. 종료 조건

- Sample16-hwp5 page 3-4 한컴 정합 (작업지시자 시각 판정 통과)
- 모든 sweep sample 회귀 0
- cargo test + clippy 통과
- WASM 빌드 통과
- 최종 보고서 작업지시자 승인
