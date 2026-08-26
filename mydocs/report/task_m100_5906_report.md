# task_m100_5906 처리결과 — 빈 host 자리차지 표의 마지막 행이 저장 선언 초과분을 흡수한다

- 이슈: [#5906](https://github.com/edwardkim/rhwp/issues/5906)
- 기준: `72674c565` (origin/devel)
- 재현 문서·정답지 모두 저장소 안: `samples/float-stack-defer.hwp` · `pdf/float-stack-defer-2022.pdf`

## 1. 결함

`samples/float-stack-defer.hwp` 는 한글 2022 정본이 **2쪽**인데 rhwp 가 **3쪽**을 낸다.
2쪽에 통째로 들어가야 할 12행 표가 10행/2행으로 갈려 마지막 두 행이 빈 3쪽으로 밀린다.

```
$ rhwp dump-pages samples/float-stack-defer.hwp
문서 로드: samples/float-stack-defer.hwp (3페이지)
=== 페이지 2 ===  PartialTable pi=1 ci=1 rows=0..10  cont=false 12x3
=== 페이지 3 ===  PartialTable pi=1 ci=1 rows=10..12 cont=true  12x3
```

`tests/fixtures/render_page_samples.tsv` 에도 `2 / 3 / delta +1` 로 등재돼 있던 문서다.

문서 구조는 단순하다. 문단 0 = 제목, 문단 1 = **빈 host** 가 자리차지(TopAndBottom)·문단 기준
float 표 두 개(11행, 12행, 둘 다 쪽나눔=나눔(행 단위))를 소유한다.

## 2. 원인 — 마지막 행 하나만 6.89px 크다

2쪽 표의 괘선 경계를 정본 PDF 와 1:1로 대조하면 **앞 11행은 ±1px 로 일치**하고 마지막 행만 다르다.

| | 정본(pt) | rhwp 전(px) | 정본(px 환산) | 차 |
|---|---|---|---|---|
| 표 상단 | 90.86 | 119.4 | 121.2 | −1.8 |
| 행1~행9 경계 9개 | 192.7 … 663.35 | 255.3 … 883.5 | 256.9 … 884.5 | −1.6 ~ −1.0 |
| 행10 하단 | 717.78 | (3쪽) 956.2 상당 | 957.0 | −0.8 |
| **행11 높이** | **52.86pt = 70.48px** | **77.37px** | 70.48px | **+6.89** |
| 표 총높이 | 679.78pt | 914.24px | 906.4px | +6.9 |

rhwp 측정 진단(임시 계기):

```
pi=1 ci=1 rows=12 common=907.347 raw=914.240 shrunk=true emptyhost=true pb=RowBreak
  row11 measured=77.373  declared(cellSz)=77.373  content floor=40.560
```

- 표의 저장 선언높이 `hp:sz` = 68,051HU = **907.347px**. 정본의 실측 총높이 679.78pt =
  67,978HU 가 바로 이 값이다.
- rhwp 측정 합 = **914.240px** → 초과 **6.893px**.
- 마지막 행은 저장 `cellSz`(5,803HU = 77.373px)를 그대로 쓴다. 그 안의 저장 줄 내용은
  40.56px 뿐이라 **36.8px 의 여유**가 있다.
- 정본의 마지막 행 70.48px = 77.373 − 6.893. **초과분과 정확히 같은 양**만 줄어 있다.

그 6.893px 가 본문 바닥(1,031.81px)을 1.6px 넘겨 쪽나눔을 부른다.

왜 아무도 회수하지 않는가:

- 페인트 경로 `LayoutEngine::fit_row_heights_to_common_height` 는 이미 같은 보정을 **키우는
  방향으로만** 한다 — `residual > 0.5` 이면 남는 몫을 마지막 행에 몰아준다. 줄이는 방향은 버린다.
- 조판 경로 `TypesetEngine::format_table` 은 빈 host float 표에서 `fit_measured_table_to_declared_height`
  의 결과가 축소이면 [#2195] 규칙("빈 앵커는 확대 방향만")으로 통째로 버린다. 마지막 행만
  회수하는 예외(`fit_measured_table_nested_tail_to_declared_height`)는 *모든 셀 `row_span == 1`* +
  *마지막 행이 1×1 중첩 표 소유* 형상 전용인데, 이 표는 rowspan 셀이 4개다.

## 3. 수정

`fit_measured_table_declared_tail_to_declared_height`(`src/renderer/height_measurer.rs`) 를 더했다.
페인트 경로가 이미 하는 "부족분 → 마지막 행" 의 **대칭**이며, 발동 조건을 좁게 잡았다.

1. 행 2개 이상, 표 선언높이 `hp:sz` 가 있을 것
2. **마지막 행이 저장 선언(cellSz)으로만 잡혀 있을 것** — `|measured − cellSz| ≤ 0.5px`.
   콘텐츠가 밀어 키운 행이면 회수할 여유가 없다고 보고 손대지 않는다 (#5714 행 성장 축 보호)
3. 마지막 행에 합성 줄 셀·중첩 표가 없을 것 (후자는 기존 nested-tail helper 담당)
4. 초과분이 `0.5px < reduction ≤ max(선언×2%, 1px)` — 반올림 급은 건드리지 않고, 큰 모순은
   선언이 stale 한 문서일 수 있으므로 종전대로 콘텐츠 기반 분할에 맡긴다 (#672 TAC 임계와 같은 폭)
5. 줄인 뒤에도 **마지막 행 셀의 저장 줄 내용 + 상하 패딩 아래로 내려가지 않을 것** — 글자를
   자르지 않는다 (#5879 가 세운 하한 규칙과 같은 축)

호출자는 두 곳이며 둘 다 native HWP5 · 비-TAC · TopAndBottom · RowBreak · 다행 계약으로 게이트한다.

- `TypesetEngine::format_table` (`src/renderer/typeset.rs`) — 쪽 경계 판정
- `LayoutEngine::resolve_row_heights_with_common_fit` (`src/renderer/layout/table_layout.rs`) — 페인트 기하

조판만 고치면 페인트가 원래 높이를 그려 표가 본문 밖으로 넘친다(76076 p81→82 의 교훈). 두 경로가
같은 행 기하를 소비해야 한다.

## 4. 결과

```
$ rhwp dump-pages samples/float-stack-defer.hwp
문서 로드: samples/float-stack-defer.hwp (2페이지)
=== 페이지 2 ===  Table pi=1 ci=1 12x3 596.6x907.3px wrap=TopAndBottom
```

2쪽 괘선 13개 경계 전부가 정본과 **≤1.8px** 로 일치한다.

| 경계 | 수정 후(pt) | 정본(pt) | 차 |
|---|---|---|---|
| 표 상단 | 89.55 | 90.86 | −1.31 |
| 행10 하단 | 717.15 | 717.78 | −0.63 |
| 표 하단 | 770.03 | 770.64 | −0.61 |

전/후/정본 3단 비교: [`edit_demo_5906/float_stack_defer_p2.png`](edit_demo_5906/float_stack_defer_p2.png)

## 5. 검증

| 게이트 | 결과 |
|---|---|
| 259문서 쪽수 게이트 (`tools/render_page_gate.py`, 전/후 TSV 대조) | 변경 **1건** — 대상 문서 `3→2` (delta `+1→0`). 회귀 **0** |
| 코퍼스 SVG self-diff (259문서 × 앞 2쪽 = 518 렌더, 전/후 SHA-256) | 변경 파일 **1개** — `float-stack-defer_002.svg`. 의도 외 변화 **0** |
| `cargo test --profile release-test --lib -p rhwp` | 3,893 passed / 0 failed |
| `regression_suite_003` (새 테스트 소속) | 105 passed / 0 failed / 1 ignored |
| `rustfmt --edition 2021 --check` (변경 3파일 + 새 테스트) | clean |
| `cargo clippy --all-targets -- -D warnings` | exit 0 |
| `node scripts/rust-unit-test-tiers.mjs --check --base-ref origin/devel` | 4,225 유지 (테스트는 `tests/cases/`) |
| `rhwp layout-anomaly samples/float-stack-defer.hwp` | 전·후 모두 `status: CLEAN` (overflow 0) |

새 테스트 `tests/cases/issue_5906_float_stack_declared_tail.rs` 는 red→green 을 실측했다.

```
수정 전: assertion `left == right` failed: 한글 2022 정본은 2쪽이다 … left: 3  right: 2
수정 후: test … ok
```

## 6. 남긴 것 — 정직하게

수정 후 2쪽에서 `LAYOUT_OVERFLOW` **진단 한 줄**이 새로 찍힌다.

```
LAYOUT_OVERFLOW: page=1, ..., type=Table, first=true, y=1055.4, bottom=1031.8, overflow=23.6px
```

표 자체는 1,026.7px 에서 끝나 본문 안이다. 초과분 23.6px 은 표 **뒤에 붙는 빈 host 앵커 줄**
(28.7px)의 흐름 전진분이고 잉크가 없다. 이 문서의 host 는 저장 LineSeg 를 하나만 가지며
(`vpos=58203` → 1쪽 표1 바로 아래), 2쪽에는 앵커 줄이 없어야 한다. rhwp 는 쪽마다 앞뒤로
한 번씩 세는 셈이다.

- 종전에는 같은 전진분이 3쪽(여유 있는 쪽)에 놓여 드러나지 않았다. 이 PR 이 만든 계산이 아니라
  **드러낸** 계산이다.
- `layout-anomaly` 는 이를 `first_in_column` 항목으로 분류해 집계하지 않는다 — 전·후 모두
  `overflow: 0 … status: CLEAN`.
- 잉크가 없어 래스터·정본 대조에는 영향이 없다.

이 앵커 줄 이중 계수는 `is_deferred_blank_para_float_stack_anchor` 계열(연속 float 2개 이상만
지연)의 사각지대이며, 범위를 넓히면 자리차지 표 전반의 흐름 경계가 움직인다. 이 PR 범위에서
분리해 후속 축으로 남긴다.
