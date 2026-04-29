# Task #455 최종 결과 보고서

## 타이틀

인라인 글상자(treat_as_char + wrap=TopAndBottom) 가 있는 문단 줄의 본문 텍스트 누락 수정

## 증상

`samples/exam_kor.hwp` 페이지 2 좌측 단의 첫 문단(pi=33) 두 번째 줄에서 본문 39자가 누락되고, 인라인 글상자 내부 "개화" 두 글자만 단 가운데(x≈428.8)에 표시됨. 본문이 1줄→3줄로 건너뛰는 비문이 되어 좌·우 단 정렬이 어긋나 보임.

## 원인

`src/renderer/layout/paragraph_layout.rs` 의 `skip_text_for_inline_shape` 플래그가 인라인 글상자(Shape with TextBox + treat_as_char) 가 있는 줄에서 외부 문단 본문 텍스트 렌더를 통째로 스킵하고 있었음.

원래 의도는 "글상자 내부 텍스트가 별도 패스에서 렌더되므로 여기서는 스킵" 이었으나, 여기서 스킵된 텍스트는 글상자 *외부* 의 문단 본문이지 글상자 *내부* 가 아니었다. 글상자 내부 텍스트(예: "개화")는 `shape_layout.rs:218` 의 `tree.get_inline_shape_position` 경로로 한 번만 렌더되므로 외부 본문을 항상 렌더해도 중복되지 않는다.

## 수정

`src/renderer/layout/paragraph_layout.rs`:
- `skip_text_for_inline_shape` 변수 정의 + 두 곳의 `if !skip_text_for_inline_shape { ... }` 가드 제거.
- 외부 문단 본문 텍스트 세그먼트(글상자 좌·우)를 무조건 렌더하도록 변경.
- 주석에 Task #455 표시.

## 검증

### 좌측 단 y 좌표별 텍스트 (수정 전 → 후)

수정 전:
```
y=295.1 "서양의 과학과 기술... 이항로를 비롯한"
y=321.4 "개화"        ← 본문 39자 누락
y=347.6 "수없는 대세로 자리잡았다..."
```

수정 후:
```
y=295.1 "서양의 과학과 기술... 이항로를 비롯한"
y=321.4 "개화"        ← 글상자 (별도 패스, 그대로)
y=322.6 "척사파의 주장은 개항 이후에도 지속되었지만, 는 거스를"   ← 본문 24+5자 복원 (글상자 좌·우 분리)
y=347.6 "수없는 대세로 자리잡았다..."
```

### 페이지 수

| 샘플 | 결과 |
|------|------|
| `exam_kor.hwp` | 20 (변동 없음) |
| `exam_eng.hwp` | 8 |
| `2010-01-06.hwp` | 6 |
| `exam_math_8.hwp` | 1 |
| `biz_plan.hwp` | 6 |
| `draw-group.hwp` | 1 |
| `atop-equation-01.hwp` | 1 |
| `equation-lim.hwp` | 1 |

### 단위 테스트

`cargo test --release`: 1117 passed, 1 ignored, 0 failed.

## 알려진 미세 차이

- 본문 글자 baseline y=322.6, 글상자 내부 "개화" baseline y=321.4 — **1.2px** 차이.
- 줄 높이가 5mm 글상자에 맞춰 1417 HU 로 늘어난 상황에서 baseline 정렬을 본문 폰트 기준으로 다시 계산하지 않아 생기는 미세 어긋남. 시각상 큰 문제는 아니나 후속 개선 여지 있음 (본 타스크 스코프 외, 별도 이슈로 분리 가능).

## 변경 파일

- `src/renderer/layout/paragraph_layout.rs` — `skip_text_for_inline_shape` 분기 제거.
- `mydocs/plans/task_m100_455.md`
- `mydocs/plans/task_m100_455_impl.md`
- `mydocs/working/task_m100_455_stage1.md`
- `mydocs/working/task_m100_455_stage2.md`
- `mydocs/working/task_m100_455_stage3.md`
- `mydocs/report/task_m100_455_report.md`
- `mydocs/orders/20260429.md` — Task #455 항목 추가.

## 커밋

- `d87af59` Task #455 Stage 2: 인라인 글상자(tac=true + TextBox) 외부 본문 텍스트 누락 수정
