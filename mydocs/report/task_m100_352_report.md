# Task #352 최종 결과 보고서

> 2026-04-28 | Branch: `local/task352`
> Issue: [#352](https://github.com/edwardkim/rhwp/issues/352)
> Milestone: v1.0.0

---

## 요약

`samples/exam_eng.hwp` 5 페이지 32번 문항의 빈칸 dash 시퀀스(29 개 ASCII `-`) 가 PDF 대비 ~3.5 배 부풀어 후속 텍스트 "of being 'stimulus-driven',"가 우측 단 경계를 넘어 잘리는 문제를 해소.

**원인**: HY신명조 ASCII 하이픈 메트릭이 비정상적으로 넓음(0.83 em). 이슈 본문이 추정한 Justify Branch B 가산 가설은 사실이 아님 — Branch A 정상 발동했으나 자연 폭 자체가 사용 가능 폭을 초과하여 압축 한도로도 회수 불가.

**수정**: 3 개 이상 연속 dash 시퀀스(leader 패턴) 만 식별해 좁은 advance(`font_size × 0.32`) 적용 + SVG/Canvas 에서 dash 글리프 스킵 + underline 부재 시 단일 가로선 렌더.

---

## 단계별 산출

| Stage | 커밋 | 산출 |
|-------|------|------|
| 1 | `428b01d` | 원인 확정 (eprintln + 코드포인트 분석) |
| 2 | `69e420b` | leader-aware 좁은 advance (text_measurement.rs) |
| 3 | `2248752` | 시각 라인 통합 + 폭 미세 조정 (svg.rs, web_canvas.rs) |

---

## 변경 파일

### 코드
- `src/renderer/layout/text_measurement.rs`
  - `is_dash_leader_run(chars, i) -> bool` 헬퍼 추가
  - 3 개 `char_width` 클로저에 leader-aware 좁은 폭 분기 (`font_size × 0.32`)
- `src/renderer/svg.rs`
  - dash run pre-pass + 글리프 스킵 + 단일 라인 렌더 (underline 부재 시)
- `src/renderer/web_canvas.rs`
  - 동일 패턴 (WASM/canvas 경로)

### 문서
- `mydocs/plans/task_m100_352.md` — 수행계획서
- `mydocs/plans/task_m100_352_impl.md` — 구현계획서
- `mydocs/troubleshootings/issue_352_root_cause.md` — 원인 분석
- `mydocs/working/task_m100_352_stage{1,2,3}.md` — 단계별 보고서
- `mydocs/orders/20260428.md` — 오늘 할일 갱신
- `mydocs/report/task_m100_352_report.md` — 본 문서

---

## 검증 결과

### Q32 블랭크 라인 측정 비교

| 항목 | 수정 전 | 수정 후 | PDF 목표 |
|------|---------|---------|----------|
| dash advance / 글자 | 12.11 px | 4.65 px | ~3.5 px (근사) |
| 29 dash 시퀀스 폭 | 351 px | **134.94 px** | ~135 px ✓ |
| 화면상 가로선 수 | 2 (dash bar 겹침 + underline) | **1** (underline 만) | 1 |
| dash 글리프 잔존 | 29 | **0** (스킵) | (PDF 글리프 미시) |
| `of being` 시작 x | 953 (단 우측 끝) | 839 (단 중앙쯤) | 단 중앙 |
| `,` 우측 잘림 | 발생 | **해소** | 정상 |

### cargo test --release

전 테스트 통과:
- 메인 라이브러리: 1023 passed / 0 failed / 1 ignored
- svg_snapshot: 6/6 passed
- tab_cross_run: 1/1 passed
- 기타 통합: 50+ passed

### 다른 샘플 회귀 점검

| 샘플 | dash 글리프 수 (after fix) | 비고 |
|------|---------------------------|------|
| exam_eng | 366 | 정상 (단발 dash 보존) |
| exam_kor | 440 | 정상 |
| exam_math_8 | 1 | 정상 |
| aift | 460 | 정상 |
| biz_plan | 13 | 정상 |

`is_dash_leader_run` 의 ≥ 3 조건 덕분에 단발 dash(예: "stimulus-driven", "32.-", "* taint--altruistic--") 는 영향 없음.

---

## 작업지시자 피드백 대응

Stage 3 단계에서 다음 피드백 받음:

1. **"왜 2줄로 그려지나?"**
   - 원인: dash leader 라인(y = baseline − 0.32 em) 과 char_shape underline(y = baseline + 2 px) 동시 출력
   - 대응: `suppress_dash_leader_line = !style.underline.is_none()` 가드 추가. underline 있는 run 은 dash leader 라인 생략.

2. **"폭이 조금 짧음"**
   - 원인: 0.3 em 적용 시 29 × 4.36 = 126 px (PDF ~135 px 보다 9 px 부족)
   - 대응: 0.3 em → **0.32 em** 으로 미세 조정. 134.94 px ≈ PDF 일치.

---

## 비포함 / 백로그

- HWP `cs` (line_seg.cs) 필드 활용 — 별 이슈 (현재 사용 안 함이 확인됨)
- `_____` underscore 등 다른 leader 글자 일반화 — 현 시점 우선순위 낮음
- `font_metrics_data` 의 HY신명조 dash 메트릭 자체 보강 — 글리프 폭 0.83 em 은 폰트 자체의 실제값일 가능성. 현 leader-aware 우회로 충분.
- `–` (U+2013 EN DASH), `—` (U+2014 EM DASH) leader 처리 — 본 이슈 범위 외. 추후 필요시 `is_dash_leader_run` 에 추가 가능.

---

## merge 계획

- `local/task352` (3 commits ahead of devel) → `local/devel` merge 후 `devel` 로 push
- closes #352 추가
