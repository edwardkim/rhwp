---
kind: investigation
status: active
canonical: mydocs/plans/task_m100_4967_v2.md
last_verified: 2026-08-25
---

# Issue #4967 — W8 font face 교정 qualification

이 디렉터리는 W8 tracker의 첫 process canary인 rank 8 `KoPubWorld바탕체 Light`의 교정 적격성 증거를
보존한다. rank 8 일괄 exact metric 후보는 Stage W8-Q5에서 `no-change`로 종결됐으며 제품 font mapping은
변경하지 않는다. #4967 tracker는 rank 1·7과 evidence-reopen lane 때문에 계속 active다. 최종 판정은
[`task_m100_4967_report.md`](../../../report/task_m100_4967_report.md)에 있다.

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
[`task_m100_4967_w8_q3r.md`](../../../working/task_m100_4967_w8_q3r.md)에 있다.

Q5 승인 뒤 push-preflight에서 process-global page-tree counter가 generated suite의 병렬 build를 함께 세는
검증 격리 결함을 발견했다. 전용 성능 가드의 전역 counter를 보존하면서 #4967 계약만 current-thread
counter로 격리했다. fresh generated suite의 기본 병렬·직렬 검증과 Docker WASM이 통과했으며, query JSON과
qualification 결과는 바꾸지 않았다.
