# task_m100_5862 처리결과 — 쪽 분할 조각 셀이 자기 글줄을 잘라 먹던 것을 고친다

- 이슈: [#5862](https://github.com/edwardkim/rhwp/issues/5862)
- 기준: `72674c565` (devel)
- 재현 문서·정답지 모두 저장소 안: `samples/hwpx_sample2.hwp` · `pdf/hwpx_sample2-2024.pdf`
- 시각 증적: [`edit_demo_5862/p8_before_after_oracle.png`](edit_demo_5862/p8_before_after_oracle.png)

## 1. 결함

8쪽에서 마지막 줄이 **가로로 반 잘리고**, 그 아래
`[청약신청주택] 주택명 : 부도임대(경주금장로얄) , 관리번호 : 2026 - 000262` 한 줄은
**한 픽셀도 그려지지 않았다.** 9쪽도 그 줄을 그리지 않으므로 어느 쪽에도 없는 **순수한 소실**이다.

| | y 1000~1030 잉크 픽셀 |
|---|---:|
| 수정 전 | **0** |
| 한글 2024 정본 | 2,946 |
| 수정 후 | **3,824** |

## 2. 원인 — 조각 clip 은 괘선이 아니라 쪽 컷이 정한다

문제의 셀은 `table_partial` 경로가 만든 **쪽 분할 조각**이다(계측: `id=45 y=226.6 h=759.1`).
조각 셀의 clip 높이는 컷 부기(`start_cut`/`end_cut`)가 정하는데, 그 부기와 실제 조판 배치가
어긋나면 조각이 **이미 자기 자식으로 붙인** 글줄이 clip 아래로 내려간다.

```
cell-clip-45 : y=226.6  bot= 985.7      ← 컷이 정한 조각 높이
직계 TextLine:            bot=1018.2      ← 조각이 배치한 마지막 줄
```

렌더 트리에서도 같다 — `Page > Body > Column > Table > Cell > TextLine y=1003.5 h=14.7`.
그 줄은 중첩 표 자손이 아니라 **셀의 직계 자식**이다.

## 3. 수정 — 세 겹으로 좁힌 포섭

`RenderNodeType::TableCell` 에 `page_fragment` 표식을 두고(`table_partial` 의 clip 셀만 true),
`expand_page_fragment_clip_to_own_text_lines` 가 그 셀의 **직계 `TextLine`** 까지만 clip 을 늘린다.

1. **`page_fragment` 셀만** — 일반 셀의 clip 은 괘선 그 자체라, 같은 보정을 넓게 적용하면
   글자가 아래 칸을 침범한다(실측: 게이트 없이 적용하면 7개 문서 135개 clip 이 바뀐다).
2. **직계 글줄만** — 중첩 표 자손까지 허용하면 다음 쪽 조각의 글자를 드러낸다(#5863 이 억제하는 경우).
3. **이미 배치된 것만 보이게 한다** — 콘텐츠를 새로 만들지도, 옮기지도 않는다.

같은 파일의 `extend_clipped_cell_vertical_clip_to_nearby_nested_table_borders`(테두리 stroke 포섭),
`expand_terminal_cell_clip_to_nested_table_descendants` 와 같은 결의 보정이다.

## 4. 검증

### red → green

```
보정 호출만 제거:  split_fragment_keeps_its_last_line_inside_the_clip ... FAILED
                   마지막 글줄(baseline 1016.0)을 담는 셀 clip 이 없다 (#5862)
복원:              3/3 통과
```

나머지 두 계약(`the_next_page_does_not_repeat_the_recovered_line`,
`the_recovered_line_appears_exactly_once_on_its_own_page`)은 **중복 방지 가드**다 —
확장이 다음 쪽 글자를 끌어오거나 같은 줄을 두 번 그리면 실패한다.

### 영향 범위 (전/후 바이너리 직접 대조)

| 검사 | 결과 |
|---|---|
| 259문서 쪽수 게이트 (`tools/render_page_gate.py`) | 245/259 — **전/후 쪽수 변화 0문서** |
| clip 높이 변화 | 21개 문서 표본에서 **5문서 38건** (게이트 없는 판은 7문서 135건이었다) |
| 부수 개선 확인 | `80168_regulatory_analysis.hwp` 17쪽 — 잘려 있던 `적정성 여부` 가 드러난다. 같은 쪽 한글 정본에도 `적정성` 이 있다(정본 대조로 옳음 확인) |

### 실행한 것

| 명령 | 결과 |
|---|---|
| `cargo test --test regression_suite_015 issue_5862` | 3 passed |
| `cargo test --lib -p rhwp` | 통과 (필드 추가로 깨진 유닛 테스트 7곳도 함께 갱신) |
| `python tools/render_page_gate.py …` (수정 전/후) | 245/259 동일, 회귀 0 |
| `cargo clippy --all-targets -- -D warnings` | exit 0 |
| `node scripts/rust-unit-test-tiers.mjs --check --base-ref origin/devel` | 4,225 (src 기준선 유지) |
| rustfmt (변경 파일별; `mod` 선언 파일은 스텁 사본으로 검사) | 차이 없음 |

## 5. 남은 것

같은 쪽의 마지막 줄이 여전히 **셀 괘선에** 살짝 걸려 반 잘리는 부분은 이 보정 범위 밖이다 —
그건 행 높이 측정(`resolve_row_heights` ↔ `height_measurer`)이 조판 내용보다 짧게 잡는 별개 축이고,
이 PR 은 **소실(0px)** 을 **표시(3,824px)** 로 돌리는 데까지만 손댄다.
