# task_m100_4333 Stage 2 — 높이 정의 사슬(#4620·#4619·#4604·#4606·#4333) 측정과 판정

- **이슈**: [#4333](https://github.com/edwardkim/rhwp/issues/4333) (우산),
  [#4620](https://github.com/edwardkim/rhwp/issues/4620),
  [#4619](https://github.com/edwardkim/rhwp/issues/4619),
  [#4604](https://github.com/edwardkim/rhwp/issues/4604),
  [#4606](https://github.com/edwardkim/rhwp/issues/4606)
- **분기 기준**: `upstream/devel` `96e52a01b` (게이트 실행 시점 `298c2c1b2` → 리베이스;
  그 사이 5개 커밋은 CI workflow·문서·python 스크립트로 **Rust 소스 변경 0**)
- **선행**: Stage 1 은 PR [#4607](https://github.com/edwardkim/rhwp/pull/4607) (미병합)
- **상태**: 코드 변경은 #4333 의 주석 오기 정정 1건. 나머지 넷은 측정으로 판정했고
  #4620 은 닫을 근거, #4619·#4604·#4606 은 보류 근거를 남긴다.
- **기록일**: 2026-08-12 KST

## 0. 요약

| 이슈 | 판정 | 근거 |
| --- | --- | --- |
| #4620 | **닫는다** — 소비자가 이미 다른 값을 쓴다 | 4절 |
| #4619 | **보류** — 코퍼스 재현 0건 + 순진한 이식은 확정 회귀 | 5절 |
| #4604 | **통일하지 않는다** — 문단 축의 두 정의는 살아 있는 소비자가 하나뿐 | 6절 |
| #4606 | **기저선 5.6% 의 97.1% 는 계측 기준의 산물** — 실제 기저선은 0.2% | 7절 |
| #4333 | 주석 오기만 정정, 누적 정의는 보류 | 3절·8절 |

## 1. 계측 기준 — AC 축이 어디서 무의미해지는가

이 사슬에서 두 번 반복된 실패 모드는 **문단 진행이 아닌 하위 단계를 재는 것**이다
(#4333 두 번째 코멘트). Stage 2 의 표는 세 값을 같은 입력으로 비교한다.

| 기호 | 정의 | 위치 |
| --- | --- | --- |
| **A** | `HeightMeasurer::measure_paragraph` 의 `total_height` | `renderer/height_measurer.rs:619` |
| **B** | `TypesetEngine::format_paragraph` 의 `total_height` | `renderer/typeset.rs:14318` |
| **C** | scratch `LayoutEngine::layout_partial_paragraph` 의 고립 advance | `renderer/layout/paragraph_layout.rs:2130` |

**C 는 모든 축에서 렌더의 문단 진행이 아니다.** 렌더의 문단 진행은
`layout_column_item`(`renderer/layout.rs:6720`)이 정하고, 다음 경우에 C 를 넘어선다.

- `Control::Shape` + `treat_as_char` — TAC-Shape 높이 바닥값(`layout.rs:7037-7076`)이 진행을 지배한다.
- `treat_as_char == false` 인 Picture/Shape — 개체는 별도 `PageItem::Shape` 로 예산을 먹는다.
- 표 — 별도 `PageItem::Table`.

따라서 **C 가 뜻을 갖는 축은 PlainText·Equation·Picture(tac=true) 뿐이다.** 나머지 축의
AC 발산은 계측 산물이지 결함이 아니다. 이 구분 없이 "측정을 렌더에 맞춘다"를 실행하면
#4333 이 이미 두 번 밟은 함정을 세 번째로 밟는다.

또 하나: `layout_partial_paragraph` 는 `is_column_top = (y - col_area.y).abs() < 1.0`
(`paragraph_layout.rs:2778`)일 때 `spacing_before` 를 한컴 정합 규칙으로 잘라낸다
(`:2771-2809`). scratch 를 `y_start = col_area.y` 로 돌리면 **모든 문단이 단 최상단으로
계측**된다. 아래 표는 두 기준을 모두 싣는다.

## 2. 축별 발산표 (352개 문서, 170,870 문단)

`samples/*.hwp|*.hwpx` 355개 중 352개 파싱 성공(암호 3건 제외). 문단은 컨트롤 우선순위로
배타 분류했다(Table > Picture tac=true > Picture tac=false > Shape tac=true >
Shape tac=false > Equation > 기타 컨트롤 > PlainText=컨트롤 없음). 발산 임계 1.0px.

| 축 | n | AB 발산 | % | AB 최대 | AC 발산(단 최상단) | % | AC 발산(단 중간) | % | AC 최대(단 중간) | AC 유효? |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | :--: |
| PlainText | 158,161 | 509 | 0.3% | 20.00 | 8,545 | 5.4% | **252** | **0.2%** | 40.00 | 예 |
| Equation | 4,198 | **0** | 0.0% | 0.00 | 14 | 0.3% | **0** | **0.0%** | 0.00 | 예 |
| Picture tac=true | 525 | 4 | 0.8% | 280.29 | 18 | 3.4% | **2** | **0.4%** | 3.11 | 예 |
| Shape tac=true | 822 | **0** | 0.0% | 0.00 | 402 | 48.9% | 367 | 44.6% | 739.32 | 아니오 (바닥값) |
| Picture tac=false | 402 | 42 | 10.4% | 238.93 | 73 | 18.2% | 73 | 18.2% | 655.84 | 아니오 (별도 item) |
| Shape tac=false | 96 | 36 | 37.5% | 25.87 | 19 | 19.8% | 2 | 2.1% | 803.99 | 아니오 (별도 item) |
| Table | 5,018 | 35 | 0.7% | 227.81 | 1,634 | 32.6% | 306 | 6.1% | 645.39 | 아니오 (별도 item) |
| 기타 컨트롤 | 1,648 | 6 | 0.4% | 33.60 | 547 | 33.2% | 24 | 1.5% | 33.60 | 혼재 |

AB 는 두 값이 모두 `(para, composed, styles, col_w)` 의 순수 함수라 기준 y 에 불변이다.

## 3. #4333 — 접힘과 바닥값의 짝, 그리고 주석 오기

`paragraph_layout.rs:3233` 의 옛 주석이 짝의 나머지 반쪽을 "reserved/skip-advance 보상
기계"로 지목했다. **실제 보상자는 `layout_column_item` 의 TAC-Shape 높이 바닥값**
(`layout.rs:7037-7076`)이다. 이번 실행에서 직접 재확인했다.

`samples/hwp3-sample16-hwp5.hwp` 구역0 문단71:

```
$ RHWP_DEBUG_PARA_TAC=all RHWP_DEBUG_TAC_CURSOR=1 rhwp export-svg …
  TAC_ADV    pi=71 line_idx=0 y=163.8 raw_lh=130.2 lh=0.0 ls=10.4 whitespace=true
  TAC_CURSOR FullPara pi=71 y_in=163.8 y_out=294.0 dy=130.2 was_tac=false
```

줄 루프의 진행은 `lh + ls = 10.4` 인데 문단 진행은 `130.2` 다 — 바닥값
`para_start + max(seg_lh, shape_max_h) = 163.8 + 130.2 = 294.0` 이 물었다. 접힘을
없애면 루프 진행이 `raw_lh + ls = 140.6` 이 되어 바닥값을 넘고, 문단은 정확히
`LineSeg.line_spacing`(10.4px) 만큼 밀린다 — `tests/issue_1116.rs` 의 한컴 PDF 핀
둘이 깨지는 값이다(#4333 두 번째 코멘트 실측).

`HeightCursor::vpos_adjust` 는 보상자가 **아니다.** 같은 문서에서:

```
$ RHWP_VPOS_DEBUG=1 rhwp export-svg …
  VPOS_CORR_SKIP: pi=72 prev_pi=71 y_in=293.97 lazy_base_corrected=-852 lazy_base=-72
```

`lazy_base < 0` 이라 `height_cursor.rs:294` 에서 조기 반환한다. 오기가 앞선 조사를
이쪽으로 보냈다. **이번 커밋은 이 주석만 사실로 바꾼다** — 동작 변경 없음.

남은 과제(누적 정의)는 8절.

## 4. #4620 — 닫는다. `TextLine` 높이 0.0 을 읽는 소비자가 없다

### 재현은 된다

`export-render-tree` 로 `2025 행정업무운영 편람(최종).hwp` 를 뜨면 이슈가 인용한 노드가
그대로 나온다(쪽 색인 163).

```json
{"type":"TextLine","bbox":{"x":113.4,"y":886.7,"w":529.1,"h":0.0},"pi":67,
 "children":[{"type":"TextRun","bbox":{"x":637.8,"y":147.4,"w":0.0,"h":0.0},"text":"","pi":67}]}
```

`h` 는 `RenderNode.bbox.height` 의 직렬화이고(`render_tree.rs:246`), 그 값은
`TextLineNode.line_height` 와 같은 변수에서 온다(`paragraph_layout.rs:3517`·`:3538`).

### 그런데 소비자 셋 다 다른 값을 쓴다

- **캐럿** — 인라인 개체 자리의 캐럿은 `TextLine` 이 아니라 **개체 자신의 렌더 bbox** 에서
  나온다(`document_core/queries/cursor_rect.rs:660-696`, `control_bboxes`).
  `TextLine` 을 읽는 경로(`find_para_line`, `:1210-1236`)는 TextRun 이 하나도 안 잡힌
  빈 문단 폴백이다.
- **히트테스트** — `hit_test_native`(`cursor_rect.rs:1508`)의 `RunInfo.bbox_h` 는 전적으로
  `TextRun`/`TableCell`/`TextBox` 에서 채워진다. `TextLine` 을 읽지 않는다.
- **줄 선택** — `cursor_nav.rs:2070` 이 `TextLine` 의 `bbox.height` 를 `CursorHit.h` 에
  담지만, 선택 사각형의 y·h 는 **항상 left_hit** 에서 온다(`cursor_nav.rs:2310`,
  주석이 그렇게 적어 뒀다). `TextLine` 쪽 값은 우측 끝 x 폴백으로만 쓰이고 버려진다.

### 실측 — 접힘 대상 681건, 0 높이 캐럿 0건

텍스트가 없고 `treat_as_char` 그림/도형만 있는 문단 전수(코퍼스 1,120건)에 대해
`get_cursor_rect_native(sec, pi, 0)` 을 돌렸다. 접힘 게이트가 요구하는 `Control::Shape`
보유 문단은 **681건**이다.

```
TAC-only paragraphs: 1120, zero-height caret: 0, nonzero: 1120
  of which TAC-Shape (접힘 게이트 대상): 681, min caret height: 12
```

**0 높이 캐럿은 한 건도 없다.** 이슈 본문의 "확인이 먼저다" 절이 지정한 종료 조건
("관측되지 않으면 소비자들이 이미 다른 값을 쓰고 있다는 뜻이므로 그 사실을 기록하고
닫는다")에 해당한다.

### 게다가 이슈가 말한 "안전한 수정"은 안전하지 않다

이슈는 "노드의 `h` 만 채우고 진행량 계산은 그대로 두면 안전하다"고 적었다. 그렇지 않다 —
`bbox.height` 는 진행량으로 되먹임된다.

- `paragraph_layout.rs:3631` — 미주 문단의 렌더 진행:
  `line_flow_height.max(max_fs).max(line_node.bbox.height)`
- `layout/picture_footnote.rs:1279`·`:1305` — 각주 위첨자 y 배치가 줄 bbox 높이를 읽는다

즉 `h` 를 채우면 미주 흐름과 각주 마커가 함께 움직인다. 관측된 결함이 없는 상태에서
기하가 움직이는 변경은 순수 위험이다.

## 5. #4619 — 보류. 재현 0건, 순진한 이식은 확정 회귀

### 재현 탐색

`layout_column_item` 의 `PartialParagraph` arm 에 임시 계측을 넣고 코퍼스 355개 문서를
**레이아웃 경로로**(`export-svg`) 전수 렌더했다. (`dump-pages` 는 레이아웃을 돌리지
않으므로 이 탐색에 쓸 수 없다 — 첫 시도가 그래서 0건이었다.)

`treat_as_char` Shape 를 담은 문단이 `PartialParagraph` 로 나오는 경우는 **문서 2개,
조각 5개**뿐이고, **전부 텍스트가 있는 문단**이다(`text_empty=false`).

```
21_언어_기출_편집가능본.hwp sec0 pi=181 start=0 end=8   y_in=1240.64 dy=197.93 seg_lh=18.89 would_floor=+18.89 frag_runs=[1,1,1,1,1,2,2,2]
21_언어_기출_편집가능본.hwp sec0 pi=181 start=8 end=13  y_in=209.76  dy=121.07 seg_lh=18.89 would_floor=+18.89 frag_runs=[1,5,2,1,1]
21_언어_기출_편집가능본.hwp sec0 pi=238 start=0 end=5   y_in=1313.28 dy=125.29 seg_lh=18.89 would_floor=+18.89 frag_runs=[2,1,1,1,1]
21_언어_기출_편집가능본.hwp sec0 pi=238 start=5 end=26  y_in=209.76  dy=508.48 seg_lh=18.89 would_floor=+18.89 frag_runs=[1,2,3,1,3,4,1,2,3,…]
HWP5-nopassword-123456.hwp  sec0 pi=258 start=0 end=1  y_in=978.77  dy=25.12  seg_lh=17.12 would_floor=+17.12 frag_runs=[3]
HWP5-nopassword-123456.hwp  sec0 pi=258 start=1 end=3  y_in=132.27  dy=46.45  seg_lh=17.12 would_floor=+17.12 frag_runs=[3,3]
```

모든 조각에서 **실제 진행(dy)이 바닥값보다 크다.** 즉 `FullParagraph` 의 바닥값을 그대로
옮겨도 다섯 조각 전부 무동작이다. 이슈가 든 전제(도형만 있는 문단이 쪽 경계에 쪼개짐)는
코퍼스에 없다 — 모든 조각의 모든 줄에 run 이 있어 `max_fs > 0` 이고 접힘이 발동하지 않는다.

### 순진한 이식은 확정 회귀다

`FullParagraph` 의 바닥값은 시작점을 이렇게 잡는다.

```rust
let para_start = *para_start_y.get(para_index).unwrap_or(&y_offset);   // layout.rs:7065
```

`layout_column_item` 안에서 `para_start_y` 에 문단을 등록하는 곳은 `FullParagraph`
arm 셋(`layout.rs:6773`·`:6797`·`:6848`·`:6903`·`:6951`·`:7021`)뿐이고, 그 밖의 등록은
`layout_table_item`(`:8386`)·`layout_shape_item`(`:9285`) 안에 있다.
**`PartialParagraph` arm 은 등록하지 않는다.** 같은 코드를 PP 에 복사하면 `unwrap_or` 가
**레이아웃이 끝난 뒤의** `y_offset` 을 집어, `shape_bottom = y_offset + effective_h` 가
항상 `y_offset` 보다 커진다 — 즉 모든 PP 조각이 매번 `effective_h + spacing_after` 만큼
밀린다. 옳은 앵커는 `pp_y_in`(`layout.rs:7167-7185`)이다.

### 옳은 이식은 #4333 의 남은 과제와 같은 것을 요구한다

바닥값은 `seg_lh = max(모든 line_segs)` 를 쓴다. 조각에 대해 이 값을 쓰면 **도형이 다른
조각에 있을 때도** 그 높이를 주장한다 — 위 pi=181 은 도형이 줄 2(조각 0..8)에 있는데
조각 8..13 도 같은 `seg_lh` 를 본다. 조각별로 옳으려면 "이 조각이 그 도형 줄을 소유하는가"
와 줄별 높이가 필요하고, 그것이 바로 8절의 누적 정의다.

**결론**: 재현이 없고, 순진한 수정은 회귀이며, 옳은 수정은 #4333 의 선행 작업을 요구한다.
#4619 는 독립적으로 고칠 수 있는 결함이 아니라 #4333 의 한 면이다.

## 6. #4604 — 통일하지 않는다. 문단 축에는 살아 있는 소비자가 하나뿐이다

### 축별 판정

- **Equation (0/4,198)·Shape tac=true (0/822)** — 이미 소수점까지 같다. 손댈 것이 없다.
- **PlainText (509/158,161, 0.3%, 최대 20.00)** — 관측 가능한 규모가 아니다.
- **Picture tac=true/false, Table** — 큰 행들의 기제가 하나다. **조판은 별도
  `PageItem` 으로 예산을 먹는 줄과 중복 예약된 인라인 개체 줄을 0 으로 두고, 측정은
  그대로 남긴다.** 줄별 값을 나란히 찍으면 바로 보인다.

  ```
  Picture tac=true  중간진도보고서 pi463   A lh=[266.96, 266.96]  B lh=[266.96, 0.0]
  Picture tac=true  투명도0-50.hwp pi0     A lh=[103.11, 103.11]  B lh=[103.11, 0.0]
  Picture tac=false pic2.hwp pi0           A lh=[37.33 ×8]        B lh=[0, 37.33, 0, 37.33, …]
  Picture tac=false hwp3-sample16 pi0      A lh=[26.67] ls=[16.0] B lh=[0.0] ls=[0.0]
  Table             중간진도보고서 pi428   A lh=[214.48, 13.33]   B lh=[0.0, 13.33]
  ```

  조판 쪽 0 은 `format_paragraph` 의 `prev_line_reserved_tac_picture_height`
  (`typeset.rs:14494`)와 블록 개체 제외 규칙이다. 같은 개체를 문단 높이와 별도
  `PageItem` 에 **두 번** 넣으면 페이지네이션이 깨지므로 조판 쪽이 구성상 맞다.

  반대 방향도 한 건 있다 — `2022년 국립국어원 업무계획.hwp` pi586 은 줄별 값이
  양쪽 동일(`lh=[1.33, 46.4] ls=[0.8, 10.4]`)인데 `A=0.00 B=58.93` 이다. 측정 쪽
  `has_table && line_segs.len() >= 2` 의 vpos 클램프(`height_measurer.rs:888-903`,
  `vpos_h.min(sum)`)가 문단 전체를 0 으로 접는다. 이쪽은 측정이 과잉 삭제한다.

  이 클램프와 `clickhere_adjustment`(`:908-940`) 때문에
  **`MeasuredParagraph.total_height` 는 자기 자신의 `sb + Σlh + Σls + sa` 와 다르다.**
  `dump_page_items_json`(`rendering.rs:5262-5271`)은 `"total"` 을 그 합으로 다시
  만들므로, 같은 문단에 대해 진단이 내는 값과 `total_height` 가 어긋난다.

### 왜 "통일"이 답이 아닌가

`MeasuredSection.paragraphs` 는 **프로덕션 페이지네이션이 읽지 않는다.**
`DocumentCore::paginate_pass` 가 `TypesetEngine::typeset_section_with_variant` 에 넘기는
것은 `measured.tables` 뿐이다(`document_core/queries/rendering.rs:4300`).
`Paginator::paginate_with_measured_opts`(= `.paragraphs` 의 소비자)는
`RHWP_USE_PAGINATOR=1` 폴백과 구역 0개 문서에서만 돈다(`:4212`).

즉 문단 높이 축에서 두 정의는 **살아 있는 소비자를 각각 갖고 있지 않다.** 한쪽은
프로덕션이고 다른 쪽은 폴백·진단이다. 여기서 "공유 함수 추출"을 하면 둘 중 하나다.

1. 프로덕션(`format_paragraph`)을 폴백에 맞춘다 → 한컴 핀이 걸린 경로를 진단값 쪽으로
   움직인다. 명백히 잘못된 방향이다.
2. 폴백을 프로덕션에 맞춘다 → 그것은 통일이 아니라 **두 번째 정의를 지우는 일**이고,
   이미 #4605/PR #4621 이 그 방향으로 가고 있다.

따라서 #4604 의 옳은 후속은 "정의를 합친다"가 아니라 "`fallback_paragraphs` 소비자를
`format_paragraph` 로 옮기거나 걷어낸다"이며, 그 전에는 축별 표를 기저선으로 남긴다.

## 7. #4606 — 기저선 5.6% 의 97.1% 는 계측 기준의 산물이다

이슈의 `288/5128 (5.6%)` 를 전수(158,161 PlainText 문단)로 다시 재면 **8,545건(5.4%)**
으로 비율이 재현된다. 잔차의 모양을 분해하니 한 가지가 지배한다.

| 잔차 `r = A − C` | 건수 | 비율 |
| --- | ---: | ---: |
| `spacing_before` | 8,293 | **97.1%** |
| `ls_last` | 3 | 0.0% |
| `sb + ls_last` | 1 | 0.0% |
| 그 외 | 248 | 2.9% |

원인은 결함이 아니라 **한컴 정합 규칙**이다. `layout_partial_paragraph` 는
`is_column_top`(`paragraph_layout.rs:2778`)일 때 `spacing_before` 를 잘라내거나 저장
`LineSeg.vertical_pos` 로 클램프한다(`:2771-2809`, Task #853/#1811 의 한컴 PDF 근거가
주석에 있다). scratch 계측은 `y_start = col_area.y` 로 돌아 **모든 문단이 단 최상단**으로
잡혔다.

`y_start` 를 단 중간으로 옮긴 대조 실행:

| 축 | AC 발산(단 최상단) | AC 발산(단 중간) |
| --- | ---: | ---: |
| PlainText | 8,545 (5.4%), 최대 75.57 | **252 (0.2%), 최대 40.00** |
| Equation | 14 (0.3%) | **0 (0.0%)** |
| Picture tac=true | 18 (3.4%) | **2 (0.4%), 최대 3.11** |

#4606 의 원 표본(Stage 1) 자체가 "`PaginationResult` 기준 **단 최상단 문단**"이었다.
즉 그 5.6% 는 그 모집단에 대해 **참이지만 설계대로**다 — 단 최상단이라서 렌더가 트림한
것이고, 트림은 한컴 PDF 근거로 들어간 규칙이다.

**PlainText 기저선의 실체는 5.6% 가 아니라 0.2% 다.** 그리고 없어진 97% 는
"측정이 틀렸다"가 아니라 "측정은 문단이 어느 단의 어느 위치에 앉을지 모른다" 이다 —
그건 페이지네이션의 **출력**이므로 측정 입력이 될 수 없다. 두 정의가 남아 있는 구조적
이유가 이것이고, 이 축에서는 렌더가 한컴 쪽이다.

남은 0.2%(252건)는 `sb`·`ls` 로 설명되지 않는다. 최대 표본:

```
hwp3-sample4-hwp5.hwp sec0 pi29  r=40.00  A=60.00 C=20.00
  sb=0.00 sa=0.00 ls_last=6.67  nlines_measured=3 nlines_comp=3 nsegs=3
```

줄 수는 3으로 같은데 진행이 20px 대 60px 다 — 줄별 진행(공백 줄 skip_advance 계열)의
차이로 보이며, 이것이 #4606 이 기록할 만한 진짜 기저선이다. 관측(쪽수) 영향은 없다.

## 8. #4333 에 남는 것

바닥값을 양방향 스냅으로 바꾸면 핀도 지키고 지표도 닫히지만, `seg_lh` 가
`line_segs` 에 대한 `max` 이지 `sum` 이 아니라 여러 줄에 걸친 TAC 도형이 잘린다.
조판 쪽에는 그 부류를 위한 누적 정의가 있다 —
`stacked_tac_picture_heights`(`typeset.rs:14469-14492`), 게이트는
`para_is_treat_as_char_picture_only && tacs.len() >= 2 && lines.len() == tacs.len() &&
모든 char_start 동일 && 모든 높이 > 8px` 다. **렌더 쪽 대응물을 먼저 만들어야 하고,
5절이 보인 대로 #4619 도 같은 것을 요구한다.** 그 전에는 접힘도 바닥값도 건드리지 않는다.

이번 실행에서 그 대응물을 만들지 않았다. 가장 많이 pin 이 걸린 경로에 좁은 특례를 넣는
대신, 판단에 필요한 사실(1절의 AC 유효 축, 5절의 앵커 위험, 8절의 게이트)을 남긴다.

## 9. 발견했지만 손대지 않은 것

이 다섯 이슈의 범위 밖이라 고치지 않았다. 각각 별도 이슈 후보다.

1. **TAC-Shape 바닥값만 도형 흐름 높이의 정의가 다르다** —
   `layout_column_item` 의 `shape_max_h`(`renderer/layout.rs:7058`)는
   `s.common().height` 만 쓰는데, 같은 속성을 렌더의 다른 자리들
   (`paragraph_layout.rs:696`, `composer/line_breaking.rs:1087`, `layout.rs:10641`)은
   `max(common.height, shape_attr.current_height)` 로 쓴다. PR #4607 이 후자를
   `ShapeObject::flow_height_hu()` 하나로 모으지만 **`layout.rs:7058` 은 건드리지
   않는다.** #4607 병합 후에도 정의가 둘로 남고, 남는 쪽이 하필 문단 진행을 지배하는
   자리다.
2. **`para_is_treat_as_char_picture_only` 두 정의가 정반대 답을 낸다** —
   `renderer/height_cursor.rs:45` 는 `para.text.trim().is_empty()`,
   `renderer/typeset.rs:1370` 은 `!para_has_visible_text(para)`(`typeset.rs:1192`,
   `\u{FFFC}` 를 제외)다. 표준적인 도형-전용 문단(`text == "\u{FFFC}"`)에서 전자는
   `false`, 후자는 `true` 다. 이 술어가 8절의 `stacked_tac_picture_heights` 를 연다.
3. **같은 이름의 함수 둘이 여전히 다른 식이다** —
   `tac_picture_or_shape_height_px` 가 `paragraph_layout.rs:691`(max)과
   `typeset.rs:2002`(common.height)에 각각 있다. PR #4607 이 정리 대상으로 삼았다.
4. **측정 표 바닥값이 조판·페이지네이터에 복제돼 있다** —
   `typeset.rs:16865-16871` 과 `pagination/engine.rs:918-922` 가 같은
   `effective_h = seg_lh.max(mt_h)` 를 각자 계산한다.
5. **`MeasuredParagraph.total_height` 가 자기 줄 벡터의 합과 다르다** —
   `height_measurer.rs:888-903`(`has_table` vpos 클램프)과 `:909-940`
   (`clickhere_adjustment`) 때문이다. `dump_page_items_json`
   (`rendering.rs:5262-5271`)은 `"total"` 을 `sb + Σlh + Σls + sa` 로 다시 만들어
   같은 문단에 대해 다른 값을 낸다.
6. **`dump-pages` 가 프로덕션이 안 읽는 높이를 보고한다** —
   `rendering.rs:5257-5258` 이 `measured.get_measured_paragraph(...)` 로 `height` 를
   만든다. 이 계열에서 수정 위치를 두 번 오도한 값이 그대로 진단 표면에 노출돼 있다.
7. **무동작 삼항** — `document_core/queries/cursor_rect.rs:4409`,
   `if child.bbox.height > 0.0 { 12.0 } else { 12.0 }`. 두 가지가 같아 조건이
   아무 일도 하지 않는다.
8. **인라인 개체 옆 캐럿 높이가 개체 크기와 무관한 상수 12.0 이다** —
   `cursor_rect.rs:676`(`let fallback_h = 12.0;`). 739px 도형 줄에서도 12px 다.
   #4620 이 주장한 0 은 아니지만 한컴과는 어긋난다.

## 10. 검증

| 게이트 | 결과 |
| --- | --- |
| `cargo fmt --all -- --check` | exit 0 |
| `cargo clippy --all-targets -- -D warnings` | exit 0 |
| `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` | exit 0 — `ok` 블록 **536**, **5,766 passed / 0 failed** |
| `cargo test --profile release-test --test issue_1116` | exit 0 — **13 passed / 0 failed** |
| `--features native-skia skia --lib` | exit 0 — 58 passed |
| `--features native-skia --test issue_2225_missing_picture_placeholder` | exit 0 — 2 passed |
| `--features native-skia --test render_p37_direct_pdf_export` | exit 0 — 4 passed |
| `wasm-pack build --target web --out-dir pkg` | exit 0 |

리베이스 뒤 `96e52a01b` 기준으로 `fmt`·`clippy`·`issue_1116`(13 passed)을 다시 확인했다.
전체 회귀는 재실행하지 않았다 — 그 사이 upstream 델타에 Rust 소스 변경이 없다.

`issue_1116` 13건에는 #4333 이 지목한 한컴 PDF 핀 둘이 그대로 들어 있다 —
`sample16_hwp5_page3_heading_positions_follow_lineseg_vpos`,
`sample16_hwp5_2022_page3_bcp_tail_glyph_stays_on_hancom_line`.

Native Skia 통합 시험 둘은 `--features native-skia` 없이 돌리면 각각 1건/0건만 잡힌다
(나머지가 feature 게이트 뒤에 있다). §4.3 의 명령 그대로 feature 를 붙여 재실행한 값이 위 표다.

**기하 무변경**: 이번 커밋의 코드 변경은 주석 한 블록뿐이다. 기계적 증거:

```
$ git diff upstream/devel..HEAD -- '*.rs' | grep -E "^[+-]" \
    | grep -vE "^(\+\+\+|---)" | grep -vE "^[+-]\s*//" | wc -l
0
```

주석이 아닌 Rust 줄이 0 이다. 따라서 쪽수 스윕·시각 대조는 대상이 아니다.

**RED 증명**: 주석 정정에는 걸 수 있는 테스트가 없다 — 지어내지 않았다. 이 주석이
문서화하는 불변식(접힘 없이는 바닥값이 지배하지 않는다)의 실질적 가드는 이미
`tests/issue_1116.rs` 의 한컴 핀 둘이고, 정정한 주석이 이제 그 가드를 정확히 지목한다.

## 11. 계측 재현 방법

이번 측정에 쓴 스크래치는 **커밋하지 않았다**(단정 없는 테스트를 저장소에 남기지 않는다).
재현 절차만 남긴다.

1. **축별 발산표(2절)** — `renderer::typeset::tests` 안에 `#[ignore]` 테스트를 두고
   `samples/*` 전수에 대해 `DocumentCore::from_bytes` → 구역별
   `(section.paragraphs, doc.composed[idx], doc.styles)` 와
   `PageLayoutInfo::from_page_def(page_def, find_initial_column_def(paras), dpi)` 의
   첫 단 폭으로 A/B/C 를 계산한다. C 는 `measure_endnote_para_advance`
   (`typeset.rs:14121-14197`)와 같은 방식으로 scratch `LayoutEngine` +
   `LayoutFrame` 을 쓴다. 단 최상단/단 중간은 `y_start` 를 `col_area.y` 로
   두느냐 `+100.0` 으로 두느냐로 가른다.
2. **#4619 재현 탐색(5절)** — `layout_column_item` 의 `PartialParagraph` arm 에
   env 게이트 `eprintln!` 을 넣고 `rhwp export-svg` 로 코퍼스를 전수 렌더한다.
   `dump-pages` 로는 레이아웃이 돌지 않는다.
3. **#4620 캐럿 실측(4절)** — `examples/` 스크래치에서 `parse_document` 로 문단을 훑어
   텍스트 없는 TAC 문단만 골라 `HwpDocument::get_cursor_rect_native(sec, pi, 0)` 의
   `height` 를 집계한다.


## 후속 이슈 (2026-08-12)

작업 중 발견했지만 배정된 다섯 이슈 밖이라 **고치지 않고** 이슈로 분리했다.

- **[#4625](https://github.com/edwardkim/rhwp/issues/4625)** — TAC-Shape 바닥값
  (`layout.rs:7058`)만 `common().height` 를 쓴다. **문단 진행을 실제로 지배하는 쪽**이
  나머지 셋(`max(common.height, current_height)`)과 다른 식이고, PR #4607 의 통일 대상에서
  빠져 있다.
- **[#4626](https://github.com/edwardkim/rhwp/issues/4626)** —
  `para_is_treat_as_char_picture_only` 두 정의가 도형만 있는 문단(`text == "\u{FFFC}"`)에
  **반대 답**을 낸다. 이 술어가 `stacked_tac_picture_heights` 를 게이트하므로 **#4333 의
  남은 작업을 시작하려면 이것이 선행 조건**이다.
- **[#4627](https://github.com/edwardkim/rhwp/issues/4627)** — 표 바닥값
  `effective_h = seg_lh.max(mt_h)` 가 `typeset.rs` 와 `pagination/engine.rs` 두 곳에 있다.
- **[#4628](https://github.com/edwardkim/rhwp/issues/4628)** — 진단 출력이 프로덕션과 다른
  높이를 말한다. `total_height` 가 자기 구성요소 합과 불일치하고, `dump-pages` 가 폴백 경로
  값을 표시 없이 보고한다. **이 값이 이 사슬에서 조사를 두 번 오도했다.**
- **[#4629](https://github.com/edwardkim/rhwp/issues/4629)** — 인라인 개체 캐럿 높이가
  개체 크기와 무관한 상수 12.0. #4620 이 주장한 0 을 검증하다 나온 **진짜** 문제다.

## 배정 이슈 넷을 닫은 근거

- **#4620** — 소비자가 아무도 `TextLine.h` 를 안 읽는다(1,120건 전수, 캐럿 최솟값 12.0,
  0 은 0건). 게다가 채우면 미주 진행·각주 마커 배치가 움직여 **해롭다**.
- **#4619** — TAC-Shape 문단이 `PartialParagraph` 에 닿는 것은 문서 2개/조각 5개뿐이고
  전부 실제 진행이 이미 바닥값을 넘어 이식이 no-op 이다. 게다가 `para_start_y` 미등록으로
  순진한 이식은 확실한 회귀다. **#4333 의 한 단면으로 접었다.**
- **#4604** — 문단 축에 살아 있는 소비자가 둘이 아니다. 통일은 프로덕션을 진단용에 맞추거나
  두 번째 정의를 지우는 것뿐이고, 후자는 #4605(PR #4621)의 방향이다.
- **#4606** — 5.6% 중 97.1% 가 `spacing_before` 이고, 렌더의 단 최상단 trim 은 한컴 정합을
  위한 **설계**다. 단 중간 대조 실험에서 5.4% → **0.2%**.
