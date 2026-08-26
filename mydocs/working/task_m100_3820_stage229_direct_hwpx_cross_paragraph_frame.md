# Stage 229 - 직접 HWPX 문단 간 source frame

## 목적

HWPX Q5의 첫 응답 줄이 최신 HWP 2020 MCP PDF/native HWP와 같은 p283이 아니라
p284로 밀리는 #3930 회귀를, 1x1 목차 표의 문단 간 physical frame을 row scanner가
누락하던 문제로 해결한다.

## 원인

- Stage 227의 `direct_hwpx_cell_has_declared_stored_frame`은 cell 전체 LineSeg stream을
  순회하므로 문단 사이 reset도 인식한다.
- 그러나 `row_has_stored_vpos_frame_rewind`은 먼저 각 문단의 `windows(2)`만 검사했다.
  section 10, paragraph 4의 1x1 목차 표는 세 source frame을 문단 22→23,
  47→48 사이 reset으로 기록하므로 raw 검사에서 false가 됐다.
- 그 결과 opening source-frame cut이 선택되지 않아 첫 physical frame의 측정 tail이
  독립 fragment가 되었고, Q5 이후 owner가 p284로 늦어졌다.
- 이 목차 표의 source frame 끝은 `46,468`, `53,456`, `52,072HU`이며 선언 cell
  `159,642HU`를 채운다. 반면 Q5 local cursor의 single reset은 선언 cell 범위를
  넘으므로 Stage 227 predicate에서 여전히 제외된다.

## 수정 계약

- direct original HWPX RowBreak row는 문단 내부 reset이라는 저장 위치에 의존하지 않고,
  `direct_hwpx_cell_has_declared_stored_frame`의 선언-cell 수용성으로 source frame을
  판정한다.
- native/HWP5-origin 경로는 기존의 문단 내부 raw rewind 규칙을 그대로 유지한다.
- 따라서 물리 다중 frame은 opening source-frame cut을 얻고, writer-local single reset은
  physical owner로 승격되지 않는다.
- 표 ID, 문단 번호, 페이지 번호, 텍스트, 폰트, pixel allowance를 사용하지 않는다.

## 검증

scratch에서 다음을 함께 실행했다.

```sh
cargo test --profile release-test \
  --test issue_3930_hwpx_hwp_save_layout \
  --test issue_3820_rowbreak_rowspan_band \
  --test issue_1733 \
  --test issue_2006_1790387_prep_pagination_pin -- --nocapture
```

결과는 #3930 3건, #3820 4건, #1733 2건, issue2006 1건 모두 통과다. #3930 HWPX/HWP는
383쪽, #1733 HWPX/HWP는 HWP 2020 MCP PDF 기준 242쪽, issue2006은 같은 MCP 기준
140쪽이다.

다음 단계에서 main 재검증과 전체 integration test, MCP 2020 PDF 시각 대조를 수행한다.
