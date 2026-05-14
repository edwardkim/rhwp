# Task #901 구현 계획서 — Stage 1~5

**이슈**: [edwardkim/rhwp#901](https://github.com/edwardkim/rhwp/issues/901)
**수행 계획서**: [task_m100_901.md](task_m100_901.md)
**Scope**: pic2.hwp 페이지 1 layout 깨짐 — Square wrap 그림 옆 텍스트 + 누적 height

## Stage 1 — paragraph 0 layout 정밀 진단

### 1.1 진단 절차

- [ ] paragraph 0 의 8 line_seg 의 cs/sw 분석:
  - ls[0] cs=24470 sw=2570 vs ls[1] cs=39123 sw=3397 — 두 wrap zone 영역
  - 그림 2개 (82.5×104.4mm Square @ x=33.8mm + 42.6×53.9mm Square @ x=125.4mm) 와 매칭
- [ ] paragraph 0 의 char 별 SVG x/y 좌표 추출 (`grep -oE '<text [^>]+y="[0-9]+...` `/tmp/pic2/pic2_001.svg`)
- [ ] 한컴 PDF (`pdf/pic2.pdf`) 페이지 1 의 "우/리/나/라" char x/y 좌표 추출 (`pdftohtml -xml`)
- [ ] 차이 정량 측정 + 시각 정합 확인

### 1.2 가설 후보

| 가설 | 검증 |
|------|------|
| H1 | line_seg cs/sw 가 wrap zone 의 좌측 빈 공간 (그림 사이) 의미인데 rhwp 가 다른 의미로 해석 |
| H2 | 같은 ts (예: ts=34/34, 35/35, 36/36) 의 2 line 처리 시 두번째 line 의 cs/sw 무시 |
| H3 | composer 가 paragraph 의 text 를 wrap zone 좁은 영역에 word-wrap 시 잘못 break |

### 1.3 정밀 측정 도구

- `cargo run --release --bin rhwp -- dump-pages samples/pic2.hwp -p 0` — paragraph 별 vpos/h
- `cargo run --release --bin rhwp -- export-svg samples/pic2.hwp -p 0 -o /tmp/pic2/`
- `pdftohtml -xml -i -f 1 -l 1 pdf/pic2.pdf /tmp/pic2-pdf1`
- 임시 디버그 print: `RHWP_PARA0` 환경변수 + paragraph 0 layout 흐름 추적

### 1.4 산출물

- `mydocs/working/task_m100_901_stage1.md`

## Stage 2 — wrap zone / Square wrap 코드 분석

### 2.1 진단 영역

- `src/renderer/typeset.rs` — wrap_around_cs / wrap_around_sw 처리
- `src/renderer/composer.rs` — line breaking + wrap zone 안 char 배치
- `src/renderer/layout/paragraph_layout.rs` — paragraph 의 char 배치 (line_seg 별)
- `src/renderer/layout/shape_layout.rs` — Square wrap 그림 배치

### 2.2 추적 항목

- [ ] paragraph 0 의 line_seg cs/sw 가 typeset 의 wrap_around 처리에 어떻게 전달
- [ ] composer 의 char 별 x 좌표 결정 시 cs/sw 사용
- [ ] paragraph_layout 의 line 별 x 시작 좌표 (= cs) 적용 확인

### 2.3 산출물

- `mydocs/working/task_m100_901_stage2.md`

## Stage 3 — Fix 적용

### 3.1 가설 별 fix 방향

| 가설 | 처리 영역 |
|------|---------|
| H1 fix | line_seg cs/sw 해석 정정 |
| H2 fix | 같은 ts 의 multi-line wrap 처리 |
| H3 fix | composer 의 word-wrap 알고리즘 |

### 3.2 회귀 검증

- [ ] `cargo test --release --all-targets` 회귀 없음
- [ ] pic2.hwp 페이지 1 layout 정합 (한컴 PDF 와 시각 비교)
- [ ] HWP3/HWP5/HWPX sample 페이지 수 회귀 없음
- [ ] golden SVG 회귀 없음

### 3.3 산출물

- `mydocs/working/task_m100_901_stage3.md`

## Stage 4 — 추가 회귀 검증 (Square wrap sample)

- [ ] Square wrap 사용 다른 sample 회귀 점검 (`samples/*-square-*` 등)
- [ ] HWPX 변환본 의 Square wrap 영향
- [ ] pic2.hwp 페이지 2 정합 (이미 거의 정합)

### 산출물

- `mydocs/working/task_m100_901_stage4.md`

## Stage 5 — 통합 + 최종 보고서

- [ ] cargo test --release --all-targets 1398+ passed
- [ ] pic2.hwp 페이지 1+2 한컴 PDF 정합
- [ ] 최종 보고서 + PR 생성

### 산출물

- `mydocs/report/task_m100_901_report.md`

## 위험 평가

| Stage | 위험 |
|-------|------|
| 1, 2 | 진단 깊이 큼 (typeset/composer/layout 다중 영역) |
| 3 | wrap zone 처리 변경 → 모든 wrap 사용 sample 영향 매우 높 |
| 4 | 회귀 검증 자료 (Square wrap sample) 다소 부족 |

## 의사결정 요청

본 구현 계획서 자체 승인. 승인 시 Stage 1 (paragraph 0 layout 정밀 진단) 진행.
