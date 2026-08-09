# Stage 98 — #4138 셀 분할 뒤 페이지 수: 회귀 게이트와 한컴 오라클 분리

## 목적

`issue1949_giant_cell_nested_tables_perf.hwp`에서 셀을 1×2로 분할한 뒤, stale
`LINE_SEG` 재래핑과 vpos 사다리 재구축은 유지하면서 페이지 수가 처음 191쪽으로 줄어든
원인을 확인한다. 195쪽이라는 기존 게이트가 한컴 저장 결과와 같은지도 별도로 검증한다.

> 이 문서는 가설과 기각 과정을 시간순으로 보존한다. 최종 판정과 채택 구현은 문서 끝의
> **최종 판정** 절을 권위 결과로 삼는다.

## 관찰과 재현

2026-08-10 현재 작업 트리에서 다음 전체 integration 회귀가 실제로 실패했다.

```text
cargo test --profile release-test --tests
exit 101
issue_4138_split_cell_stale_linesegs:
  split_cell_into_reflows_stale_segs_and_rebuilds_ladder: 191, expected 195
  split_cells_in_range_reflows_stale_segs: 191, expected 195
```

두 테스트 모두 stale text `LINE_SEG` 0개와 vpos 사다리 역행 0개라는 본래 #4138의
안전 계약은 통과한 뒤, 최종 쪽수 핀에서만 실패한다.

## 이력 분리

- 깨끗한 `1ea5f0210` 기준도 같은 191/195 실패였다. 그러므로 현재 Stage 96/97의
  중첩 표 조각 수정이 새로 만든 실패가 아니다.
- #4138을 처음 도입한 `bf803bf53`에서는 두 테스트가 모두 195쪽으로 통과한다.
- `bf803bf53..1ea5f0210` bisect 결과 첫 불량 커밋은
  `380891a5b2e1d04dc582658fcb7439fa09b2f3ee` (`fix: #3820 빈 문단 페이지 소유 정렬`)이며,
  직전 `b5e12293f`는 195쪽으로 통과한다.

따라서 기존 쪽수 게이트를 즉시 191로 낮추지 않는다. 반대로, 195가 한컴의 편집 후
저장 결과라는 증거도 아직 없으므로 단지 과거 구현값을 복원하지 않는다.

## 현재 가설

`380891a5b`는 `한양신명조` 12pt 이상 U+0020 공백을 518/1024em 대신
411/1024em으로 측정하는 PDF 보정을 추가했다. 해당 변경은 `reflow_line_segs`가 셀 분할
직후 새 폭에서 모든 stale 문단을 다시 줄바꿈할 때 직접 적용될 수 있다. 공백 폭 축소가
긴 셀의 줄 수를 줄이면 4쪽 차이가 생길 수 있다.

빈 문단 fallback 및 nested-tail height 경로에는 일시 계측을 넣어 #4138 실행에서 호출되지
않음을 확인했다. 그러므로 이 두 경로를 추측으로 되돌리지 않는다.

## 검증 순서

1. `tests/issue_2430_cell_rewrap_threshold.rs`를 우선 실행한다.
2. 분할 후 문서를 HWP로 직렬화하고 HWP 2020으로 PDF를 산출한다.
3. 한컴 PDF의 쪽수와 행/문단 경계를 191·195 후보 및 RHWP 출력과 직접 비교한다.
4. PDF 원인이 공백 폭 적용 범위이면, 원명·문서 종류·편집 재래핑 경로를 근거로 좁힌
   수정과 회귀 게이트를 만든다. 반대이면 기존 쪽수 게이트의 근거를 갱신한다.

## 불변 조건

- stale text `LINE_SEG`는 0개여야 한다.
- 대상 행의 vpos 사다리는 단조여야 한다.
- `#3820` PDF p81 한양신명조 표의 line decision을 근거 없이 훼손하지 않는다.
- 최종 수정 뒤 focused 테스트와 전체 `release-test --tests`를 모두 실제 완료 상태로 확인한다.

## 한컴 2020 오라클 결과

실제 `split_table_cell_into_native(0, 0, 2, 2, 0, 1, 2, false, false)` 뒤 HWP를
직렬화하여 한컴 2020 `PrintToPDFEx`로 다시 출력했다. 이 결과는 원 HWP를 단순 변환한
PDF가 아니라, 문제의 편집을 실제로 적용한 저장본이다.

| 재래핑 U+0020 advance | RHWP 페이지 | 저장 HWP 재파싱 | 한컴 2020 PDF |
| --- | ---: | ---: | ---: |
| 411/1024em (기존 #3820 12pt 분기) | 191 | 191 | 197 |
| 512/1024em (일반 반각) | 195 | 195 | 197 |
| 550/1024em | 196 | - | - |
| 565/1024em | 197 | 197 | 197 |
| 580/1024em | 198 | - | - |

대상 셀은 원명 `한양신명조`, 12pt, bold, ratio 100%, spacing 0이며, 원래는 3,175줄,
1×2 분할 뒤 재래핑된다. 그러나 이 표의 문서 전체 쪽수를 565/1024em으로 맞춘 것은
실제 line decision을 재현한 것이 아니라 하단 overflow를 공백 폭으로 상쇄한 우연한 결과다.

한컴 2020이 두 HWP 후보를 HWPX로 재저장한 `Contents/section0.xml`은 1,922,492 bytes로
완전히 동일했다. 대상 문단의 첫 page line start도 한컴은
`[0, 23, 41, 61, 79, 96, 114, 131, 147]`인데, 512/1024em RHWP는 마지막 경계만
`148`로 한 글자 차이여서 실제 줄바꿈을 거의 그대로 따른다. 565/1024em은
`[0, 23, 41, 61, 79, 95, 112, 128, 144]`로 이미 여섯째 줄부터 다르다.

따라서 #3820 p81에서 검증된 14pt/HWPX의 411/1024em 보정과 12pt HWP 재래핑을 임의로
분리하지 않는다. 남은 195/197 차이는 글자폭이 아니라 1×N `RowBreak` 부분 표의
페이지네이터가 64px HWPX 허용치를 native HWP에도 적용해 본문 하단을 최대 22px 넘긴
32개 조각을 수용한 데서 발생한다.

최종 한컴 기준 파일은 다음이다.

- HWP: `mydocs/pr/assets/task_m100_3820_stage98_4138_split_cell_page_count_oracle/oracle/issue1949_split_1x2_hancom_matched_565.hwp`
- PDF: `mydocs/pr/assets/task_m100_3820_stage98_4138_split_cell_page_count_oracle/oracle_pdf_hancom_matched_565/issue1949_split_1x2_hancom_matched_565_2020.pdf`
- 검증: HWP editor page 197, PDF page 197, validation `ok`.

## 게이트 정정 보류와 실제 보정 대상

기존 `tests/issue_4138_split_cell_stale_linesegs.rs`의 195쪽은 #4138 도입 당시 RHWP
실측값일 뿐 한컴 오라클이 아니므로 단독 정답으로 삼지 않는다. 다만 565 공백폭으로
197쪽을 만드는 것도 잘못된 물리 원인이므로 게이트 변경은 보류한다. 먼저 native HWP
`RowBreak`에는 2px 경계, 저장 HWPX에만 64px 측정 drift 여유를 적용하여 overflow를 없애고,
그 뒤 한컴 PDF의 197쪽 및 page-by-page 경계를 다시 비교한다.

## 현재 검증

- `issue_2430_cell_rewrap_threshold`: 2 passed (직전 계측 변경 뒤 실행).
- 한컴 HWPX 재저장 oracle: 512·565 후보 `section0.xml` raw equality 확인.
- native HWP 512 후보: 195쪽, `LAYOUT_OVERFLOW` 32개(최대 22.1px). 이것이 다음 코드
  보정 전 baseline이다.
- #3820의 p81/p82, #4138 focused 및 전체 integration 회귀는 RowBreak 보정 뒤 다시 실행한다.

## 현재 source 경로와 PDF 검증의 분리

2026-08-10 현재 focused 실행에서 `issue_2430_cell_rewrap_threshold`는 2/2 통과했다.
`issue_4138_split_cell_stale_linesegs`는 두 API 경로 모두 191쪽으로 실패한다. 이 값은
native HWP를 편집한 메모리 문서가 직렬화·재파싱되기 전의 `page_count()`이며, 512/565
후보 HWP를 재파싱한 195/197쪽 비교와 같은 입력 경로가 아니다. 따라서 이를 197로 단순
승격하거나, HWPX의 64px tolerance를 global하게 줄여 맞추지 않는다.

native continuation row에도 2px 경계를 강제하는 실험은 원 분할 전 표를 115쪽에서 3쪽으로
붕괴시켰다. `end_row == cursor_row`인 거대 continuation은 재-cut 실패 시 진행 상태를 잃을
수 있으므로, 그 실험은 즉시 되돌렸다. 후속 수정은 이 경로의 `res2`/cursor 보존을 별도
검증한 뒤에만 한다.

### 76076 p33--p34 current PDF 재판정

현재 `target/pr-review/release-test/rhwp`로
`samples/76076_regulatory_analysis.hwp`와 한컴 2024 PDF p33--p34를 직접 raster 비교했다.
산출물은 `output/task-m100-3820-stage98-current-76076-p33-p34/`에 남겼다.

- p33 diff 19.72%, p34 diff 14.81%는 주로 글꼴 획·자폭 차이다.
- p34 nested cell의 visible text right는 outer border 안쪽에 남았고,
  `table-cell-text-boundary-candidates.tsv` 및 `table-cell-text-overlap-candidates.tsv`는 후보 0건이다.
- p34 연속 표의 top stroke는 body clip에 0.5만 보이는 구조 후보 1건이지만, 한컴 PDF도
  연속 fragment의 물리 상단에서 같은 border를 공유한다. 사용자 화면에서 보였던 우측선 침범을
  이 current SVG/PDF pair로 재현하지 못했으므로 코드 변경 근거로 쓰지 않는다.

### #1921 p8 current PDF 재판정

`samples/issue1921/59043_regulatory_analysis.hwp`와 한컴 2022 PDF p8의 direct 비교는
`output/task-m100-3820-stage98-current-issue1921-p8/`에 남겼다. raster diff 31.27%는
font/이미지 raster 차이가 크지만, 표 6행 좌측 셀의 사진 2개는 모두 cell 안에 있고 PDF와
같은 page owner다. 기존 `regulatory_59043_page8_square_picture_stays_in_its_table_cell`
회귀가 정확히 이 계약(사진 2개 + bottom containment)을 이미 고정하므로 중복 게이트는
추가하지 않는다. 같은 binary에서 #1921 전체 focused 5/5, #2308 5/5, #3820 p35--p36 2/2를
통과했다.

### native RowBreak 물리 tail 보정 1차 결과

`advance_row_cut`의 논리 `consumed_height`와 실제 표시 fragment 높이가 어긋나는
native continuation을 재-cut하도록 바꾼 뒤 다시 측정했다. 첫 번째 문제 fragment는
논리 컷이 992px인 반면 paint footprint는 1,019.5px여서, 종전의 `budget - over` 재시도는
같은 표시 높이를 다시 선택하고 실패했다. 이 때 paint tail 25.6px까지 재시도 예산에서
제외하면 cursor를 보존한 채 다음 source unit부터 새 fragment가 시작한다.

- 원본 `issue1949_giant_cell_nested_tables_perf.hwp`: 115쪽 유지.
- `issue_2430_cell_rewrap_threshold`: 2/2 통과.
- #4138 편집 직후 메모리 문서: 191쪽에서 193쪽으로 개선됐지만, historical pin 195와
  한컴 2020 PDF 197에는 아직 미달한다.
- 512/1024em split HWP 재파싱: 195쪽에서 196쪽으로 개선됐지만, 한컴 PDF 197보다 1쪽
  부족하다. full text-owner ledger는 PDF 글꼴 인코딩 때문에 공통 문자열 boundary를
  만들지 못했으므로, 이 값을 page-count 정답으로만 사용하지 않고 native RowBreak의
  남은 경계 허용을 별도로 계측한다.

1차 보정은 최대 22.1px overflow 32개를 제거했지만 완결이 아니다. native RowBreak의
일반 허용을 2px에서 0.1px로 좁힌 대조 실험은 #4138 메모리 문서 193쪽과 native HWP
재파싱 196쪽을 전혀 바꾸지 못했다. 따라서 이 전역 경계 변경은 채택하지 않고 2px로
되돌렸다.

### HWP/HWPX page ledger: 남은 한 쪽의 실제 갈림점

같은 512/1024em 편집 HWP를 한컴 2020으로 HWPX로 재저장한 뒤, RHWP의
`dump-pages --json`으로 각 physical page의 `startCut/endCut`을 대조했다.

- native HWP 재파싱은 196쪽, 한컴 재저장 HWPX는 197쪽이다.
- p1--p3의 row cut은 동일하다. 처음 달라지는 p4는 native `[115,1] -> [153,1]`
  (used 973.1px), HWPX `[115,1] -> [154,1]` (used 998.7px)다.
- 마지막은 native p196 `[7488,1] -> end`인 반면 HWPX는 p196
  `[7480,1] -> [7519,1]`, p197 `[7519,1] -> end`다.
- 두 HWPX oracle의 `Contents/section0.xml` hash는
  `42bba2c411322e369707a5b3e52349932472c744722f534f5a55906fb43398d2`로 같으며,
  대상 p9의 HWPX `LINESEG.vpos`는 69000HU 뒤 `0, 1920, 3840, 5760`으로 page-local
  reset된다. 원 native HWP의 같은 line segment는 `61320..74760`으로 계속 증가한다.

즉 565/1024em 공백폭이 아니라, native HWP의 continuation 재조판이 한컴이 저장한
page-local reset 경계를 완전히 재현하지 못하는 것이 남은 차이다. 다음 보정은
`!has_stored_line_segs` 전체에 2px를 적용하는 broad policy를 원래의 64px 측정 여유로
복원하고, 실제 `row_start_cut`이 있는 native continuation의 physical paint tail에만
0.1px strict cut을 적용한다. 이렇게 해야 HWPX 저장 layout의 64px drift 계약과 관련 없는
native RowBreak 행의 기존 배치를 바꾸지 않는다. 변경 뒤에는 #2430을 먼저 실행하고,
#4138/HWPX ledger/원본 115쪽을 다시 확인한다.

### 2차 strict-cut 판정 (미채택)

위의 narrow strict-cut을 적용한 동일 release-test binary에서 다음을 재실행했다.

- `issue_2430_cell_rewrap_threshold`: 2/2 passed.
- 원본 `issue1949_giant_cell_nested_tables_perf.hwp`: 115쪽 유지.
- 512/1024em split HWP 재파싱: 196쪽, 한컴 재저장 HWPX: 197쪽.
- `issue_4138_split_cell_stale_linesegs`: 두 API 경로 모두 193쪽으로 historical 195 pin에서
  실패.

따라서 continuation paint tail은 실제 결함이지만 남은 1쪽의 원인은 아니다. 이 변경은
#2430와 원본을 보존하는 범위에서만 유지하고, #4138의 page-count expectation이나 full
integration gate는 아직 바꾸지 않는다. 다음 분석 대상은 `rebuild_table_cell_vpos_ladder_native`
가 폭 변경 뒤 모든 line segment를 단조 누적시켜 한컴 재저장 HWPX의 physical-page reset을
소거하는 경로다. reset을 그대로 복원하면 과거 222쪽 hard-break 회귀가 있었으므로, 저장
reset을 native hard break로 오인하지 않는 page-local row-cut 신호를 먼저 분리해 검증한다.

조사 전용으로 같은 split 뒤 source provenance만 `Hwpx`로 바꾼 focused 실행은 191쪽이었다.
이는 HWPX profile의 tolerance/분기만으로는 한컴 197쪽을 재현하지 못하고, 한컴 재저장본의
실제 per-page `LINESEG.vpos` reset이 필요한 입력 증거임을 확인한다. 이 probe test는
회귀 파일에 남기지 않았다.

### 셀 폭 재래핑의 trailing inline table 조사

native/HWPX dump에서 대상 좌측 셀 2,507문단의 LineSeg 수를 문단별로 비교했다. HWPX는
전체 7,495줄, native는 7,487줄로 8줄이 적다. 그중 p288, p322, p567, p2001, p2286의
다섯 문단은 모두 `text + treat-as-char nested table` 형상으로, native는 표 높이를 본문
첫 줄에 합쳐 1 LineSeg로 저장하고 HWPX는 본문 줄과 표 줄 2개로 저장했다. 예를 들어 p288은
native `lh=17992` 한 줄, HWPX는 `lh=1200` 본문 줄 뒤 `lh=18272` 표 줄이다.

이는 page count만 맞추기 위한 padding 조정이 아니라, 우측 cell clip에서 nested table의
source line owner가 본문과 합쳐지는 실제 구조 결함이다. 첫 narrow probe에서는 trailing
table p322/p2001만 2 LineSeg가 되었고, dump의 실제 control position을 재확인했다.

- p288은 8자 문단의 char index 6에 폭 31,523HU nested table이 삽입된 중간 control이다.
- p2286은 6자 본문 뒤 폭 19,633HU의 trailing inline picture다.
- p567은 object가 아니라 일반 text wrapping 차이다.

따라서 구현은 문단에 inline control이 하나이고 앞에 text가 있을 때, (a) object 폭 자체가
현재 줄 폭을 넘거나 (b) 그 줄의 text prefix 폭과 object 폭의 합이 줄 폭을 넘는 경우에만
control position에서 독립 LineSeg를 삽입한다. 후자는 object가 셀 전체 폭보다 작아도 p2286처럼
본문 뒤의 잔여 폭에 들어가지 않는 경우를 포착한다. 삽입 위치도 vector 끝이 아니라 control의
실제 character offset이 속한 line 직후로 두어, p288처럼 중간에 삽입된 table 뒤의 text 순서를
보존한다. table/picture/shape/equation/form에 공통으로 적용되지만 시작 위치 control·같은 줄에
들어가는 object·복수 control 문단은 기존 경로를 유지한다.

2026-08-10 focused 확인에서 #2430은 2/2 통과했고, #4138의 stale/vpos 및 위 네 inline source
line 계약도 통과했다. 다만 두 분할 API의 페이지 수는 메모리와 native HWP 저장→재파싱 모두
193으로, historical pin 195와 한컴 PDF 197에 아직 미달한다. 따라서 이번 보정은 실제
clip/source-owner 결함을 고정하되, 남은 page-owner 차이는 별도 원인으로 계속 추적한다.

### 저장 HWP와 `standard_halfem` HWPX의 전수 LineSeg 대조

임시 진단으로 현재 코드의 분할 HWP를 native로 저장·재파싱하고, 기존
`issue1949_split_1x2_standard_halfem_2020.hwpx`와 대상 좌측 셀 2,507문단의 LineSeg 수를
전수 대조했다.

| 입력 | 페이지 | 대상 셀 LineSeg 합계 |
| --- | ---: | ---: |
| 현재 native HWP 저장→재파싱 | 193 | 7,339 |
| 한컴 2020 `standard_halfem` HWPX | 197 | 7,492 |

153줄 차이 중 대부분은 control이 없는 일반 문단에서 native가 한 줄 적은 패턴이다. 따라서
이 HWPX는 현재 411/1024em 공백 보정 경로가 만든 저장 HWP의 직접 oracle이 아니라, 별도로
512/1024em을 적용해 산출한 candidate의 한컴 재저장본이다. 이를 현재 입력의 197쪽 정답으로
그대로 단언하거나, 153줄을 RowBreak tolerance로 상쇄해서는 안 된다.

반대로 p288/p322/p2001/p2286의 text+inline-control 분리는 새 회귀 단언으로 고정했고 두 API
경로 모두 통과했다. 이후 비교 기준은 **현재 native 저장 HWP를 한컴 2020으로 실제 HWPX/PDF
재저장한 결과**로 다시 산출한 뒤, 그와 현재 HWP를 대조해야 한다.

현재 저장 HWP를 실제로 한컴 2020에 넘긴 결과도 위 전제를 확증했다.

| 산출물 | 페이지 | SHA-256 |
| --- | ---: | --- |
| RHWP native 저장 HWP | RHWP 재파싱 193 | `ccd56fa8…a8416` |
| 한컴 2020 HWPX 재저장 | RHWP 판독 197 | `d3c79b6c…e91fe` |
| 한컴 2020 PrintToPDFEx PDF | PDF 197 | `3591a275…a1381` |

동일 입력 HWP와 그 한컴 HWPX를 다시 전수 대조하면 대상 셀은 각각 7,339줄과 7,492줄이며,
159개 문단에서 줄 수가 다르다. 처음부터 p5 `7→8`, p13 `8→9`, p31 `6→7`처럼 control 없는
일반 문단에서 HWP가 한 줄 적다. 그러므로 남은 4쪽 차이는 RowBreak tolerance나 위 네 inline
control이 아니라, 셀 분할 후 native HWP 재래핑의 text advance/line-break 모델이 한컴보다
낙관적인 문제다. 다음 단계는 첫 불일치 문단의 text·문단 모양·셀 폭·LineSeg start를 같은
HWP/HWPX에서 대조해, 폭 상수 조정 대신 재현 가능한 line-break 경계로 원인을 한정한다.

첫 불일치 p5의 직접 대조에서 셀 폭(22,395HU), 좌·우 padding(각 141HU), paragraph margin/
indent/line spacing은 같았다. 그러나 다음 차이가 있다.

| 항목 | RHWP 저장 HWP | 한컴 2020 HWPX |
| --- | --- | --- |
| 한글 글꼴명 | `한양신명조` | `HY Sinmyeongjo` |
| LineSeg start | `0, 23, 41, 59, 78, 96, 113` | `0, 23, 41, 58, 75, 93, 110, 126` |
| `column_start` | `0` | `500` |
| paragraph attribute | HWP raw `attr1=0x06000180`, `attr2=8` | HWPX `attr1=384`, `KEEP_WORD` |

문단 속성의 binary/HWPX 표기 차이는 serializer 보존 정보일 수 있으므로 즉시 동작 원인으로
단정하지 않는다. 반면 `한양신명조`와 한컴이 재저장한 `HY Sinmyeongjo`의 별칭 경로는 현재
`text_measurement.rs`의 12pt 공백 411/1024em 보정과 직접 만난다. 다음 검증은 p5의 실제
token 폭을 두 이름으로 각각 계산하고, 411 보정을 HWPX의 한컴 별칭에도 적용할지 또는 native
HWP 재래핑에서 제외할지를 2개 문서(#3820 p81 포함) 오라클로 판정한다.

## 최종 판정 — 현재 입력 오라클과 14pt 전용 보정

과거 `standard_halfem`/565 후보가 아니라 **현재 411 경로에서 직접 생성한 HWP**를 다시
한컴 2020으로 저장·출력했다. 현재 입력 HWP는 아래 호출의 native 직렬화 결과이며, 보관된
`issue1949_split_1x2_current_reflow_20260810.hwp`와 byte-identical이다.

```rust
let mut doc = HwpDocument::from_bytes(&fixture)?;
doc.split_table_cell_into_native(0, 0, 2, 2, 0, 1, 2, false, false)?;
let hwp = doc.export_hwp_native()?;
```

| 산출물 | SHA-256 | 페이지 |
| --- | --- | ---: |
| 원본 sample | `ef10261cd29325116028e4f4f3e6be1a72c675eb771bddfd8484e7fe5aa94b4e` | 115 |
| 당시 411 native HWP | `ccd56fa8bfe9cfa3972bb5d080d64a7deda3f07d946ad987af73177c99a8416c` | RHWP 193 |
| 위 HWP의 한컴 2020 HWPX | `d3c79b6cae9975a2144d5a6acc1ad5bdb6c6e082e589d24a8645da8b137e91fe` | 197 |
| 위 HWP의 한컴 2020 PDF | `3591a275c9741e1be6af4d5a68141316cca069f2c2647305e41bfbe0069a1381` | 197 |

411·512·565 입력을 한컴이 다시 저장한 HWPX의 `Contents/section0.xml`은 모두
`42bba2c411322e369707a5b3e52349932472c744722f534f5a55906fb43398d2`로 같다. 따라서
197쪽은 특정 후보 상수를 골라 맞춘 값이 아니라, 현재 편집 입력에도 적용되는 한컴 오라클이다.

현재 HWP와 그 한컴 HWPX의 대상 셀 2,507문단을 전수 대조한 결과는 다음과 같다.

- native HWP: 7,339줄
- 한컴 HWPX: 7,492줄
- 줄 수가 다른 문단: 159개, 한컴이 순증 153줄
- 최초 차이 p5: native 7줄 `[0,23,41,59,78,96,113]`, 한컴 8줄
  `[0,23,41,58,75,93,110,126]`

p5에는 control이 없고 셀 폭·padding·문단 margin/indent/spacing도 같았다. 원인은
RowBreak 허용치가 아니라 12pt `한양신명조` U+0020 line decision이었다. 전역 512 실험은
#4138을 197쪽으로 만들었지만, 원본 issue1949의 한컴 115쪽을 116쪽으로 바꾸고 #3820 p81의
411 계약까지 덮으므로 채택하지 않았다. 최종 범위는 다음과 같다.

- 원본/일반 12pt `한양신명조`: 411/1024em, issue1949 115쪽 보존
- 셀 분할 직후 stale 재조판의 12pt `한양신명조`: 일반 반각 512/1024em
- #3820 p81의 일반 14pt `한양신명조`: 411/1024em
- 10pt `한양신명조`: 일반 반각 512/1024em

구현은 기존 12pt 이상 411 측정을 보존하고, `reflow_line_segs_after_cell_split`의 전용
tokenization에서 정확한 12pt(16px, 허용 오차 0.01px)에만 512를 반환한다. 이와 함께
좁아진 셀의 `text + inline control`을 독립 physical line으로 분리하고, native continuation의
실제 paint tail만 strict cut하는 보정을 유지한다.

## 최종 focused 검증

공통 환경은 `CARGO_TARGET_DIR=target/pr-review CARGO_INCREMENTAL=0`, profile은
`release-test`다.

- `issue_3820_hanyang_shinmyeongjo_space_is_standard_body_only`: 1 passed
- `issue_4138_split_cell_stale_linesegs`: 2 passed, 두 API 모두 저장·재파싱 197쪽
- `issue_2308_render_normalized_derived_state`: 5 passed
- `issue_2430_cell_rewrap_threshold`: 2 passed

회귀 게이트는 메모리 중간 상태의 `page_count()` 대신 실제 제품 경로인 native HWP
저장→재파싱 쪽수를 검사한다. 동시에 stale text `LINE_SEG` 0개, vpos 사다리 단조,
p288/p322/p2001/p2286의 inline-control 독립 source line을 고정한다.

### 전체 회귀가 밝힌 inline-control 적용 범위

12pt 공백은 일반 반각으로 되돌리고, #4138에서 발견한 `text + treat-as-char control`의
source line 분리를 일반 `reflow_line_segs`에 넣은 뒤 전체 `cargo test --profile release-test
--tests`를 실행했다. #2430 2/2, #4138의 두 split API와 저장→재파싱 197쪽, #3820 p81 14pt
공백 단위 회귀는 통과했다. 그러나 원본 `issue1949_giant_cell_nested_tables_perf.hwp`의 기존
115쪽 계약을 쓰는 다음 네 테스트가 모두 116쪽으로 늘어 실패했다.

- `issue2214_scoped_cache_coherence_preserves_transient_pagination`
- `issue2424_resumable_delete_commits_only_after_final_fragment`
- `issue2424_new_edit_stales_old_job_and_sync_flush_restarts_latest_revision`
- `issue2424_resumable_pagination_commits_only_after_final_fragment`

이 문서의 object 분리는 **셀을 1×2로 나눈 뒤, 폭이 줄어 stale LineSeg를 다시 계산하는
경우**에만 한컴 oracle이 요구한다. 원본 문서는 저장된 control host LineSeg가 이미 권위
경계이므로, 모든 일반 재래핑에 적용하면 안 된다. 따라서 구현은 기본
`reflow_line_segs`의 기존 동작을 유지하고, `reflow_stale_cells_after_split`가 호출하는
전용 reflow에만 inline-control source-line 분리를 opt-in한다. 이는 115쪽 기준을 임의로
바꾸지 않으면서 #4138의 실제 우측 clip/페이지 결함을 고정하는 최소 범위다.

같은 전체 회귀에서 `test_cursor_rect_after_line_break_at_end`도 line-break flag 단언으로
실패했다. 이것은 inline control이 없는 문단이므로 위 범위 축소 뒤에도 남는지 분리해 다시
확인한다. 남으면 현재 작업과 독립된 기존 dirty 변경으로 취급해 원인을 분리한다.

### 원본 115쪽 oracle과 split 197쪽을 함께 만족시키는 공백 범위

원본 HWP의 이미 보관된 한컴 PDF는 115쪽이다.

- `pdf/issue1949_giant_cell_nested_tables_perf-2020.pdf`: 115쪽
- `pdf/issue1949_giant_cell_nested_tables_perf-2024.pdf`: 115쪽

따라서 전체 `issue2424`가 115→116으로 바뀐 상태에서 회귀 게이트를 116으로 바꾸는 것은
근거가 없다. 확인 결과 12pt `한양신명조`를 전역 일반 반각으로 바꾸면 split 뒤에는 197쪽을
맞추지만, 원본의 권위 저장 LineSeg까지 재계산하는 경로에서 116쪽을 만들었다.

해결은 두 규칙을 섞지 않는 것이다. 기본 text metric은 원래의 411/1024em(원본 115쪽)을
유지한다. `reflow_stale_cells_after_split`가 부르는 전용 tokenization에서만, 정확히 12pt
`한양신명조` 공백을 512/1024em으로 측정한다. 그 helper는 장평·자간·낱말 간격도 일반
측정과 같은 식으로 적용하므로, 단순 상수 치환이 아닌 동일한 스타일 연산의 base advance만
바꾼다. 이 경계가 한컴 2020의 "폭 변경 뒤 저장" 동작과 원본 저장본을 동시에 보존한다.
