---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-08
---

# Task #3820 Stage 72 — issue4090 우측 Square 표의 좌측 본문 wrap 복원

## 출발점

Stage 71의 전체 17쪽 PDF 대조는 `samples/issue4090/156492236_규제샌드박스_min.hwpx`에서
p5, p7, p15, p17의 우측 점선 placeholder 표와 나란히 있어야 할 본문이 rhwp 출력에서
사라지는 결함을 확정했다. Stage 65의 `review_144.png`처럼 보정 전 증적을 최종 결과로
사용하지 않으며, 이 Stage는 issue4090의 별도 `Square` 표 계약만 다룬다.

## PDF와 HWPX의 계약

대표 p5의 anchor는 global `pi=44`의 1×1 비인라인 표다.

- `textWrap="SQUARE"`, `textFlow="BOTH_SIDES"`, `treatAsChar="0"`
- 문단 기준 우측 위치: `horzRelTo="PARA"`, `horzOffset=26319`
- 표 폭 `21022 HU`, 높이 `13241 HU`
- 바로 다음 본문 `pi=45`의 저장 `LINE_SEG`는 처음 6줄에
  `column_start=0`, `segment_width=26319 HU`를 기록하고, 마지막 1줄만 전폭
  `segment_width=48188 HU`로 돌아온다.

즉 PDF의 p5는 표 왼쪽 띠에 6줄을 그리고, 표 아래에서 같은 문단의 마지막 줄을 전폭으로
계속한다. p7/p15/p17도 같은 right-side Square-table + left-strip + full-width tail 형상이다.
이는 단순 reflow 선택이나 font raster 차이가 아니라, HWPX가 명시한 줄 영역 계약이다.

## 현재 경로와 결함 위치

현재 release-test `dump-pages`의 p5는 `pi=45`를 `startLine=6, endLine=7`로 일반
`PartialParagraph`에 남긴다. 따라서 마지막 전폭 tail은 정상적으로 배치된다. 하지만 p5
render tree에는 그 앞 6줄의 `TextLine`이 전혀 없고, placeholder 표만 그려진다.

`typeset_wrap_around_paragraph()`은 이 형상에서 `WrapAroundPara { start_line: 0,
end_line: 6 }`를 기록하고 tail만 `PartialParagraph`로 남기도록 설계되어 있다. 실제 원인은
strip 재조판 자체가 아니라 **소유 컬럼 경계의 손실**이었다. typeset은 `WrapAroundPara`를
`ColumnContent.wrap_around_paras`에 기록하지만 `PaginationResult.wrap_around_paras`는 이전
호환 필드라 빈 `Vec`로 반환한다. 반면 `LayoutEngine::build_single_column()`은 그 전역 빈
목록을 `layout_wrap_around_paras()`에 전달했다. 따라서 p5의 prefix 6줄은 조판에는 기록돼도
paint 단계의 `related` 집합에 들어오지 않았다.

## 보정 범위

1. `build_single_column()`은 해당 column의 `wrap_around_paras`를 우선 전달하고, 기존
   synthetic/legacy caller에서만 전역 목록으로 fallback한다.
2. empty host의 우측 Square 표에서는 typeset과 같은
   `empty_host_square_table_left_strip()`으로 layout strip도 계산한다.
3. 전폭 tail은 현재처럼 일반 `PartialParagraph`로 남긴다. 다른 그림 wrap, 좌측 표 wrap,
   text host 표, 그리고 p6/p8/p16의 explicit tail page에는 적용하지 않는다.
4. 자동 sweep의 overlap/overflow 지표가 이 **wrap exclusion**을 놓친 사실은 별도
   automation 후보로 남긴다. 이번 renderer 보정에 무리하게 결합하지 않는다.

## 회귀 계약과 검증 계획

- 새 focused regression `tests/issue_4090_square_table_left_wrap.rs`: p5 render tree에
  `pi=45` prefix text run이 존재하고 그 우측이 `pi=44` 표의 좌측선 밖을 침범하지 않음을
  확인한다. 마지막 tail text run도 표 아래에 남아야 한다.
- `tests/issue_4090_hwpx_tail_page_break.rs`: 17쪽 및 p5→6, p7→8, p15→16 tail 계약을
  계속 통과했다.
- 직접 PDF sweep(p5/p7/p15/p17, 180 DPI)에서 네 페이지 모두 점선 placeholder 좌측의
  본문이 PDF와 같이 복원된 것을 확인했다. 증적은
  `mydocs/pr/assets/task_m100_3820_stage72_issue4090_square_left_wrap/`에 보존한다.

실행 명령:

```bash
CARGO_TARGET_DIR=target/task-3820-stage65-hwpx-noninline-tac CARGO_INCREMENTAL=0 \
  cargo test --profile release-test --test issue_4090_square_table_left_wrap -- --nocapture
CARGO_TARGET_DIR=target/task-3820-stage65-hwpx-noninline-tac CARGO_INCREMENTAL=0 \
  cargo test --profile release-test --test issue_4090_hwpx_tail_page_break -- --nocapture
python3 scripts/visual_sweep.py --key task3820-stage72-issue4090-square-left-wrap \
  --hwp samples/issue4090/156492236_규제샌드박스_min.hwpx \
  --pdf pdf/issue4090/156492236_규제샌드박스_min-hancom2020-production-verify.pdf \
  --pages 5,7,15,17 \
  --rhwp-bin target/task-3820-stage65-hwpx-noninline-tac/release-test/rhwp --dpi 180
```

전역 재조판은 하지 않았다. 줄 수·page count와 tail 계약은 유지됐다. p1--17 전체 비교의
나머지 페이지 및 자동 sweep의 wrap-exclusion 후보화는 후속 Stage에서 분리한다.
