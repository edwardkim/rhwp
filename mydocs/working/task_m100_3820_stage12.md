---
kind: analysis
status: active
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-05
---

# Task #3820 Stage 12 — p9 그림 블록 수직 흐름 정합

## 목적

`정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp`의 9쪽에서
한컴 2020 기준 PDF와 rhwp가 같은 페이지 수(215쪽)를 유지하면서도 그림·표 묶음과 뒤 본문이
수직으로 다르게 흐르는 결함을 재현하고, 원인을 좁혀 보정한다.

## 기준과 재현 계약

- 입력: `samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp`
- 정답지: `pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf`
- 기준 commit: `0794b174a` (`fix: 중첩 표 페이지네이션과 PUA 표시를 보정한다`)
- 기준 페이지: 1-based 9쪽 (`dump-pages -p 8`의 `global_idx=8`)

전수 `fidelity_compare --text-only --export-all-svg --layout-ledger`는 PDF·SVG·render tree가
모두 215쪽이고 raw PUA/U+FFFD glyph 위험이 0건임을 확인했다. 그러나 이는 그림과 표의 물리
배치를 판정하지 않는다. 54쪽 위험 범위를 대상으로 한 기준 PDF raster 대조에서 p9만
`column_text_flow_collapse`로 승격됐고, 실제 비교 이미지에서도 다음 차이가 보인다.

- rhwp의 `표 1`/`그림 1` 그림 묶음과 뒤의 본문·`그림 2` 묶음이 기준 PDF보다 아래에 배치된다.
- 따라서 p9의 아래쪽 흐름은 같은 페이지에 머물지만 기준보다 늦게 시작한다.
- p94·p129·p157·p160·p161·p164·p178·p214의 자동 후보는 표 fragment·각주·쪽번호 frame의
  보수적 신호이며, 직접 PDF 대조에서 이 Stage의 수정 대상으로 승격하지 않았다.

## source→layout 관찰

`dump-pages -p 8`, `dump --section 0 --para 222` 결과, 문제 그림 묶음은 다음 HWP5 구조다.

- `pi=222`, `ci=0`: 2×2, `RowBreak`, `treat_as_char=false`, `wrap=TopAndBottom` 표.
- 원본 HWPX `Contents/section0.xml`도 이 표에 `treatAsChar="0"`,
  `flowWithText="1"`, `vertRelTo="PARA"`, `vertOffset="5317"`을 기록한다. 따라서
  `flowWithText` 비트 해석의 결함이 아니라 Para-relative offset의 기준이 핵심이다.
- 표 common vertical position은 문단 기준 `5317 HU`(18.8 mm)이며, 첫 행 두 cell이 각각
  `bin_id=27`, `bin_id=28` 그림을 가진다. 둘째 행은 `표 1`, `그림 1` caption이다.
- 다음 `pi=225`, `ci=0`: 2×1, `RowBreak`, `treat_as_char=false`, `wrap=TopAndBottom` 표이며,
  `bin_id=29` 그림과 `그림 2` caption을 가진다.

문제 표의 owner 문단 저장 줄은 `vpos=21800, 23800, 25800 HU`, 각 `line_height=1000 HU`다.
따라서 host 저장 높이는 `25800 - 21800 + 1000 = 5000 HU`이고, `vertical_offset=5317 HU`는
이미 host 마지막 줄의 하단을 지난 위치를 가리킨다.

즉 일반 inline 표나 이미지 단독 위치가 아니라, **문단 기준 세로 offset을 가진 non-inline
TopAndBottom 표 내부 그림·caption 묶음**의 flow reservation/anchor 계약이 조사 대상이다.

## 원인과 보정

기존 `native_multiline_visible_float_table_top` 보정은 본문을 포함한 다중 저장 줄
`TopAndBottom` 표의 양수 offset을 일률적으로 **host 본문 끝 기준**으로 취급했다. 따라서
`para_y + host_height + vertical_offset + outer_margin_top`을 사용했다. 이는 offset이 host
저장 높이보다 짧은 p13 형상에는 필요하지만, p9처럼 offset이 이미 host 끝을 지난 경우 host
높이를 한 번 더 더해 표와 다음 흐름을 한 문단만큼 아래로 밀었다.

보정은 native HWP5·single `TopAndBottom` visible float·문단 기준 양수 offset이라는 기존
제한을 유지한 채, 다음 기하 조건을 추가했다.

- `vertical_offset < host_stored_height`일 때만 기존 host-end anchor를 사용한다.
- `vertical_offset >= host_stored_height`이면 helper가 개입하지 않고 기존 Para anchor 경로가
  offset을 한 번만 적용한다.

focused unit regression은 3줄 host에서 `4000 HU` offset은 기존 host-end 경로를 계속 사용하고,
`5317 HU` offset은 일반 Para anchor로 돌리는 경계를 고정한다. 이로써 같은 shape의 p13 보정을
유지하면서 p9의 이중 가산만 제거한다.

## 완료 기준

- p9의 그림·caption·후속 본문 수직 순서와 시작 위치가 기준 PDF의 같은 흐름으로 복원된다.
- 전체 page count 215, p9 raw glyph 위험 0, 기존 Stage 11의 p166–215 및 issue2007 focused 회귀를
  침범하지 않는다.
- 수정 전후 p9 기준 PDF·rhwp·overlay 증적과 render-tree source 근거를 이 문서에 추가한다.

## 검증 및 증적 (2026-08-05)

모든 cargo 명령은 `CARGO_TARGET_DIR=target/task-3820-3821-fidelity`와
`CARGO_INCREMENTAL=0`으로 실행했다.

- `cargo fmt --check`와 `git diff --check` 통과.
- `cargo test --profile release-test --lib native_multiline_visible_float_uses_host_end_only_for_short_offset`
  통과: `1 passed; 0 failed; 3265 filtered out`.
- 동일 release-test binary로 p9 기준 PDF raster sweep을 수정 전·후 각각 1/1쪽 완료했다.
  두 실행 모두 입력 SVG/render tree 전체 페이지 수는 215이고 p9 raw glyph 위험은 0건이다.

| p9 비교 | pixel match | ink match | 자동 판정 |
| --- | ---: | ---: | --- |
| 수정 전 | 83.08709% | 7.24426% | `column_text_flow_collapse` |
| 수정 후 | 92.25435% | 44.12352% | 없음 |

수정 후 p9에서 `표 1`·`그림 1` 묶음, 이어지는 본문, `그림 2` 묶음이 기준 PDF와 같은 세로
시작 위치로 복귀했다. 지표는 보조 신호이며 최종 판정은 아래 reference/current/overlay 직접
대조로 했다.

- [수정 전 review PNG](../pr/assets/task_m100_3820_stage12_p009_float_offset/review_p009_before.png)
- [수정 후 review PNG](../pr/assets/task_m100_3820_stage12_p009_float_offset/review_p009_after.png)
- [수정 전 overlay PNG](../pr/assets/task_m100_3820_stage12_p009_float_offset/overlay_p009_before.png)
- [수정 후 overlay PNG](../pr/assets/task_m100_3820_stage12_p009_float_offset/overlay_p009_after.png)
- [수정 후 run manifest](../pr/assets/task_m100_3820_stage12_p009_float_offset/run_manifest_after.json)와
  [summary](../pr/assets/task_m100_3820_stage12_p009_float_offset/summary_after.json)

같은 binary로 p9·p13·p23–24를 safety sweep했다(4/4 완료, SVG/render tree 전체 215쪽).
p13의 관련 표는 `Square` wrap이라 이번 helper의 적용 대상이 아니며 직접 비교에서 p9 보정으로
새 이동이 생기지 않았다. p23–24의 그림·caption raster 차이는 이번 조건과 별개의 기존 잔여
결함으로 그대로 관찰된다. 자동 flag가 없다는 사실만으로 이를 해결로 판정하지 않으며, 이 Stage의
수용 범위도 p9으로 한정한다.

## 다음 분석 대상

- p23–24: 그림 21–24의 이미지·caption flow와 raster 차이는 별도 source contract로 분리한다.
- 이 문서는 p9 보정의 분석·검증 기록이며, fixture 전체의 모든 시각 결함이 해결됐다는 의미가 아니다.
