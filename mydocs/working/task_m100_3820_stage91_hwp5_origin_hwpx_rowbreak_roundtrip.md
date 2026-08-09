---
kind: investigation
status: active
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-09
---

# Task #3820 Stage 91 — HWP5-origin HWPX RowBreak 왕복 소유권

## 발견 경로

Stage 90의 전체 `cargo test --profile release-test --tests`는 #2755 보정 뒤 library
test `3375 passed; 0 failed; 13 ignored`까지 통과했지만, 이후
`issue_1939_hwp5_origin_hwpx_strict_render_diff_is_stable`에서 실패했다. 대상은 확장자가
`.hwpx`인 HWP5 바이너리 `samples/issue1891/76076_regulatory_analysis.hwpx`다.

`export-hwpx --verify-pages`는 원본과 재파스 모두 82쪽임을 확인했다. 그러나 strict
`render-diff --via hwpx`에서는 33, 49, 50, 80, 81쪽에 구조 차이가 남고 최대 변위는
662.95px다. 쪽수만 같다는 사실은 통과 근거가 아니다.

## 직접 시각 대조

현재 release-test binary로 원본과 `export-hwpx` 산출물의 p34(0-based page 33)를 SVG로
내보내고 raster로 확인했다.

| 입력 | p34 계속 표의 첫 source line |
| --- | --- |
| 원 HWP5 | `- 자율안전확인신고한 분쇄기 등: 약 65,000대 추정` |
| HWP5-origin marker HWPX 재파스 | `현황 추이(p.270)`를 먼저 재도색한 뒤 위 줄을 그림 |

이는 Stage 88에서 원 HWP5에 대해 막은 p33→p34 source cursor 중복이 marker HWPX에서
다시 나타난 것이다. PDF와 HWP5 원본의 page boundary를 정답으로 하며, HWPCTRL 출력·API
fixture를 이 결함의 정답지나 baseline으로 사용하지 않는다.

## 원인

`Document::layout_profile()`은 marker HWPX에 `hwp5_origin_hwpx() == true`를 정확히
부여한다. 이 marker의 명시적 계약도 HWP5의 저장 line-seg와 pagination semantics를
보존하는 것이다.

하지만 Stage 84--88의 좁은 RowBreak 보정은 `native_hwp5_layout()`만 확인한다.
`native_hwp5_layout()`은 원 HWP5 컨테이너에만 true이고, marker HWPX에서는 false다.
그 결과 p34 terminal child의 `force_source_start_cut`가 꺼져 이미 p33이 소유한 source
unit을 p34에서 다시 paint한다. 이 차이는 원본 HWPX에 HWP5 규칙을 넓히는 문제가 아니라,
**rhwp 자신이 부착한 HWP5-origin marker의 기존 계약을 누락한 것**이다.

## 보정 범위와 검증 계획

`LayoutCompatibilityProfile`에 원 HWP5와 marker HWPX만 묶는 명시적 pagination/layout
predicate를 추가하고, Stage 84--88이 새로 도입한 다음 좁은 gate에만 사용한다.

1. declared nested-tail row-height parity,
2. short parent/child fragment 및 terminal source cursor,
3. 해당 child의 render-normalization width projection,
4. empty 1×1 RowBreak host의 flow box.

일반 HWPX의 `hwpx_stored_layout()`은 포함하지 않는다. HWPCTRL 문서
`mydocs/manual/webhwpctrl_compat_development.md`와 `tools/hwpctrl_compat/README.md`가
정한 대로 public control API/COM fixture/ledger는 변경하지 않는다.

수정 후 순서는 `issue_1939` strict gate, `issue_1891` marker page-count gate, Stage 89
focused 편집 gates, 그리고 전체 `cargo test --profile release-test --tests` final summary다.
baseline 또는 PDF 정답은 변경하지 않는다.

## 1차 보정 후 재측정 — 범위가 여전히 넓음

`hwp5_stored_pagination_layout()`을 위의 세 좁은 RowBreak 경로에 적용한 현재
worktree에서, release-test `rhwp render-diff`를 원 fixture에 직접 실행했다. 결과는
82/82쪽, 구조 불일치 0건이지만 `maxDisp=84.32px`로 strict 1px gate에 실패했다.
따라서 page count나 구조 node 수가 같다는 결과를 성공으로 취급하지 않는다.

1px 초과 페이지는 0-based `0, 6, 16, 22, 32, 33, 38, 49, 50, 54, 55, 65, 69, 70,
80`이다. 최대 p23(0-based 22)의 `TextLine20`은 84.32px, p39의 `TextLine19`은
75.87px, p70의 `TextLine31`은 69.39px 이동했다. 모두 node 수와 path는 일치하고
`dy`만 음수이므로, 남은 결함은 source unit 중복이 아니라 재파스 전후의 수직
height/cursor 기준선 불일치다.

이는 662.95px 구조 불일치를 줄인 유효한 중간 결과이나, marker HWPX 전체에
HWP5 RowBreak declared-tail/near-fit 규칙을 적용한 것이 원인 범위를 넓혔을 가능성을
배제하지 못한다. 다음 코드 변경 전에는 각 새 predicate 호출부가 실제로
`RowBreak + TopAndBottom + 마지막 짧은 nested child` 형상까지 다시 확인하는지
검사하고, p23·p39·p70의 공통 table 형상을 비교한다. 일반 HWP5-origin 본문/표의
수직 offset 보정으로 확대하지 않는다.

### 2차 보정 — HWP5-origin 범위를 구조 증거로 재협소화

조사 결과 `fit_measured_table_nested_tail_to_declared_height`는 호출부의
`RowBreak/TopAndBottom` gate 외에는 마지막 행에 **어떤** 1×1 block child가 한 번만
있어도 후보로 수락한다. 반면 source-owner cursor와 width projection의 실제 계약은
마지막 행의 첫 문단이 빈 host이고 nested table control이 정확히 하나이며, 나머지는
line-seg 한 줄 이하의 빈 reset인 경우로 더 좁다. marker HWPX에 새 profile을 켜면서
이 차이가 p23·p39·p70 등의 일반 표까지 tail-height 축소를 허용했을 가능성이 있다.

따라서 `fit_measured_table_nested_tail_to_declared_height` 자체에 동일한
host-ownership 조건을 넣었다. 마지막 행의 첫 문단이 빈 host이고, 단 하나의 비-TAC
1×1 child만 가지며, 후속 문단은 vpos=0 빈 reset뿐인 경우에만 선언 높이를 회수한다.
이 조건은 p81 목적 표를 유지하면서, 단지 마지막 행 안에 1×1 표가 있다는 사실만으로
선언 높이를 회수하지 않게 한다.

또한 같은 HWP5-origin profile을 empty-host RowBreak의 stored line-advance/paint anchor
호출에 일관되게 전달했다. 원본 HWPX의 `hwpx_stored_layout()`은 여전히 false이며,
`hwp5_stored_pagination_layout()` unit test로 marker HWPX와 native HWP5만 수용함을
고정했다.

### 2차 재측정

수정된 release-test binary에서 다음 strict gate가 `1 passed; 0 failed`로 통과했다.

```sh
target/pr-review/release-test/deps/issue_1939-… \
  --exact issue_1939_hwp5_origin_hwpx_strict_render_diff_is_stable --nocapture
```

이는 82쪽 page count, node structure, 최대 변위 1px 조건을 함께 검사한다. 따라서 p23의
24.7px 누적 기준선 차이와 p34의 중복 source line 모두 이 gate에서 해소됐다. 이후
`issue_1891`, Stage 89 focused edit gates, 그리고 최종 전체 release-test는 순차 실행해
이 결과를 전체 회귀 성공으로 승격해야 한다. baseline·fixture·PDF 정답은 변경하지 않았다.
