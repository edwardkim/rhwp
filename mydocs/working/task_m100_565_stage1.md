# Task #565 Stage 1 — 정밀 진단 보고서

- **이슈**: [#565](https://github.com/edwardkim/rhwp/issues/565)
- **단계**: Stage 1 (정밀 진단, **코드 무수정**)
- **작성일**: 2026-05-04

## 1. 진단 절차

1. `samples/exam_science.hwp` 페이지 2 SVG 출력 (`rhwp export-svg samples/exam_science.hwp -p 1`)
2. 12번 문제(문단 0.61) IR 덤프 (`rhwp dump`)
3. SVG 파일 내 모든 `<g transform="translate(x,y)">` 좌표 추출 + 인라인 수식 텍스트 매칭
4. `paragraph_layout.rs` 의 인라인 수식 처리 경로 코드 정독

## 2. 핵심 발견 — 9개 수식이 동일 좌표에 겹쳐 그려짐

12번 문제 본문의 9개 인라인 수식이 SVG에서 모두 **정확히 동일한 (x, y) = (534.8, 1218.106)** 좌표에 그려짐:

```
translate(534.8,1218.106666666667)  → "X"
translate(534.8,1218.106666666667)  → "A"
translate(534.8,1218.106666666667)  → "B"
translate(534.8,1218.106666666667)  → "C"
translate(534.8,1218.106666666667)  → "D"
translate(534.8,1218.106666666667)  → "m-4"
translate(534.8,1218.106666666667)  → "m-2"
translate(534.8,1218.106666666667)  → "m+2"
translate(534.8,1218.106666666667)  → "m+4"
```

**대조**: 같은 페이지 다른 문제의 인라인 수식들은 정상 분산:

```
translate(774.8,152.586) → "A"   translate(802.8,152.586) → "B"   (x 28 px 분리)
translate(570.89,1232.54) → "67"   translate(653.27,1232.54) → "78"   (보기 표 셀 내부 — 정상)
```

→ **EquationNode 자체는 생성되지만, 9개 모두 동일한 시작 좌표를 받음**. SVG 렌더 코드(`svg.rs:341-368`) 는 `node.bbox.x/y` 를 그대로 `<g transform="translate(...)">` 로 사용하므로 결함은 **`paragraph_layout.rs` 단계의 BoundingBox 좌표 산출**에 있음.

## 3. IR 정상 확인

`rhwp dump samples/exam_science.hwp -s 0 -p 61` 결과:

- 문단 0.61: cc=137, text_len=56, controls=10 (표 1 + 수식 9)
- ls[0] ts=0, vpos=74118, lh=2864 (큰 lh — 인라인 TAC 표 포함)
- ls[1] ts=13, vpos=77442, lh=1150
- ls[2] ts=60, vpos=79052, lh=1150
- 수식 컨트롤 모두 정상:
  - [1] rmX 675x1125, [2] rmA 825x1125, [3] rmB 675x1125, [4] rmC 675x1125, [5] rmD 750x1125
  - [6] m-4, [7] m-2, [8] m+2, [9] m+4 모두 2558x1125

→ **IR 단계 결함 없음**. 좌표 산출 결함만 남음.

## 4. 코드 분기 — 인라인 수식 처리 경로 매트릭스

`src/renderer/layout/paragraph_layout.rs` 에 인라인 `Control::Equation` 처리가 **2 경로** 존재:

### 경로 A — L1830-1885 (`run_tacs` 루프 내부)

위치: `for &(tac_rel, tac_w, tac_ci) in &run_tacs` 루프 내 (L1734~1941).

좌표:
- `BoundingBox::new(x, eq_y, tac_w, eq_h)` (L1879)
- `x` 는 sub-segment 텍스트 폭 (`x += seg_w`, L1773) 과 tac 폭 (`x += tac_w`, L1939) 으로 누적
- 누적 정상

활성 조건: 해당 run 안에 1개 이상의 tac 가 있고 (`!run_tacs.is_empty()`), 그 tac 가 `Control::Equation` 인 경우.

### 경로 B — L2274-2323 (`comp_line.runs.is_empty()` 가드)

위치: L2245 `if comp_line.runs.is_empty() && !tac_offsets_px.is_empty()` 분기 내.

좌표:
- `BoundingBox::new(inline_x, eq_y, tac_w, eq_h)` (L2318)
- `inline_x = effective_col_x + effective_margin_left + align_offset` (L2268, line 시작)
- 각 수식 처리 후 `inline_x += tac_w` (L2322)
- 누적 정상

활성 조건: 해당 line 의 `comp_line.runs` 가 비어있고 (텍스트 run 없음) tac만 있는 경우. Task #287 케이스 (디스플레이 수식이 자체 LINE_SEG 가질 때).

### 두 경로 모두 누적 코드는 정상으로 보임

코드 정독 기준 두 경로 모두 좌표 누적은 정상. 그럼에도 SVG 결과는 9개 동일 좌표 → **두 경로 중 어느 게 실제 활성인지 / 활성 경로가 어떤 변수 전달 단계에서 좌표 누적을 잃는지** 디버그 로그(임시)로 정밀 추적 필요.

## 5. 추가 의심 시나리오

| # | 가설 | 검증 방법 |
|---|------|----------|
| H1 | 12번 문제의 line 분할에서 각 수식이 **별도 ComposedLine** 으로 분리 → 9개 line_node 모두 동일 시작 inline_x 사용 | 디버그 로그로 `comp_line` 개수와 각 line 의 `runs.is_empty()` / tac_offsets_px 분포 확인 |
| H2 | composer.rs `find_control_text_positions` 가 9개 수식의 텍스트 위치를 모두 동일 pos 로 산출 → run_tacs/tac_offsets_px 누적 안 됨 | 디버그 로그로 tac_controls 의 (pos, width) 출력 |
| H3 | run 분할 결과 `run_tacs` 가 9개 모두 같은 run 에 들어가지 못하고 별도 run 에서 처리됨 → 매 run 시작마다 `x` 가 리셋(=line 시작 x)되는데 sub-segment 텍스트 폭이 누적되지 않음 | 디버그 로그로 run 개수와 `run_tacs` 분포, 각 run 시작 시점의 x 값 확인 |

## 6. 회귀 위험 후보 영역 (Stage 3 sweep 대상)

- 인라인 수식 사용 다른 샘플:
  - `samples/exam_kor*.hwp`, `samples/exam_science.hwp` 다른 문제, `samples/issue-505-equations.hwp`
- 표 셀 내부 인라인 수식 (12번 표의 ① m+/m- 셀들 — 현재 SVG 정상 분산 확인됨)
- 디스플레이 수식이 자체 LINE_SEG 갖는 케이스 (Task #287 회귀 방지)

## 7. Stage 2 진입 권고 — 디버그 로그 임시 추가

본질 정정 방향(분기 통합 / 경로별 조건 정정 / 좌표 산출 보강) 결정 전에 **임시 디버그 로그(`eprintln!`)** 를 두 경로에 추가하여:

1. 12번 문단(0.61) 의 `comp_line` 개수와 각 line 의 `runs.len()`, `tac_offsets_px.len()` 출력
2. 경로 A 활성 시: 매 tac 처리 시점의 `(x, tac_w, tac_ci)` 출력
3. 경로 B 활성 시: 매 tac 처리 시점의 `(inline_x, tac_w, tac_ci)` 출력
4. 결과로 어느 경로가 활성이며 좌표 누적이 어디서 깨지는지 정확히 식별

확정 후 디버그 로그 제거 + 본질 정정 안 1-3개 비교하여 `task_m100_565_impl.md` 작성.

## 8. 승인 요청

본 진단 보고대로 **Stage 2 (구현 계획서 작성) 진입을 승인 요청**합니다. 시작 시점에 디버그 로그 임시 추가가 포함되며, 본질 정정 방향 확정 후 디버그 코드는 모두 제거합니다.
