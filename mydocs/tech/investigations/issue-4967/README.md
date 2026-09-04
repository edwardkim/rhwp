---
kind: investigation
status: active
canonical: mydocs/plans/archives/task_m100_4967_v3.md
last_verified: 2026-08-26
---

# Issue #4967 — W8 font face 교정 qualification

이 디렉터리는 W8 tracker의 face별 교정 적격성 증거를 보존한다. rank 8 `KoPubWorld바탕체 Light`의
일괄 exact metric 후보와 rank 1 `문체부 바탕체` name-relation 후보는 각각 Stage W8-Q5와 W8-R1-Q5에서
`no-change`로 종결됐으며 제품 font mapping은 변경하지 않는다. rank 7 `KoPubWorld돋움체 Light`도
Stage W8-R7-Q5에서 `no-change`로 종결됐다. qualification 가능한 rank 1·7·8이 모두 완료되고 나머지 14개는
명시적 evidence 변화 전까지 terminal이므로 현재 W8 queue는 비어 있다. 완료된 face의 최종 판정은
[`task_m100_4967_report.md`](../../../report/archives/task_m100_4967_report.md)에 있다.

## Stage W8-R7-Q0 증거 호환성과 bounded cohort

rank 7은 같은 KoPubWorld family인 rank 8의 metric 결과를 재사용하지 않고 W3·W4·W5·W7.5 증거를 독립
대사한다. 재현 도구는 `scripts/font_rank7_qualification.py`, 계약 테스트는
`scripts/tests/test_font_rank7_qualification.py`다.

- local-only 원장: `output/4967/w8-r7-q0/rank7_private_cohort.json`, mode `0600`
- 공개 baseline: [`rank7_qualification_baseline.json`](rank7_qualification_baseline.json), mode `0644`
- 10k corpus 재parse·Hyper-V Oracle 재실행·제품 source 변경: 0

기존 journal의 rank 7 cohort는 5문서(HWP 3, HWPX 2), target 63,858자다. W4 위험 63,732자와
category·format·compressed fixed-context 수치가 일치했고 위험량 전부가 stored lane이다.

| 판정축 | 문자 수 |
| --- | ---: |
| table-cell / body / header / footer | 52,149 / 11,504 / 175 / 30 |
| compressed 전체 / 위험 | 63,082 / 62,960 |
| ratio 95·spacing -9 | 51,226 |
| bold / italic | 4,468 / 0 |

exact local TTF는 W5 SHA-256과 일치하고 `KoPubWorld돋움체 Light`, `KoPubWorldDotum Light`,
`KoPubWorldDotumLight`가 같은 SFNT에 있다. 반면 현행 registry와 W7 projection에는 Canvas2D WOFF2·
CanvasKit OTF supply rule만 있고 Rust layout-name·layout-metric rule은 없다. supply URL이 존재한다는 사실을
metric identity로 승격하지 않는다. Stage W8-R7-Q1은 W5 hash-sealed fixture를 복원해 HWP/HWPX의 current
runtime boundary와 native·WASM parity를 먼저 판정한다.

## Stage W8-R7-Q1 공개 fixture와 current runtime boundary

W5 generator로 rank 7 HWPX fixture를 재생성해 봉인 SHA-256
`1cc8062c6fd0da39cfddc4182115226717516d4250e693b43596293374236f9e`와 byte-exact함을 확인했다. 이
HWPX를 현재 HEAD의 `rhwp convert --verify --verify-pages`로 두 번 변환한 HWP5는 SHA-256
`3a844e0530ecede89301ab1f3c2381865412f8472aa08733cdb9d1d25223ee7f`, IR 차이 없음, 1쪽 동치를
유지했다.

재현 도구는 `scripts/font_rank7_runtime_boundary.mjs`, 계약 테스트는
`scripts/tests/font_rank7_runtime_boundary.test.mjs`다. 공개 정본은
[`rank7_runtime_boundary.json`](rank7_runtime_boundary.json), 두 형식의 계보는
[`rank7_runtime_boundary.manifest.json`](fixtures/rank7_runtime_boundary.manifest.json)에 있다.

- HWPX·HWP5는 각 1,556건이며 형식별 native·Docker WASM trace가 byte-exact하다.
- requested·normalized·metric alias face는 전건 `KoPubWorld돋움체 Light`다.
- metric entry는 전건 `null`, match kind는 전건 `none`이고 heuristic 폭 분포는 390 / 1,114 / 52다.
- HWPX는 `substFont=KoPubWorld바탕체 Light`를 document·paint 후보에 보존하지만 HWP5는 보존하지 않는다.
- 이 metadata 차이에도 두 형식의 source+layoutMetric projection, 실제 layout run geometry와 fixed-frame
  6축은 byte-equivalent하다.
- 따라서 document substitution은 current layout metric에 영향을 주지 않으며 paint 후보 체인만 바꾼다.

current layout의 첫 divergence는 `layout-metric`이다. Stage W8-R7-Q2는 제품 source를 바꾸지 않고 exact
TTF와 CDN OTF·WOFF2의 `hmtx` identity, current heuristic 대비 누적 advance와 첫 frame crossing을 제한
비교한다. HWPX의 substitution을 layout fallback으로 승격하지 않는다.

## Stage W8-R7-Q2 exact metric 제한 비교

재현 도구는 `scripts/font_rank7_metric_hypothesis.py`, 계약 테스트는
`scripts/tests/test_font_rank7_metric_hypothesis.py`다. 공개 정본은
[`rank7_metric_hypothesis.json`](rank7_metric_hypothesis.json)이다. CDN bytes는 local-only cache에만 두고
tracked 결과에는 hash·size·SFNT aggregate만 기록한다.

- exact TTF와 CDN OTF·WOFF2는 bytes·name·outline identity가 아니지만 공통 cmap 25,973자의 advance
  mismatch가 0이고 fixture 53자도 metric-compatible하다.
- current transform 1,556건을 mismatch 0으로 재생한 뒤 exact base를 적용하면 847,977 → 807,233
  HWPUNIT로 40,744 감소한다. narrower 778, wider 726, equal 52라 평균만으로 판정하지 않는다.
- actual fixed-frame 6축에서는 crossing 앞당김·신규 발생 0, 지연 3, 제거 1, 불변 2다.
- Q0 style domain 63,858자의 ratio·spacing 축은 모두 modelled지만 aggregate에는 codepoint 분포가 없어
  weighted delta를 주장하지 않는다.
- bold 노출 4,468자는 regular metric + synthetic bold가 advance를 바꾸지 않는 source 불변식을 확인했으나,
  공개 fixture에 bold record가 없으므로 Q3에서 동적으로 재확인한다.

Q2 판정은 `qualified-for-q3`이며 target은 `layout-metric` 한 plane이다. font·paint identity, 배포 권한,
제품 registry·metric DB·fallback·supply 변경은 승인되지 않았다. Q3는 Q0에서 동결한 5문서만 대상으로
same-snapshot actual geometry와 stored-row admission을 판정한다.

## Stage W8-R7-Q3 bounded same-snapshot qualification

재현 도구는 `scripts/font_rank7_private_qualification.py`, 공용 projector는
`scripts/font_rank8_private_qualification.py`, 계약 테스트는
`scripts/tests/test_font_rank7_private_qualification.py`다. 공개 정본은
[`rank7_private_qualification.json`](rank7_private_qualification.json), 상세 결과는 local-only
`output/4967/w8-r7-q3/` mode `0600`이다.

- Q0 동결 5문서만 읽었고 source usage 63,858자와 render observation 74,969자를 별도 회계로 유지했다.
- exact metric 74,132자 적용, current transform mismatch 0, trace truncated page 0이다.
- overflow 제거·감소 301 line과 함께 신규·증가 171 line이 관찰됐으며 모두 table-cell이다.
- modelled 51건은 모두 HWPX admitted stored-row의 동일 signature다. current 0px → candidate 0.707px,
  line advance +162 HWPUNIT로 신규 overflow가 생겼다.
- bold render 1,269자에는 modelled regression이 없지만 nested style 63,465자가 미조인이라 Q0 bold
  4,468자 전체의 dynamic completion은 주장하지 않는다.
- cache-unmodelled 55,461자도 제품 이득·회귀 근거로 승격하지 않는다.

결정적 modelled regression은 open evidence gap보다 우선한다. 따라서 Q3 판정은 `no-change`이고 Q4의
backend·portable·시각 정책에는 진입하지 않는다. 제품 registry·metric DB·fallback·paint·supply 변경도
없다. 다음 gate는 Q5 최종 disposition과 #4967 tracker evidence-reopen lane 감사다.

## Stage W8-R7-Q5 최종 disposition과 tracker 감사

rank 7은 `no-change`로 최종 동결한다. qualified 전용 product-correction 자식 이슈·registry operation·
acceptance matrix는 만들지 않는다. rank 1·7·8은 모두 종결됐고 나머지 rank는 외부 evidence 변화가 실제로
발생할 때만 재개한다.

GitHub read-only 감사에서 #4967은 OPEN이지만 실제 sub-issue가 0개이고 #4960의 W8 checkbox도 미완료였다.
현재 변경 병합 뒤 #4967을 #4960 sub-issue로 연결하고 W8을 완료 표시한 다음, 최종 comment·reopen 조건을
남겨 #4967을 completed close하는 것을 권고한다. 새 source·provider·identity·capability evidence가 생기면
tracker를 reopen하거나 face별 새 이슈를 등록한다. 이번 Q5의 GitHub mutation은 0이다.

## Stage W8-R1-Q0 경계

rank 1 `문체부 바탕체`는 rank 8 결론을 재사용하지 않고 기존 W3·W4·W5·W7.5 증거를 독립 대사한다.
재현 도구는 `scripts/font_rank1_qualification.py`, 계약 테스트는
`scripts/tests/test_font_rank1_qualification.py`다.

- local-only 원장: `output/4967/w8-r1-q0/rank1_private_cohort.json`, mode `0600`
- 공개 baseline: [`rank1_qualification_baseline.json`](rank1_qualification_baseline.json)
- source 경계: [`rank1_source_provenance_attestation.json`](rank1_source_provenance_attestation.json)
- 10k corpus 재parse·Hyper-V Oracle 재실행·제품 source 변경: 0

기존 journal의 rank 1 cohort는 22문서(HWP 15, HWPX 7), target 209,066자다. W4 위험 208,986자와
category·format·compressed fixed-context 수치가 일치했고 위험량 전부가 stored lane이다. exact local SFNT는
`문체부 바탕체`와 `MBatang` family name을 함께 가지며 현행 metric projection에도 `MBatang` entry 370이
있다. 반면 v2 registry의 두 이름에 대한 explicit rule은 없다. 이 차이가 제품 miss인지 W4 관찰 경계인지
Stage W8-R1-Q1에서 runtime trace로 판정한다.

## Stage W8-R1-Q1 runtime 관찰 경계

공개 HWPX fixture와 그 fixture를 `rhwp convert --verify --verify-pages`로 결정적으로 변환한 HWP5 fixture를
같은 Font Decision Trace에 넣었다. 변환 계보와 digest는
[`rank1_runtime_boundary.manifest.json`](fixtures/rank1_runtime_boundary.manifest.json), HWP fixture는
[`rank1_runtime_boundary.hwp`](fixtures/rank1_runtime_boundary.hwp)에 고정했다. font bytes와 private corpus
identity는 포함하지 않는다.

재현 도구는 `scripts/font_rank1_runtime_boundary.mjs`, 계약 테스트는
`scripts/tests/font_rank1_runtime_boundary.test.mjs`다. 공개 정본은
[`rank1_runtime_boundary.json`](rank1_runtime_boundary.json)이다.

- HWPX와 HWP5 모두 target 1,556건이며 runtime decision semantics가 같다.
- 두 형식 모두 requested·normalized·alias-resolved face가 `문체부 바탕체`, layout-name step은 0이다.
- metric entry는 전건 `null`, match kind는 전건 `none`이고 heuristic width 분포도 같다.
- 각 형식의 native·현행 WASM canonical trace는 byte-exact하며 형식 간 trace digest도 같다.
- W4 face miss는 raw-name 계측의 오탐이 아니라 runtime에서도 재현되는 실제 unresolved 경계다.
- 첫 divergence는 기존 `MBatang` metric anchor 전의 `layout-name` plane이다.

Q1 disposition은 `qualified-for-q2-layout-name-hypothesis`다. Q2는 제품 규칙을 바꾸지 않고 가상
`문체부 바탕체 -> MBatang` relation에서 현행 generated metric과 exact `MT.TTF hmtx`만 제한 비교한다.
paint identity·font supply 또는 제품 변경은 아직 qualification하지 않는다.

## Stage W8-R1-Q2 exact metric 제한 비교

`scripts/font_rank1_metric_hypothesis.py`는 현행 generated `MBatang` entry 370, exact local `MT.TTF hmtx`,
Q1의 current trace를 함께 읽는다. 제품 registry·metric DB·fallback을 바꾸지 않고 가상
`문체부 바탕체 -> MBatang` relation만 적용한다. 계약 테스트는
`scripts/tests/test_font_rank1_metric_hypothesis.py`, 공개 정본은
[`rank1_metric_hypothesis.json`](rank1_metric_hypothesis.json)이다.

- 현행 entry는 모든 Hangul 11,172자에 1,000, space에 500을 제공하고 그 밖에는 기존 heuristic을 보존한다.
- exact source의 layout-bearing cmap은 Hangul 2,350자와 space 1자이며 현행 entry와 advance mismatch가 0이다.
- generated entry가 exact cmap 밖 Hangul 8,822자까지 폭을 제공하므로 font identity나 exact source 계보는
  주장하지 않는다.
- 공개 fixture 1,556건에서 current→virtual relation과 virtual→exact advance delta가 모두 0이다.
- 장평·자간·justification transform 13축과 fixed-frame 6축의 total advance·첫 crossing도 모두 불변이다.
- 전체 layout-bearing domain의 base advance가 동치이므로 Q0의 private 22문서를 다시 parse하지 않았다.
- Q0 aggregate의 bold 38,090자는 regular metric을 쓰는 metadata-only `boldFallback` 경로이며 layout
  advance가 불변이다. projector는 bold·italic 4개 style 조합도 닫는다.

rank 1 disposition은 `no-change`다. runtime name miss는 실제지만 이를 `MBatang`으로 연결해도 layout 이득이
없고 portable supply·paint identity도 qualification되지 않았다. 제품 alias·metric·fallback 변경과
W8-R1-Q3에는 진행하지 않는다.

2026-08-26 `upstream/devel@ee7e8a6ed`와 PR 생성 직전 `upstream/devel@6240d255b`를 차례로 통합할 때마다
새 네이티브·Docker WASM 산출물을 입력으로 Q1·Q2를 다시 실행했다. 최신 Q1은 각 형식 1,556건의
native/WASM byte parity와 `layout-name` 경계를 유지했고, Q2는 두 advance delta와 fixed-frame crossing을
모두 0으로 유지했다. 갱신된 canonical hash는 각 JSON 정본에 기록했으며 제품 판정은 계속 `no-change`다.

## Stage W8-R1-Q5 최종 disposition

rank 1은 `no-change`로 동결한다. W4 face-miss는 Q1 runtime에서도 재현됐으므로 계측 lineage 오탐이 아니다.
반면 Q2는 가상 name relation과 exact metric이 전체 layout-bearing domain에서 현행 advance와 동치임을
증명했다. 수정할 layout delta가 없으므로 qualified 전용 product-correction 자식 이슈와 registry operation을
만들지 않는다. Q3·Q4는 후보 부재로 미진입한다.

rank 1 Q5 시점에는 rank 7과 evidence-reopen lane이 남아 있어 #4967을 OPEN으로 유지했다. 이후 rank 7도
완료됐으며 최종 tracker 운영 판정은 W8-R7-Q5 절을 따른다. rank 1을 다시 열려면 layout 이득을 증명하는
새 evidence 또는 현재와 다른 하나의 decision plane 가설이 필요하다.

공식 문체부 자료는 문화체육관광부 바탕체의 자유 이용·유료 판매 금지·출처 표시 조건을 설명하지만, 해당
자료에서 local `MT.TTF`와 byte-exact한 공식 배포 artifact를 확인하지 못했다. local SFNT의
`OS/2.fsType=2`도 restricted-license embedding을 선언한다. 따라서 Q0은 metric 계측 입력과 font bytes
공급 권한을 분리하고 portable supply를 `blocked`로 유지한다.

## Stage W8-Q0 경계

- private W3 journal을 다시 parse하거나 10k corpus를 재실행하지 않는다.
- rank 8을 실제로 사용한 문서의 경로·이름·본문·hash는 owner-only local output에만 둔다.
- tracked baseline에는 aggregate, evidence digest와 privacy gate만 남긴다.
- W5 exact/subst/missing Hyper-V ladder는 재사용하며 이번 단계에서 VM을 실행하지 않는다.
- v2 registry와 다섯 runtime projection은 읽기 전용이다.

재현 도구는 `scripts/font_rank8_qualification.py`, 계약 테스트는
`scripts/tests/test_font_rank8_qualification.py`다. local-only 입력이 있는 메인테이너 환경에서 projector를
실행하면 `rank8_private_cohort.json`은 mode `0600`, 공개 baseline은 mode `0644`로 생성된다.

## Stage W8-Q1 current 기준선

W5에서 봉인된 generator로 rank 8 fixture를 재현했다. fixture의 생산 계보는 #4963, qualification 소비
목적은 #4967이며 bytes SHA-256은 W5 ladder와 같은
`f6edc8fc43dfd3256385e9752979c14a7041e50c06d36be47cef6e3486835084`다. font bytes는 포함하지 않는다.

`scripts/font_rank8_trace_baseline.mjs`는 다음을 fail-closed로 고정한다.

- current release native와 Docker WASM의 1,556-record trace canonical byte parity
- `maxCharacters=4096`의 전건 완료와 누락 0
- layout metric entry·match kind·heuristic width source 분포
- 표 셀 28,980 HWPUNIT, 글상자 29,434 HWPUNIT의 실제 content width와 대표 장평·자간별 frame slack
- absolute path·private corpus identity·font bytes·full trace의 tracked output 유입 0

결과 정본은 `rank8_current_trace_baseline.json`이다. Q1에서는 현행 상태만 측정하며 metric DB, fallback,
paint·supply 규칙은 바꾸지 않는다. Canvas2D·CanvasKit actual paint는 trace만으로 관찰할 수 없으므로
`studioSnapshotRequired` 상태를 그대로 보존한다.

## Stage W8-Q2 exact metric 가설

`scripts/font_rank8_metric_hypothesis.py`는 외부 font root의 W5 exact TTF와 현행 registry가 가리키는
`font-kopubworld@1.0.3` OTF·WOFF2를 읽어 다음 경계를 분리한다.

- TTF와 CDN OTF·WOFF2는 bytes·name table·outline identity가 아니다.
- 세 source는 fixture 53개 codepoint의 advance가 같고, TTF와 CDN source의 전체 공통 cmap 25,970개도
  advance mismatch가 0이다.
- CDN OTF와 WOFF2는 26,089개 cmap advance 및 fixture outline digest가 서로 같다.
- current trace의 ratio → letter spacing → justification transform을 1,556건 모두 재생한 뒤 base advance만
  exact `hmtx`로 바꾼다.
- fixed-frame 대표 6축에서 metric capacity crossing의 앞당김·신규 발생은 0이다.

결과 정본은 `rank8_metric_hypothesis.json`이다. Q2는 `layout-metric` 하나만 Q3 검증 대상으로
qualification하며 font identity·paint identity, 배포 권한 또는 제품 변경을 승인하지 않는다.

## Stage W8-Q3 bounded private qualification

`scripts/font_rank8_private_qualification.py`는 Q0에서 동결한 6문서만 읽고, current Font Decision Trace의
transform을 px 정밀도로 재생한 뒤 exact TTF advance만 대입한다. 제품 metric DB·registry·paint·supply는
바꾸지 않는다. render-tree 원문은 문서별 임시 디렉터리에서만 사용하고 자동 폐기하며, 상세 좌표·경로·문서
hash는 mode `0600` local-only 결과에만 둔다.

공개 결과 `rank8_private_qualification.json`은 다음 경계를 고정한다.

- W3 source usage 43,432자와 페이지 render observation 44,117자는 반복 story 때문에 다른 회계다.
- current transform 0 mismatch, exact metric 43,735자 적용, cmap miss 213자와 특수 advance 169자는 보존했다.
- frame을 조인한 문서는 개선 3·악화 1, 독립 query 간 run set이 달라진 2문서 4,397자는 `unmodelled`다.
- 한 표 셀 same-partition projection에서 +144 HWPUNIT, 1.92px overflow가 새로 발생했다.
- current query는 stored-row cache admission을 노출하지 않으므로 LineSeg validity를 주장하지 않는다.

결과는 `blocked`이며 W8-Q4로 진행하지 않는다. 재개 조건은 같은 composition snapshot에서 TextLine
frame·context와 stored-row admission을 함께 제공하는 읽기 전용 evidence query다. 이 query 보강과 제품
변경은 현재 qualification의 승인 범위가 아니다.

## Stage W8-Q3R same-snapshot evidence query

승인된 보강은 쪽당 하나의 `PageRenderTree`를 Font Decision Trace, TextLine frame/context와 stored-row cache
disposition이 함께 소비하게 한다. 기존 trace API와 hash는 유지하고 새 결합 query만 추가한다. frame
provenance가 불완전한 문단은 `unmodelled`로 남기며, query 결과로 제품 metric·registry·fallback·paint·supply를
변경하지 않는다. 구현 검증 뒤에도 Q0의 동일 6문서만 다시 판정한다.

구현 뒤 동일 6문서를 재판정해 trace↔line 미조인은 0이 됐다. target 44,117자의 cache disposition은
`admitted` 15,788, `rejected` 10,187, `unmodelled` 18,142였다. modelled 회귀 5줄 중 admitted가 4줄이며,
표 셀 +1.92px 신규 overflow도 admitted였다. 따라서 rank 8 일괄 exact metric 후보는 `no-change`이고 Q4와
제품 변경으로 진행하지 않는다. 정본 수치는 갱신된 `rank8_private_qualification.json`, 과정과 판정 논리는
[`task_m100_4967_w8_q3r.md`](../../../working/archives/task_m100_4967_w8_q3r.md)에 있다.

Q5 승인 뒤 push-preflight에서 process-global page-tree counter가 generated suite의 병렬 build를 함께 세는
검증 격리 결함을 발견했다. 전용 성능 가드의 전역 counter를 보존하면서 #4967 계약만 current-thread
counter로 격리했다. fresh generated suite의 기본 병렬·직렬 검증과 Docker WASM이 통과했으며, query JSON과
qualification 결과는 바꾸지 않았다.
