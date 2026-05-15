# Task #902 구현 계획서 — WMF unit scale 정합

**이슈**: [edwardkim/rhwp#902](https://github.com/edwardkim/rhwp/issues/902)
**수행 계획서**: [task_m100_902.md](task_m100_902.md)

## Stage 1 — 다중 sample WMF binary + PDF 추출 + pattern 분석

### 1.1 진단 절차

- [ ] **HWP3 sample 의 WMF picture 추출** (sample14/16/17/18/19):
  - 각 HWP3 파일에서 picture 추출 (bin_data, format=WMF)
  - WMF binary 의 첫 N 개 record (SETMAPMODE, SETWINDOWEXT, SETWINDOWORG, SETVIEWPORTEXT 등) parsing 결과 정리
- [ ] **rhwp 의 현재 SVG 출력**:
  - 각 sample 의 페이지 별 export-svg
  - WMF picture 의 `<svg viewBox=...>` + `<text font-size=...>` + 좌표 측정
- [ ] **PDF 의 한컴 정합 측정**:
  - pdf/hwp3-sample{N}-hwp5-2022.pdf 의 WMF 영역 페이지 추출
  - pdftohtml -xml 로 text 좌표/크기 측정
- [ ] **공통 pattern 식별**:
  - WindowExt 와 실 element 좌표의 ratio
  - font-size 의 actual rendering 비례
  - 한컴 정합 ratio (PDF 측정 vs rhwp SVG)

### 1.2 측정 도구

- WMF binary parser (`src/wmf/parser/`) 또는 임시 dump
- `rhwp export-svg sample.hwp -p N` (특정 페이지)
- `pdftohtml -xml -i -f N -l N pdf/sample.pdf /tmp/out`

### 1.3 가설 검증 표

| 가설 | 검증 방법 |
|------|----------|
| H1: 모든 한컴 사적 WMF 가 SetWindowExt 단 1회 + SetViewportExt 미호출 | sample 다수 WMF binary 패턴 검사 |
| H2: WindowExt 와 element 좌표 ratio 가 sample 별 일관 | ratio 계산 + 비교 |
| H3: font-size 정합 ratio 존재 (예: rhwp 출력 × R = PDF 정합) | PDF text size 측정 |

### 1.4 산출물

- `mydocs/working/task_m100_902_stage1.md`
- 측정 결과 표 (sample × WMF metric × rhwp/PDF)

## Stage 2 — Fix 알고리즘 후보 도출 + 결정

### 2.1 Stage 1 결과 기반 Fix 방향 선정

Stage 1 의 pattern 식별 결과에 따라:
- **단순 ratio 발견 시**: γ (font-size scale factor) 또는 viewBox scale 적용
- **WindowExt/ViewportExt 명확한 의미 파악 시**: α (정밀 ratio 처리)
- **viewBox 의 의미 다르게 해석해야 할 시**: β (viewBox 결정 알고리즘)
- **공통 pattern 없을 시**: 부분 정합 또는 won't-fix

### 2.2 산출물

- `mydocs/working/task_m100_902_stage2.md`
- Fix 방향 권고 (작업지시자 승인 후 Stage 3)

## Stage 3 — Fix 적용 + 핵심 정합 검증

### 3.1 Fix 적용

- [ ] 선정된 fix 영역 (`src/wmf/converter/svg/mod.rs` 등) 변경
- [ ] sample16 페이지 18 WMF 텍스트 한컴 정합 확인 (PDF 비교)

### 3.2 핵심 회귀 검증

- [ ] `cargo test --release --all-targets`: 1411+ passed 유지
- [ ] 다른 WMF 사용 sample 페이지 수 / SVG 회귀:
  - sample14 (Task #860 fixture)
  - sample17/18/19
  - 기존 WMF 회귀 paper sample
- [ ] golden SVG 회귀

### 3.3 산출물

- `mydocs/working/task_m100_902_stage3.md`

## Stage 4 — 광범위 회귀 검증

### 4.1 전체 sample 점검

- [ ] `samples/` 모든 WMF 포함 sample SVG 비교 (before/after)
- [ ] 시각 회귀 점검 (debug-overlay 또는 PDF 비교)
- [ ] sample 별 정합 / 회귀 분류 결과 표

### 4.2 산출물

- `mydocs/working/task_m100_902_stage4.md`

## Stage 5 — 통합 + 최종 보고서 + PR

- [ ] 최종 보고서 (`mydocs/report/task_m100_902_report.md`)
- [ ] orders 갱신
- [ ] PR 생성 (작업지시자 승인 후)
- [ ] issue #902 회신

### 산출물

- `mydocs/report/task_m100_902_report.md`

## 위험 평가

| Stage | 위험 | 완화 |
|-------|------|------|
| 1 | WMF binary parser 직접 접근 부족 → 측정 정확도 | 임시 dump 도구 또는 기존 `src/wmf/parser/` 활용 |
| 2 | 공통 pattern 부재 → 단일 fix 알고리즘 도출 불가 | 부분 정합 또는 won't-fix 결정 |
| 3 | WMF 처리 변경 → 다른 sample 회귀 | 다중 sample 회귀 점검 단계별 |
| 4 | golden SVG 회귀 | sample 별 우선순위 + 결과 분류 |

## 의사결정 요청

본 구현 계획서 자체 승인. 승인 시 Stage 1 (다중 sample WMF binary + PDF 추출 + pattern 분석) 진행.
