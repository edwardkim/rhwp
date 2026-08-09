---
kind: investigation
status: active
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-10
---

# Task #3820 Stage 107 — 정책연구 p87 각주 138 소실

## 범위와 시작 상태

- 브랜치: `task/3820-production-fidelity`
- Stage 107 시작 commit: `3cec519f3`
- 최종 검증 기준 commit: `9291116af` 위 미커밋 Stage 107 변경
- 입력: `samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp`
- 한컴 2020 기준 PDF:
  `pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf`
- 선행 근거: Stage 25의 미판정 후보, Stage 98·102 산출물, Stage 106 재순위화

Stage 106은 p78·p79의 큰 visible-text 신호를 PDF extraction 오탐으로 종결했다.
최신 render tree와 기존 직접 비교에서 다음 실제 잔여는 1-based p87이다.

- 한컴 PDF p87: 본문에 각주 marker 138, 하단에 각주 138 본문 존재
- rhwp p87: `FnMarker=1`이지만 `FootnoteArea=0`
- rhwp p88: 각주 139만 존재해 138이 단순히 다음 쪽으로 이월된 것도 아님

이 stage에서는 각주 138의 source 컨트롤·문단·`LINE_SEG`, marker가 속한
본문/float/table fragment와 page owner 등록 경로를 추적한다. 다른 페이지 휴리스틱을
넓히지 않고 이 source contract에만 최소 보정을 적용한다.

## 초기 가설

1. marker는 paint tree에 남았으므로 파서의 Footnote control 전체 소실은 아니다.
2. 각주 138 본문은 page footnote owner 등록 전에 fragment가 연기·제거되었거나,
   연기 후 p88에 재등록되지 않았을 가능성이 크다.
3. 우선 source에 저장된 명시적 page reset/반복-zero `LINE_SEG`, 표 셀 내부,
   terminal RowBreak, body current-page 경로인지를 구분해 기존 Stage 103·104 보정과
   중복하지 않도록 한다.

## 완료 기준

- 최신 p87 PDF/rhwp 직접 비교에서 각주 138의 marker·separator·본문·쪽 owner가
  한컴 PDF와 일치한다.
- p88의 각주 139 및 본문 흐름을 침범하지 않는다.
- fixture 전체 215쪽, `issue_3738_rowbreak_table_footnote_fragment` 33/33을 유지한다.
- 분석, 수정 전/후 증적, focused 회귀를 이 문서에 기록한 뒤 커밋한다.

## source 분석

`rhwp dump --section 0 --para 937`과 HWP→HWPX 구조 대조로 다음을 확정했다.

- `pi=937`, `controls=2`: `ci=0` 2×2 non-TAC `RowBreak` 표 26, `ci=1` 각주 138.
- 각주 138은 표 셀 내부 문단이 아니라 **표 host/caption 문단의 최상위 형제
  control**이다.
- 각주 본문은 1개 문단, 1개 저장 줄이며
  `의학적 적합성 판단을 위해 수집되어야 하는 자료(2010/53/EU) 부록 내용 표로 정리`를
  가진다. reset/fragment 특수 형상은 없다.

같은 fixture의 표+최상위 각주 형제는 정확히 3건이다.

| host | 표 | 각주 | 한컴 PDF 본문 owner |
| --- | --- | ---: | --- |
| pi 937 | 표 26 | 138 | p87, 표가 끝나는 같은 쪽 |
| pi 962 | 표 27 | 142 | p91, p90→p91 표의 terminal fragment 쪽 |
| pi 1000 | 표 28 | 147 | p95, p94→p95 표의 terminal fragment 쪽 |

세 건 모두 각주 marker는 caption에 표시되지만, 각주 본문의 물리 owner는 표의
마지막 fragment가 있는 쪽이다. 형제 각주 control을 처리할 때 그 terminal fragment는
아직 current items에 있거나, table-cell 각주 분할 때문에 이미 completed page로
flush된 상태일 수 있다.

## 코드 원인과 수정 계약

`src/renderer/typeset.rs`의 최상위 control loop는 `Control::Footnote`를
`if !has_table` 안에서만 등록한다. 표 셀 내부 각주는 별도 queue가 담당하지만,
이 조건은 표 host의 **형제 각주**까지 cell 각주로 잘못 간주해 세 건 모두
본문을 등록하지 않는다.

수정은 표가 있는 문단의 **유일하고 표에 직접 인접한 최상위 형제 각주**에 대해서만
다음 계약을 적용한다.

1. 셀 내부 각주 queue와 중복하지 않고 `FootnoteSource::Body` 출처를 유지한다.
2. 동일 표의 terminal fragment가 current items 또는 completed page에 실제로
   존재하는지를 검증한 뒤 그 다음의 현재 각주 lane에 등록한다.
3. layout과 같은 composed-line 높이로 새 fragment의 projected fit을 계산한다.
   terminal 표와 같은 current page의 공간이 부족하면 표 owner를 먼저 확정하고
   fresh page에 각주를 정확히 한 번 보존한다.
4. 일반 문단 각주의 reset·collision·multi-note route와 table-cell queue의 owner
   규칙은 변경하지 않는다.
5. p87/p91/p95의 138/142/147 owner와 p88/p90/p94의 비소유를 하나의 실물
   integration 회귀로 고정한다.

## 수정 전 실패 재현

실물 fixture 회귀
`native_hwp5_table_host_footnotes_follow_the_terminal_fragment_page`를 먼저 추가해
표 host의 세 형제 각주 owner를 동시에 고정했다.

- p87: 각주 138 본문과 고유 문구를 소유하고 p88은 소유하지 않아야 한다.
- p91: 각주 142 본문과 고유 문구를 소유하고 p90은 소유하지 않아야 한다.
- p95: 각주 147 본문과 고유 URL을 소유하고 p94는 소유하지 않아야 한다.

수정 전 실행은 p87 각주 138 부재 assertion에서 예상대로 실패했다.

```text
test native_hwp5_table_host_footnotes_follow_the_terminal_fragment_page ... FAILED
p87은 표 26 terminal fragment와 형제 각주 138을 같이 소유해야 함
test result: FAILED. 0 passed; 1 failed; 30 filtered out
```

단순히 `if !has_table` 가드만 제거하는 구현은 채택하지 않는다. pi 962의 p90에는
기존 각주 141이 있어 일반 문단용 `multi_note_routed`가 각주 142를 marker 쪽 p90으로
소급할 수 있지만, 한컴 PDF의 실제 본문 owner는 표 terminal fragment 쪽 p91이다.
따라서 표 host의 최상위 형제 각주는 일반 본문 owner heuristic을 타지 않고, 표의
terminal fragment가 current 또는 flushed 상태임을 확인한 뒤 현재 각주 lane에
등록해야 한다.

## 구현

- native HWP5 단일단에서 문단에 최상위 `Footnote`가 정확히 하나, 최상위 `Table`이
  정확히 하나이고 각주 control index가 표 control index 바로 다음인 경우만 후보로
  삼는다. 표는 visible numbered
  caption, `TopAndBottom`/`RowBreak`, 양수 offset 안에 저장 host line 전체가 들어가는
  기존 좁은 predicate를 그대로 만족해야 한다.
- `native_table_host_terminal_fragment_placement`는 동일 표의 완료 `Table`, 또는
  `end_row == row_count`이고 `end_cut`이 빈 `PartialTable`을 current items에서 먼저
  찾고, 없으면 completed pages에서 찾아 `Current`/`Flushed`를 구분한다. 둘 다 아닌
  경우에는 이 보정 경로를 타지 않는다.
- non-split 각주의 content 높이는 layout과 같은 composed line의 `line_height`와
  `line_spacing`을 합산한 exact metric을 쓴다. 문단 자체가 없는 malformed Body 각주는
  실제 layout처럼 0을 반환하고, table-cell queue만 기존 400HU 최소 예약값을 유지한다.
  stored-line reset 각주는 기존 split 계약의 prefix/suffix 높이와 separator flag를
  유지한다.
- 최종 감사에서 이 exact metric에 table-cell queue의 400HU 최소 예약값이 섞인 것을
  발견했다. 문단 자체가 없는 malformed Body 각주는 실제 layout처럼 0, 200HU 한 줄은
  200HU, 비어 있는 문단만 400HU로 재도록 분리했다. table-cell queue는 별도 helper에서
  기존 400HU 하한을 유지하므로 queue의 owner·capacity 계약은 바뀌지 않는다.
- projected fit은 table-cell queue와 공유하는
  `footnote_fragment_fits_current_page`로 계산한다. 이 helper는 기존 각주 높이에 새
  fragment·separator·note 간격을 더하고, footer가 없는 page의 footer band 회수,
  safety margin, zone/bottom exclusion을 반영한다. terminal fragment가 `Current`이면
  32px 물리 guard, `Flushed`이면 0px guard를 사용한다.
- fit 부족은 후보 탈락으로 처리하지 않는다. 현재 page를 `force_new_page`로 확정한 뒤
  fresh page에 각주를 정확히 한 번 등록해 `has_table` no-op에 의한 데이터 소실과
  terminal 표/각주 겹침을 함께 막는다.
- 등록은 `register_unqueued_table_footnote_with_content_height`를 사용해 exact composed
  높이 예약과 기존 stored-reset fragment 계약을 공유한다.
- table-cell queue도 같은 fit helper를 사용하지만 기존 terminal/multi-column safety
  margin과 12/32/0px guard 선택, owner/fragment 순서는 그대로다. 표가 없는 일반
  문단의 reset·collision·multi-note route도 변경하지 않는다.

## focused 검증

2026-08-10, `CARGO_INCREMENTAL=0`, `target/pr-review`, `release-test`로 실행했다.

- 수정 전 정확 회귀: 0/1, p87 각주 138 부재로 예상대로 실패.
- 수정 후 정확 회귀: 1/1 passed. p87/88, p90/91, p94/95의
  138/142/147 owner를 동시에 고정했다.
- composed 높이 정책 단위 회귀: 1/1 passed. 무문단 Body 각주 0, 200HU 단일 줄,
  빈 문단 400HU 및 table-cell queue의 400HU 하한을 각각 고정했다.
- 공간 부족 확대 각주 fallback: 1/1 passed. terminal p87에는 겹치지 않고 fresh
  p88에 각주 138이 정확히 한 번 남으며 FootnoteArea가 footer를 넘지 않는다.
- terminal 표 page가 먼저 flush된 합성 회귀: 1/1 passed. p87의 table-cell 각주
  60000 prefix와 p88의 번호 없는 tail로 terminal page를 먼저 확정한 뒤에도, 형제
  각주 138이 p88에 정확히 한 번 보존되고 footer를 넘지 않는다.
- Body exact metric과 table-cell queue 하한 분리 단위 회귀: 1/1 passed. 문단 없는
  각주 0/400HU, 200HU 한 줄 200/400HU, 빈 문단 400HU를 각각 고정한다.
- exact metric 보정 직후 우선 게이트
  `issue_2430_cell_rewrap_threshold`: 2/2 passed.
- `issue_3738_rowbreak_table_footnote_fragment` 전체: 33/33 passed.

실물 회귀는 owner 문자뿐 아니라 다음 기하를 함께 고정한다.

- 각주 138/142/147은 p87/p91/p95에서 각각 정확히 한 번만 나타난다.
- p88의 139, p90의 141, p91의 143–145는 그대로 유지된다.
- p87/pi937, p91/pi962, p95/pi1000 표 하단과 후속 본문 하단은 각주 separator
  위에 있고, 각주 영역 하단은 footer를 넘지 않는다.

## 선행 한컴 PDF 직접 재대조

Current/Flushed terminal 판정과 exact-height 예약을 최종 고정하기 전의 구현으로
1-based p86–p95 10쪽을 raster화해 한컴 2020 기준 PDF와 페이지별로 직접 비교했다.
아래 결과와 assets는 원인·owner 판정의 선행 증적으로 유지한다. 최종 고정 source와
동일 binary로 다시 수행하는 visual sweep은 아직 pending이며, 아래 수치를 최종
시각 게이트 통과로 해석하지 않는다.

- PDF/render tree 쪽수: 215/215.
- 직접 raster 대조에서 p87의 marker와 각주 본문 138이 같은 쪽에 복원됐고 p88은
  139만 유지한다. 자동 회귀는 각주 본문 owner와 기하를 고정한다.
- p90은 141만, p91은 142–145를 소유한다.
- p94에는 147이 없고 p95에 marker·각주 본문 147이 함께 존재한다.
- p86–p95 `layout-candidates.tsv`의 body/footnote, table/footer, frame 밖 배치,
  cell text overlap은 모두 0건이다.
- pixel diff 14.37–18.65%는 글꼴 raster 차이가 중심이며 각주 owner·표 fragment·
  본문 흐름 판정은 PDF와 일치한다.
- 재현 가능한 중간 산출물은
  `output/task-3820-stage107-policy-p086-p095-after/`에 두었고, 추적 증적에는 입력
  HWP/PDF SHA-256을 함께 기록했다. 작업지시자의 최종 시각 승인 전까지 이 판정은
  직접 대조 후보 해소 상태로 둔다.

증적:

- [p87 수정 전](../pr/assets/task_m100_3820_stage107_policy_p087_footnote138_loss/compare_p087_before.png)
- [p87 수정 후](../pr/assets/task_m100_3820_stage107_policy_p087_footnote138_loss/compare_p087_after.png)
- [p88 수정 후](../pr/assets/task_m100_3820_stage107_policy_p087_footnote138_loss/compare_p088_after.png)
- [p90 수정 후](../pr/assets/task_m100_3820_stage107_policy_p087_footnote138_loss/compare_p090_after.png)
- [p91 수정 후](../pr/assets/task_m100_3820_stage107_policy_p087_footnote138_loss/compare_p091_after.png)
- [p94 수정 후](../pr/assets/task_m100_3820_stage107_policy_p087_footnote138_loss/compare_p094_after.png)
- [p95 수정 후](../pr/assets/task_m100_3820_stage107_policy_p087_footnote138_loss/compare_p095_after.png)
- [pixel report](../pr/assets/task_m100_3820_stage107_policy_p087_footnote138_loss/report.tsv)
- [page-count ledger](../pr/assets/task_m100_3820_stage107_policy_p087_footnote138_loss/page-count-ledger.tsv)
- [layout ledger](../pr/assets/task_m100_3820_stage107_policy_p087_footnote138_loss/layout-candidates.tsv)

같은 선행 구현으로 `visual_sweep.py`의 3-way review를 p87·88·90·91·94·95에
실행했다. 요청/완료는 6/6, SVG/render tree는 215/215, raster/review/overlay는
각각 6/6이며 자동 visual flag는 0건이었다. 평균 pixel match는 89.959%, 최저는
89.097%였고, `visual_accuracy_proxy_percent`는 글꼴 raster 차이에 민감해 평균
9.000%이므로 합격 수치로 해석하지 않는다. 최종 고정 source의 3-way review와
페이지별 PDF 직접 판정은 별도로 다시 수행해야 한다.

- [p87 3-way review](../pr/assets/task_m100_3820_stage107_policy_p087_footnote138_loss/review_p087_final.png)
- [p88 3-way review](../pr/assets/task_m100_3820_stage107_policy_p087_footnote138_loss/review_p088_final.png)
- [p90 3-way review](../pr/assets/task_m100_3820_stage107_policy_p087_footnote138_loss/review_p090_final.png)
- [p91 3-way review](../pr/assets/task_m100_3820_stage107_policy_p087_footnote138_loss/review_p091_final.png)
- [p94 3-way review](../pr/assets/task_m100_3820_stage107_policy_p087_footnote138_loss/review_p094_final.png)
- [p95 3-way review](../pr/assets/task_m100_3820_stage107_policy_p087_footnote138_loss/review_p095_final.png)
- [overlay metrics](../pr/assets/task_m100_3820_stage107_policy_p087_footnote138_loss/overlay_metrics.json)
- [visual sweep manifest](../pr/assets/task_m100_3820_stage107_policy_p087_footnote138_loss/visual_sweep_manifest.json)

## 최종 검증 게이트

2026-08-10 최종 고정 source와 `CARGO_TARGET_DIR=target/pr-review`,
`CARGO_INCREMENTAL=0` 기준으로 다음을 순차 실행했다.

- `issue_3738_rowbreak_table_footnote_fragment`: **33/33 passed**. 실물 owner·기하
  회귀와 공간 부족 fallback, terminal page 선-flush 합성 회귀를 포함한다.
- 인접 focused 회귀: #4138 2/2, #2097 1/1, #1858 1/1, #1937 1/1,
  #2559 1/1, 모두 passed.
- `cargo test --profile release-test --tests`: **exit 0**. lib 3384 passed / 13 ignored,
  전체 integration·실물 fixture를 끝까지 실행했다. 과거 실패 지점인
  `overflow_cell_lines_do_not_grow`도 674개 샘플 스윕을 67.62초 수행해 1/1 passed,
  마지막 `visual_baseline_all_samples`까지 passed.
- `cargo clippy --all-targets --target-dir target/pr-review -- -D warnings`:
  **exit 0**.
- Native Skia 게이트: lib `skia` 58/58, #2225 2/2, direct PDF p37 4/4,
  모두 passed.
- 최종 고정 source와 동일 binary의 p86–p95 visual sweep 및 한컴 PDF 직접 판정:
  **pending**.
- WASM build와 브라우저 최종 확인: **작업지시자 수동 게이트 / pending**.

## Stage 96 p65 대체폰트 재감사

추가 요청에 따라
`stage96-issue2279-r27-final/svg/86712_regulatory_analysis_065.svg`를 원본 HWP와
한컴 2024 기준 PDF p65에 다시 대조했다.

- SVG의 `□비용` 첫 글자는 missing-glyph 두부가 아니라 원문과 공식 PDF에 실제로
  들어 있는 `U+25A1 WHITE SQUARE` 체크박스다.
- SVG의 `U+FFFD`와 PUA는 각각 0건이며, 사용된 `H2GTRM.TTF`/HY중고딕은 `□`, `비`,
  `용` 글리프를 모두 가진다.
- 기존 `한양중고딕 → HY중고딕/HYGothic-Medium` 별칭, 휴먼명조·휴먼고딕의 outline
  fallback 우선, fidelity 비교의 `--font-style` 계약과 회귀가 이미 적용돼 있다.

따라서 p65의 사각형을 제거·치환하거나 전역 symbol fallback을 추가하면 오히려 원문과
PDF를 훼손한다. 이 재감사에서는 추가 폰트 코드를 변경하지 않았다.

## 판정

표 host의 최상위 형제 각주 138/142/147은 셀 각주 queue 대상이 아니지만 기존
`has_table` 가드 때문에 Body 등록에서도 제외되고 있었다. native HWP5의 저장된 번호형
RowBreak 표 host, 유일한 표 뒤 형제 각주, Current/Flushed terminal fragment라는
좁은 계약에서만 기존 각주 등록 경로를 재사용해 p87/p91/p95 소유권을 복원했다.
focused 33/33과 두 합성 회귀, 전체 release-test, Clippy, Native Skia까지 통과했다.
최종 고정 binary의 6쪽 visual sweep과 한컴 PDF 직접 판정을 마지막으로 기록한 뒤
Stage 107을 커밋한다.
