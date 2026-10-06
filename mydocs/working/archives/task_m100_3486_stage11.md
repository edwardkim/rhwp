---
kind: investigation
status: active
canonical: mydocs/manual/codex/docs_and_git_workflow.md
last_verified: 2026-07-30
---

# Task #3486 Stage 11 — HWP3 제품명 옛한글 glyph의 의미 판별

- 이슈: [#3486](https://github.com/edwardkim/rhwp/issues/3486)
- 작업 기준: `devel` (`42e1f125dae664bf50f2053784b5e0a213bea2e2`; PR branch는 최종 준비 시 생성)
- 선행 커밋: `d79e38fde` (Stage 10: 24쪽 Canvas/PDF 직접 대조와 bitmap 경계 보정)
- 입력: `samples/HWP3-password-123456.hwp` (24쪽), HWP5/HWPX 동등 문서 fixture
- 오라클: `pdf/HWP3-password-123456.pdf`

## 관찰과 우선순위

Stage 10의 실제 Studio/PDF p1 좌우 증적에서 Studio 제목은 `ᄒᆞᆫ글 97 안내문` 계열의 옛한글 glyph로,
PDF는 현대 `한글 97 안내문`으로 보인다. 사용자도 같은 "한" glyph 차이를 명시적으로 지적했다. 이것은
1px Canvas boundary와 무관한, 페이지 상단에서 즉시 보이는 fidelity 결함이다.

다만 Stage 9는 CanvasKit에 옛한글 shaping 자체가 있고 `ᄒᆞᆫ`은 한 cluster로 shape됨을 확인했다. 따라서
`ᄒᆞᆫ → 한`의 전역 치환은 실제 역사 문서의 옛한글을 훼손할 수 있어 허용하지 않는다. p3 표 bullet/폭,
본문 글꼴과 image 조판은 이 Stage에서 함께 추측해 수정하지 않는다.

## 검증할 가설

HWP3 조합형 decode가 title의 raw code를 언어학적 옛한글 Jamo로 보존했지만, 이 **특정 한컴 제품명
표현**은 당시 전용 글꼴에서 현대 상표명 `한글`로 표시하는 compatibility convention일 수 있다. 이 가설은
다음 모두를 만족할 때만 좁은 보정 후보가 된다.

1. 동일 문서의 HWP3/HWP5/HWPX IR에서 해당 sequence의 code point·char shape·문단/컨트롤 위치를
   비교해 format conversion이 sequence를 보존하는지 확인한다.
2. 24쪽 전체에서 `ᄒᆞᆫ`의 모든 출현 위치를 세고, PDF에서 각각 현대 제품명인지 실제 옛한글인지
   시각 대조한다.
3. raw HWP3 조합형 값·font face·style context가 일반 본문 옛한글과 구별되는지 확인한다.
4. 보정 범위가 확정되더라도 parser의 원문 IR은 보존한다. PDF로 증명된 product-name context에서만
   paint/display projection을 적용하고, 일반 `ᄒᆞᆫ`과 다른 문서에는 영향을 주지 않는 회귀 검사를 둔다.

위 증명이 안 되면 글꼴 asset/대체 policy의 한계로 기록하고 코드 수정하지 않는다. 반대로 좁은
product-name convention이 확인되면 분석 문서·코드·focused test·Studio/PDF p1 재비교를 다음 하나의
commit에 넣는다. 이 문서는 단독 commit하지 않는다.

## 즉시 확인된 배제 근거

선행 Stage 8의 같은 문서 네 형식 대조에는 이미 HWP3, HWP5, 평문 HWPX, 암호 HWPX가 모두
`ᄒᆞᆫ글 97 안내문` IR을 가진다는 기록이 있다. 즉 HWP3 raw decode만의 오류가 아니며, 위 가설의
1번 전제가 성립하지 않는다. Stage 9도 공개 `Source Han Serif K Old Hangul`에서 이 자모열이 한
cluster로 shape됨을 확인했다.

따라서 이 제목은 **HWP3 전용 parser 보정 대상으로 승격하지 않는다**. 원문을 유지한 채 한컴의
독점 legacy glyph/metric에 가까운 font asset 또는 공통 font policy가 확보되기 전에는, 전역 현대
음절 치환을 도입하지 않는 것이 맞다.

현재 worktree에는 Stage 10 커밋에 포함하지 않은 HWP3 table·renderer 관련 병행 변경도 있다. p3의
검은 셀이 현 시점 24쪽 재실행에서 보이지 않는 점을 이 Stage의 공으로 귀속하거나 그 변경을 섞지
않는다. 다음 구현 Stage는 병행 변경이 분리된 뒤 p3 table bullet/폭 또는 다른 PDF로 증명된 독립
원인 하나를 새 분석 문서에서 선택한다.

## 2026-07-30 재개·`devel` 동기화 기록

재개 전에 `upstream/devel`을 fetch하고 로컬 `devel`과 fast-forward 가능 여부를 확인했다. 기준은
`42e1f125dae664bf50f2053784b5e0a213bea2e2`이며 이미 일치했다. 이 동기화에는 #3486의 glyph·table
판정을 바꾸는 변경이 포함되지 않는다.

따라서 이 Stage의 결론도 그대로 유지한다. 제품명으로 보이는 PDF 표시 차이는 실제 사용자-visible
후보이지만, HWP3 parser의 전역 decode 보정이나 `ᄒᆞᆫ → 한` 치환을 정당화하지 않는다. 다음 작업의
권위 경로는 [수행계획서 v2](../../plans/archives/task_m100_3486_v2.md)의 Stage 12-A다. 먼저 visual sweep이
페이지별 SVG raster·PDF raster·compare·overlay·review·page metrics를 모두 성공한 경우에만 원자적
checkpoint를 남기고, 동일 HWP/PDF hash·Git HEAD·rhwp binary·DPI·diff threshold provenance에서만
`--resume`/shard 결과를 합치도록 한다.

이 도구 보강은 완료 sweep을 가장하지 않기 위한 증적 계약이며, p3 table bullet/폭이나 제품명 glyph의
원인을 고쳤다는 판정은 아니다. Stage 12-B에서 독립 한컴 PDF와 Studio Canvas를 다시 잡고,
source → IR → layout → paint 경로가 한 행으로 확정된 뒤에만 최소 코드 보정을 검토한다.

## 2026-07-30 Stage 12-B 판정 보완

Stage 12-B의 p3·p19 한컴 PDF 재대조로, 이 문서의 전역 정규화 배제 결론은 유지하되 **닫힌 제품명
표시 convention**은 별도로 확정했다. raw HWP3/HWP5/HWPX의 `ᄒᆞᆫ글` 계열과 모델 IR은 그대로
보존하고, 한컴 PDF가 현대 글리프로 인쇄하는 `ᄒᆞᆫ글`·`ᄒᆞᆫ메일`·`ᄒᆞᆫ팩스`·`ᄒᆞᆫ소프트`만 최종
`displayText`에 각각 `한글`·`한메일`·`한팩스`·`한소프트`로 투영한다.

첫 composer 경로 보정만으로는 p3 표 셀 제목처럼 `TextRunNode`를 직접 만드는 layout 경로가 남았다.
그래서 render tree 완성 뒤의 공통 순회에도 같은 닫힌 어휘를 적용했다. 이 단계는 parser/IR/search/
caret offset을 바꾸지 않으며 일반 `ᄒᆞᆫ겨울` 같은 옛한글과 CharOverlap 런은 바꾸지 않는다. p3·p19
재실행에서 legacy glyph 후보는 모두 0건이 되었다. 표·줄높이·본문 흐름 차이는 별도 fidelity 결함으로
남아 있으므로, 이 제품명 보정으로 p3 전체가 PDF와 동등하다고 판정하지 않는다.
Stage 12-A의 현재 구현·focused checkpoint 검증은 [Stage 12 작업 기록](task_m100_3486_stage12.md)에
분리해 계속 기록한다.
