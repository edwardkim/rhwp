# Task #896 구현 계획서 — Stage 1~5

**이슈**: [edwardkim/rhwp#896](https://github.com/edwardkim/rhwp/issues/896)
**수행 계획서**: [task_m100_896.md](task_m100_896.md)
**Scope (작업지시자 확정)**: **차이 1 + 차이 2 모두 본 task**

## 진행 순서

1. **Stage 1** — 차이 1 (paragraph_layout ◦ x 좌표) 정밀 진단
2. **Stage 2** — 차이 1 Fix + 회귀 검증
3. **Stage 3** — 차이 2 (WMF 그림 안 텍스트 겹침) 정밀 진단
4. **Stage 4** — 차이 2 Fix + 회귀 검증
5. **Stage 5** — 통합 검증 + 최종 보고서

## Stage 1 — 차이 1 정밀 진단 (paragraph_layout)

### 1.1 진단 절차

- [ ] paragraph 397 의 SVG x=107.3 의 정확한 산출 흐름 추적
- [ ] paragraph 396 의 SVG x=117.81 과 비교
- [ ] 첫 빈 char_shape (id=1117 spacing=0% char="") 가 paragraph 시작 x 에 미치는 영향 측정
- [ ] paragraph_layout 의 paragraph 시작 x 계산 코드 위치 찾기:
  - 후보 1: `src/renderer/layout/paragraph_layout.rs` 의 `effective_margin_left` 계산
  - 후보 2: `composer.rs` 의 line breaking + start x 계산
  - 후보 3: `text_measurement.rs` 의 첫 char_shape spacing 사용

### 1.2 가설 후보

| 가설 | 검증 방법 |
|------|----------|
| H1: paragraph_layout 이 모든 char_shapes 의 첫 entry 의 spacing 을 paragraph 시작 x 에 반영 | char_shape 의 spacing 변경 후 결과 변화 측정 |
| H2: 첫 빈 char ("") 의 font_size 가 line height 보정에 영향 → indent 변경 | corrected_line_height 흐름 추적 |
| H3: composer 의 line start 가 첫 char_shape 의 spacing 적용 | composer 코드 추적 |

### 1.3 정밀 측정 도구

- `RHWP_TYPESET_DRIFT=1 cargo run --bin rhwp -- export-svg samples/hwp3-sample16.hwp -p 17` — paragraph 별 측정
- 임시 디버그 print: `RHWP_PI397` 등 paragraph index 별

### 1.4 Stage 1 산출물

- `mydocs/working/task_m100_896_stage1.md` — 진단 결과 + 가설 검증 + fix 방향

## Stage 2 — 차이 1 Fix + 회귀 검증

### 2.1 Fix 적용

가설 검증 후 root cause 별 fix 적용:

| 후보 | 처리 |
|------|------|
| H1 fix | 첫 char_shape 가 빈 char ("") 인 경우 spacing 무시 또는 char_shape skip |
| H2 fix | corrected_line_height 의 빈 char 처리 정합 |
| H3 fix | composer 의 line start 계산에서 빈 char 제외 |

### 2.2 회귀 검증

- [ ] `cargo test --release --all-targets` 1355+ passed
- [ ] sample16 페이지 18 paragraph 397/398/399 의 ◦ x 좌표 정합 (paragraph 396 과 같은 x ≈ 117.8)
- [ ] HWP3 sample 6종 페이지 수 + golden SVG 회귀
- [ ] HWP5/HWPX sample 회귀 점검

### 2.3 Stage 2 산출물

- `mydocs/working/task_m100_896_stage2.md` — Fix 결과 + 회귀 점검

## Stage 3 — 차이 2 정밀 진단 (WMF)

### 3.1 진단 절차

- [ ] sample16 paragraph 394 의 picture (WMF `bin_id=3`) 추출
- [ ] WMF binary 의 text record 분석 (`text_out`, `ext_text_out`)
- [ ] rhwp 의 WMF text rendering 결과 vs 한컴 viewer (PDF) 비교
- [ ] 텍스트 겹침 위치 정확히 식별 (Windows 서버군, DMZ 등 영역)

### 3.2 가설 후보

| 가설 | 검증 방법 |
|------|----------|
| H1: WMF text_out 의 x/y 좌표 변환 오류 | point_s_to_absolute_point 흐름 추적 |
| H2: set_text_align (top/baseline) 잘못 적용 | text_align 처리 코드 분석 |
| H3: 한컴 사적 WMF 확장 (font/clipping) | WMF binary record 분석 |
| H4: WMF text clipping 누락 → 다른 element 위에 그려짐 | WMF clip rect 처리 코드 |

### 3.3 정밀 측정 도구

- WMF binary 직접 분석 (hex dump 또는 dump tool)
- WMF → SVG 출력 비교 (rhwp 결과 vs WMF reference renderer)

### 3.4 Stage 3 산출물

- `mydocs/working/task_m100_896_stage3.md` — WMF 진단 결과 + fix 방향

## Stage 4 — 차이 2 Fix + 회귀 검증

### 4.1 Fix 적용

가설 검증 후 root cause 별 fix:

| 후보 | 처리 영역 |
|------|---------|
| H1 fix | `src/wmf/converter/player.rs` 의 좌표 변환 |
| H2 fix | text_align 처리 정합 |
| H3 fix | 한컴 확장 fallback |
| H4 fix | clipping 정합 |

### 4.2 회귀 검증

- [ ] sample16 페이지 18 의 WMF 그림 안 텍스트 정합
- [ ] 다른 WMF 샘플 (sample10, sample14 등) 회귀 점검
- [ ] `cargo test --release --all-targets` 회귀 없음

### 4.3 Stage 4 산출물

- `mydocs/working/task_m100_896_stage4.md` — WMF Fix 결과 + 회귀 점검

## Stage 5 — 통합 검증 + 최종 보고서

### 5.1 통합 검증

- [ ] `cargo test --release --all-targets` 1355+ passed
- [ ] HWP3 sample 6종 페이지 수 회귀 없음
- [ ] HWP5/HWPX 주요 샘플 페이지 수 회귀 없음
- [ ] golden SVG 회귀 없음
- [ ] sample16 페이지 18 시각 정합:
  - ◦ x 좌표 (paragraph 397/398/399)
  - WMF 그림 안 텍스트

### 5.2 최종 보고서

- `mydocs/report/task_m100_896_report.md`

### 5.3 PR 생성

- base: `devel`, head: `jangster77:local/task896`
- PR body 에 두 차이 모두 명시
- `closes #896`

## 위험 평가

| Stage | 위험 | 완화 |
|-------|------|------|
| 1, 2 | paragraph_layout 변경 → 모든 paragraph 영향 | HWP3/HWP5/HWPX sample 다수 회귀 점검 |
| 3, 4 | WMF 변환 변경 → 다른 WMF 샘플 회귀 | 다양한 WMF 샘플 시각 정합 비교 |
| 5 | 통합 영향 | 단계별 누적 검증 |

## 의사결정 요청

본 구현 계획서 자체 승인. 승인 시 Stage 1 (차이 1 paragraph_layout 정밀 진단) 진행.
