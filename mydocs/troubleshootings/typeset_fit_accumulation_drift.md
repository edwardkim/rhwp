# Typeset fit 누적 drift + 단독 항목 페이지 — 트러블슈팅 (Task #359)

## 증상

`samples/k-water-rfp.hwp` 페이지 3 등에서 LAYOUT_OVERFLOW 경고:
```
LAYOUT_OVERFLOW: page=0, col=0, para=34, type=FullParagraph, y=1288.0, bottom=1028.9, overflow=259.1px
```

`samples/kps-ai.hwp` 의 빈 페이지 (페이지 10 단독 빈 문단) + 단독 텍스트 페이지 (페이지 35 의 pi=317 한 항목).

## 원인

### 1. fit 누적의 trailing_ls 누락 (drift origin)

`typeset.rs::typeset_paragraph` 의 fit 분기에서 `current_height += fmt.height_for_fit` 으로 누적. `height_for_fit = total_height - trailing_ls`. N items 누적 시 N × trailing_ls 만큼 누적이 적게 되어 layout 단계의 실제 y 진행과 어긋남.

k-water-rfp p3: 36 items × 평균 ~9px = ~311px LAYOUT_OVERFLOW.

### 2. 단독 항목 페이지 (vpos-reset 가드와 fit 안전마진의 상호작용)

`LAYOUT_DRIFT_SAFETY_PX = 10.0` 안전마진이 fit 판정을 보수적으로 만들어, 0.x px 차이로 fit 실패한 항목이 단독으로 새 페이지 시작. 그 다음 pi 가 vpos-reset 가드 (`first_vpos=0 + prev_last_vpos>5000`) 로 또 새 페이지 → 단독 항목 빈 페이지 발생.

## 해결

`src/renderer/typeset.rs`:

### 1. fit 판정과 누적 분리
- fit 판정: `height_for_fit` (trailing_ls 제외) — 마지막 항목의 trailing_ls 는 페이지 끝에서 의미 없음
- 누적: `total_height` (full) — 다음 항목의 시작 위치 계산용

### 2. 단독 항목 페이지 차단 가드
다음 pi 의 `first_vpos=0` + 현재 pi 의 `last_vpos>5000` (vpos-reset 가드 발동 예정) 시:
- 현재 pi 가 빈 문단 → skip
- 현재 pi 가 일반 텍스트 → fit 안전마진 (10px) 1회 비활성화

### 3. 가드 제외 조건
다음 pi 가 `column_type == Page/Section` (force_page_break) 이면 가드 발동 안 함. 정상 쪽나누기 신호이므로 단독 페이지 발생하지 않음 (hwp-multi-001 회귀 차단).

## 재발 방지 체크리스트

페이지네이션 누적 로직 변경 시:
- [ ] fit 판정과 누적 갱신은 다른 의미를 가지는지 확인
  - fit 판정: 항목이 페이지에 들어가는지 (마지막 자리 trailing_ls 무의미)
  - 누적: 다음 항목 시작 위치 (full height 필요)
- [ ] 안전마진 (LAYOUT_DRIFT_SAFETY_PX) 변경 시 vpos-reset 가드와 상호작용 점검
- [ ] hwp-multi-001 의 force_page_break + vpos-reset 동시 케이스 회귀 테스트

vpos-reset 기반 가드 추가 시:
- [ ] force_page_break (column_type=Page/Section) 케이스 제외
- [ ] 단독 항목 페이지 발생 가능성 점검
- [ ] 여러 샘플 (kps-ai, hwp-multi-001, k-water-rfp) 회귀 비교

## 진단 도구

- `RHWP_TYPESET_DRIFT=1`: per-pi typeset drift trace
- `RHWP_TYPESET_DRIFT_LINES=1`: per-line 분해
- `dump-pages -p N`: 페이지 N 의 항목 + used/hwp_used 비교
- `dump -s S -p P`: 문단의 ParaShape, LINE_SEG, 표 속성 상세

## 관련 task

- Task #359 (본 task)
- Task #321~#332 (initial vpos-reset 가드 도입)
- Task #347 (좌표 정합)
- Task #342 (typeset_layout_drift_analysis)
