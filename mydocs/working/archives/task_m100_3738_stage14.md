---
kind: investigation
status: completed
canonical: mydocs/manual/bug_hunting_playbook.md
last_verified: 2026-08-02
---

# Task #3738 Stage 14 — HWP p58 existing-footnote reset-tail 조기 이월

## 분리 근거

Stage 13 commit `03f20be40`은 p30 각주 29와 p68 그림 49의 소유권만 수정했다. 이후 사용자 visual
inspection에서 p58–p59, p66, p77–p81, p83–p84, p87, p90, p99–p100이 기준 PDF와 다르다고 보고됐다.
동일 release-test native binary로 선택 sweep을 재실행했으며, 다음 세 원인을 분리했다. 이 Stage는 그중
p58의 existing-footnote reset-tail 조기 이월만 수정·검증한다. p77/p83/p66은 Stage 15로 넘겨 독립적으로
다룬다.

## 재현 증거와 가설

### 1. p58 — 각주 safety margin이 stored page-tail 세 줄을 한 줄로 축소

- 문단 646은 HWP5 `LINE_SEG` `vpos=64000/66000/68000` 세 줄 뒤에 `vpos=0` reset을 가진다.
- rhwp p58에는 첫 줄만 있고 p59는 `호주 정부의 …`로 시작한다. PDF p58은 세 줄 모두
  (`캐나다의 …`, `호주 정부의 …`, `Medical Research Council … 치료와`)를 각주 70 위에 둔 뒤,
  PDF p59를 `독립적이며 …`로 시작한다.
- native render tree에서 p58의 첫 줄은 `y=936.5px`, actual `FootnoteArea` top은 `y=1008.0px`다.
  두 후속 stored line의 bottom은 이 물리 경계 앞에 들어온다. 반면 일반 `available_height`는 existing
  footnote 31.4px 외에 40px `footnote_safety_margin`과 drift margin을 빼므로 한 줄만 남긴다.
- 범위 가설: native HWP5, 한 단, 기존 footnote, 일반 text paragraph, 그리고 다음 stored reset 전의
  연속 positive `LINE_SEG`만 실제 footnote top 안에 드는지 확인해 safety margin을 완화한다. 표·그림·각주
  control·reset 이후 line 및 일반 미저장 흐름에는 적용하지 않는다.

### Stage 15로 넘기는 p77 — 그림 51 TAC picture+caption 표가 p78 단독 쪽을 생성

- 문단 876은 `treat_as_char=true`, `RowBreak` 2×1 표이며 첫 row는 Picture `bin_id=51`, 둘째 row는
  `그림 51.` caption이다. p77 `export-text`에는 caption이 없고 p78에는 caption만 있다.
- p77 render tree의 마지막 본문 line은 `y=752.5px`, footnote top은 `y=945.3px`이고 table 876의 계산 높이는
  `209.8px`다. 현재는 table을 p78에 통째로 보내고 p78에 그림/caption만 둔다. PDF는 그림 51과 caption을
  p77 footnote 위에 함께 둔다.
- p77 상단의 continuation table 866은 rhwp `h=243.2px`로 계산돼 기준 PDF의 해당 fragment보다 크게 잡힌
  것으로 보인다. 따라서 TAC table의 near-fit만 완화하기 전에 continuation row geometry와 preceding flow를
  먼저 대조한다. 그림 51을 p77에 넣는 것은 이 선행 height가 물리적으로 회수되는 경우에만 허용한다.
- 이 p78 단독 쪽이 생긴 뒤 rhwp p79는 PDF p78의 `3. EU`를 시작한다. 그러므로 사용자가 보고한 p78–p81,
  p83–p84, p87, p90, p99–p100의 같은-번호 차이는 우선 이 첫 page shift의 연쇄인지, 독립 overflow인지
  구분해야 한다.

### Stage 15로 넘기는 p83 — page-shift와 별개의 full paragraph overflow 후보

- native export는 `LAYOUT_OVERFLOW`를 기록했다: `page=82`, `para=897`, `FullParagraph`, final line
  bottom `1160.0px`, column bottom `1039.3px` (최대 120.7px 초과).
- 이 결함은 p77 page shift만으로 설명하지 않고, paragraph 897의 full-fit/line-split 분기와 실제 renderer
  geometry를 별도로 확인한다.

### Stage 15로 넘기는 p66 사용자 UI 관측

Stage 13의 fresh native SVG/raster p66에서는 본문·각주 ink가 겹치지 않았다. 그러나 사용자가 UI에서
충돌을 다시 관측했으므로, Stage 14에서는 사용자 수동 WASM build를 다시 실행하지 않고 그 산출물의 head와
native render tree의 page/footnote geometry가 같은지 먼저 읽기 전용으로 확인한다. 서로 다르면 stale artifact
또는 rendering-path 차이로 기록하고, 같으면 p66도 별도 source-level 회귀로 승격한다.

## 구현과 결과

`typeset.rs`의 line-split loop에 native HWP5 existing-footnote reset-tail 후보를 추가했다. 적용 조건은
한 단, visible plain text, control 없음, 기존 footnote 존재, 첫 stored line이 body 하단 30%에 있으며,
positive `LINE_SEG` 뒤에 `vpos=0` reset이 있는 경우다. candidate line range가 현재 flow tail과 16px 안에서
정렬되고 physical `FootnoteArea` top 안에 전부 들어가야만 일반 40px safety margin을 넘길 수 있다. reset
이후 줄·표/그림/각주 control·다단·일반 저장 흐름은 기존 fit 규칙을 그대로 사용한다.

회귀 test `native_hwp5_reset_tail_uses_the_actual_existing_footnote_boundary`는 p58에 `호주 정부의 …` 및
`Medical Research Council`이 남고 p59가 `독립적이며 …`로 시작함을 고정한다. focused fixture test 4/4와
release-test native binary build를 통과했다. p58–p59 144 DPI sweep에서는 2/2가 완료되고 structural flag는
없었다. p58은 실제 각주 top 위에 세 줄을 보유하며 p59는 PDF와 같은 문장 경계에서 재개한다. 전체 native
HWP page count는 223에서 222로 줄었지만 PDF의 215와는 아직 다르다.

정량·PNG 증적은 [Stage 14 visual sweep](task_m100_3738_stage14_visual_sweep.md)에 기록한다.

## Stage 15 이관 순서

1. p77 table 866 fragment와 table 876의 geometry를 PDF와 대조하고, root cause를 확정한 뒤 별도 구현한다.
2. p83 full paragraph overflow가 p77 page shift와 독립인지 확인한다.
3. 사용자 UI의 p66 footprint와 native render tree를 비교한다. 수동 WASM build는 다시 실행하지 않는다.
4. 각 구현은 focused test·선택 sweep·결과 보고를 남기고 커밋한다. 남은 첫 결함이 있으면 다시 다음 Stage로
   분리한다.
