# 구현계획서 — Task M100 #1313

적분기호(∫) 상·하한이 위아래로 벌어지고 기호 높이가 정답보다 작음

- 이슈: edwardkim/rhwp#1313
- 브랜치: `local/task1313`
- 수행계획서: `mydocs/plans/task_m100_1313.md` (승인 완료)

## 측정 기준 (조사 완료, 96dpi 픽셀)

`samples/3-10월_교육_통합_2022.hwp` p.9 첫 줄 적분 기준:

| 항목 | 정답(한글2022) | 우리 렌더 | 비고 |
|------|---------------|-----------|------|
| ∫ 글리프 세로 높이 | ~23px (≈1.9×fs) | ~16px (≈1.33×fs) | 우리가 작음 |
| 상한 "4" | 적분 상단부에 겹쳐 위치 | 적분 윗끝 위로 떠 있음 | 위로 벌어짐 |
| 하한 "0" | 적분 하단 끝에 근접 | 적분 아랫끝 아래로 떨어짐 | 아래로 벌어짐 |

(fs = 식 본문 글자크기 12px)

## 근본 원인

1. **글리프 크기**: `op_fs = fs * BIG_OP_SCALE`(1.5) → 적분 글리프가 정답(≈1.9×fs)보다 작다.
2. **상·하한 오프셋 불일치**: `sup_shift = op_h*0.1`, `sub_shift = op_h*0.55` 가 실제(작은)
   글리프 높이와 어긋나 상·하한이 빈 공간에 떠 보인다.
3. **상수 공유**: `BIG_OP_SCALE` 를 ∑/∏ 와 공유하므로 적분만 키우려면 상수 분리가 필요하다.

## 수정 대상 (적분 관련 5개 지점)

| 파일 | 위치 | 내용 |
|------|------|------|
| `layout.rs` | `layout_math_symbol()` 311행 | bare 적분 글리프 op_fs/baseline |
| `layout.rs` | `layout_integral()` 703행 | op_fs, op_baseline, sup_shift, sub_shift, script_x |
| `svg_render.rs` | BigOp integral 분기 236행 | op_y, 첨자 배치 |
| `canvas_render.rs` | BigOp integral 분기 170행 | op_fs, op_y (SVG와 동기화) |
| `skia/equation_conv.rs` | BigOp integral 분기 347행 | op_fs, op_y (동기화) |

> 3개 렌더 경로(svg/canvas/skia)는 **반드시 동일 수식**으로 유지한다. HWP3 전용 분기 아님.

## 구현 단계

### 1단계 — 적분 전용 스케일 상수 분리 + 글리프 높이 확대
- `layout.rs` 에 `INTEGRAL_SCALE` 상수 신설 (초기값 ≈ 2.0~2.15, 측정 기반 튜닝).
- 적분 경로(`layout_math_symbol`, `layout_integral`, 3개 렌더 경로)의 `op_fs` 계산을
  `BIG_OP_SCALE` → `INTEGRAL_SCALE` 로 교체. **∑/∏ 등은 `BIG_OP_SCALE` 그대로 유지.**
- 글리프 baseline/op_y(`op_fs*0.8`, `op_fs*0.7` 등) 비율을 새 크기에 맞게 조정.
- **검증**: ∫ 글리프 세로 높이가 정답(~23px)에 근접. ∑/∏ 렌더는 변화 없음(다른 샘플 대조).

### 2단계 — 상·하한을 글리프 끝에 밀착 재조정
- `layout_integral()` 의 `op_baseline`, `sup_shift`, `sub_shift`, `script_x` 를 새 글리프
  실제 상단/하단 좌표 기준으로 재계산 → 상한은 윗끝, 하한은 아랫끝에 밀착.
- `svg_render.rs` / `canvas_render.rs` / `skia/equation_conv.rs` 의 op_y·첨자 배치 동기화.
- `BIG_OP_TRAIL_PAD`(적분 뒤 피연산자 간격)·전체 식 baseline 정렬과의 상호작용 확인.
- **검증**: p.9 적분 상·하한이 정답처럼 기호 끝에 밀착.

### 3단계 — 회귀·시각 정합 검증
- `rhwp export-svg samples/3-10월_교육_통합_2022.hwp -p 8` → 정답 PDF p.9 픽셀 크롭 비교.
- 다른 페이지/샘플의 적분식, ∑/∏ 등 큰 연산자 회귀 점검.
- 기존 수식 테스트(`issue_1139`, `issue_1219`, `issue_1061` 등) 포함 `cargo test` 전체 통과.
- `cargo build --release` 성공.
- 필요 시 회귀 방지용 테스트(적분 상·하한 위치/글리프 높이) 추가.

## 단계별 산출물
- 각 단계 완료 시 `task_m100_1313_stage{N}.md` 작성 + 해당 소스 커밋(타스크 브랜치).
- 전 단계 완료 후 `report/task_m100_1313_report.md` 최종 보고서.

## 리스크 / 대응
- **∑/∏ 동반 변경**: `INTEGRAL_SCALE` 분리로 차단, 다른 큰 연산자 샘플 대조로 확인.
- **3경로 불일치**: 동일 수식·동일 상수 사용, 단계마다 SVG 기준으로 canvas/skia 점검.
- **baseline·trailing 상호작용**: 1·2단계 분리 진행, 단계별 시각 검증으로 조기 발견.

## 검증 명령
```bash
cargo build --release
cargo test
./target/release/rhwp export-svg samples/3-10월_교육_통합_2022.hwp -p 8 -o output/svg/
# → output/svg/3-10월_교육_통합_2022_009.svg 를 정답 pdf/3-10월_교육_통합_2022.pdf p.9 와 비교
```
