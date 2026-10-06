---
kind: report
status: final
canonical: mydocs/report/archives/task_m100_4967_report.md
last_verified: 2026-08-26
---

# Task M100 #4967 최종 보고서 — W8 rank 8·rank 1·rank 7 face 교정 qualification

## 1. 최종 판정

rank 8 `KoPubWorld바탕체 Light`에 exact `hmtx` advance를 일괄 적용하는 `layout-metric` 교정 후보는
**`no-change`**다. exact metric은 일부 실제 줄의 overflow를 제거하거나 slack을 늘렸지만, 현행 stored-row
cache policy가 실제로 수용한 줄에서도 신규 overflow를 포함한 회귀가 확인됐다. 평균 개선량으로 fixed-frame
회귀를 상쇄하지 않는 보호 불변식에 따라 제품 metric DB·registry·fallback·paint·supply는 변경하지 않는다.

rank 1 `문체부 바탕체`를 `MBatang`에 연결하는 `layout-name` 후보도 **`no-change`**다. localized name의
runtime miss는 실제지만, 가상 relation과 exact `MT.TTF hmtx`가 전체 layout-bearing 문자 영역에서 현행
advance와 동치여서 layout 이득이 없다. 이름 match metadata만 만들고 조판을 바꾸지 않는 제품 rule은
추가하지 않는다.

rank 7 `KoPubWorld돋움체 Light`의 일괄 exact `layout-metric` 후보도 **`no-change`**다. 공개 fixture에서는
fixed-frame 비회귀 가능성이 있었지만 동결된 5문서의 same-snapshot 판정에서 HWPX table-cell admitted
stored-row에 current 0px → candidate 0.707px 신규 overflow가 생기는 한 modelled boundary signature를
확인했다. 평균 폭 감소와 다른 line의 개선으로 이 회귀를 상쇄하지 않는다.

세 qualification 가능한 rank 1·7·8은 모두 종결됐다. 나머지 14개 rank는 W5에서 source unavailable,
protected partial 또는 capability mismatch의 terminal disposition과 명시적 재개 조건을 받았다. 빈
evidence 대기열을 영구 OPEN 작업으로 유지하지 않고, 현재 W8 scope는 완료로 닫은 뒤 새 evidence가 실제로
생기면 #4967을 reopen하는 운영을 권고한다. 실제 close·#4960 checklist·sub-issue 관계 변경은 현재 code
candidate 병합 뒤 별도 승인으로 수행한다.

## 2. 판정 계보

| Stage | 질문 | 결과 |
| --- | --- | --- |
| W8-Q0 | 기존 W3·W4·W5·W7.5 증거를 재사용할 수 있는가 | 기존 journal에서 실제 6문서 cohort를 재선정, 10k 전수 재계측 0 |
| W8-Q1 | current native·WASM이 동일한 현행 heuristic을 관찰하는가 | 1,556/1,556 trace canonical parity, metric entry 0 |
| W8-Q2 | exact metric 후보가 한 decision plane으로 제한되는가 | TTF·OTF·WOFF2 공통 cmap advance mismatch 0, `layout-metric`만 Q3 진입 |
| W8-Q3 | 실제 6문서에서 개선과 비악화를 증명하는가 | query snapshot 불일치로 4,397자 미조인, `blocked` |
| W8-Q3R | 같은 snapshot에서 frame·run·cache disposition을 판정할 수 있는가 | 미조인 0, modelled 회귀 5줄, 최종 `no-change` |
| W8-Q4 | backend·portable 적용 정책을 설계할 것인가 | 후보가 Q3R에서 기각돼 불필요, 미진입 |
| W8-Q5 | 제품 교정 후속 이슈를 제안할 것인가 | qualified가 아니므로 자식 이슈 초안·등록 0 |

단계별 local commit은 `f5710a4c4`, `f5eeec366`, `bc25152cc`, `90768be62`, `0c2ff0068`,
`9983ae66d`에 고정했다. push-preflight 계수기 격리까지 포함한 code candidate는 `a637c9c8e`이며
PR #6069로 제출했다.

## 3. 결정적 실사용 증거

Q3R은 Q0에서 동결한 동일 6문서·534쪽만 다시 읽었다. 쪽마다 하나의 `PageRenderTree`를 Font Decision
Trace, `TextLine` frame/context, exact run-index membership와 stored-row cache disposition이 함께 소비했다.

| 항목 | 결과 |
| --- | ---: |
| source usage | 43,432자 |
| page render observation | 44,117자 |
| target line | 2,043줄 |
| trace↔line 미조인 | 0자 |
| cache `admitted` | 15,788자 / 706줄 |
| cache `rejected` | 10,187자 / 316줄 |
| cache `unmodelled` | 18,142자 / 1,021줄 |
| overflow 제거 | 217줄 |
| slack 증가 | 114줄 |
| 불변 | 1,707줄 |
| overflow 증가 | 3줄 |
| overflow 신규 | 2줄 |

회귀 5줄 중 4줄은 현행 cache key가 physical-row geometry를 재현한 `admitted`, 1줄은 `rejected`였다.
특히 표 셀의 target 9자가 exact metric에서 `+144 HWPUNIT`, 즉 `+1.92px` 넓어져 새 overflow를 만든 사례는
`admitted`였다. 따라서 `unmodelled` 18,142자를 더 해석하더라도 이미 증명된 modelled 회귀를 없앨 수 없다.

개선 331줄과 회귀 5줄을 단순 개수나 평균으로 상쇄하지 않는다. HWP 사용자는 장평·자간과 고정 표·글상자에
문자열을 맞추므로, 소수의 경계 crossing도 문서 조판을 바꾸는 실제 회귀다.

## 4. correction hypothesis 판정

exact TTF와 현행 CDN OTF·WOFF2의 advance compatibility는 성립했다. 이 결과는 exact metric을 안전하게
계측하는 근거이며 bytes·name·outline 또는 paint identity가 같다는 뜻은 아니다. Q2가 `layout-metric` 한
decision plane만 후보로 제한한 것도 이 경계를 지키기 위해서였다.

실사용 판정은 같은 face의 exact advance가 현행 heuristic보다 항상 안전한 방향으로 작용하지 않음을 보였다.
장평·자간·justification·fixed frame이 결합되면 문단별 여유 폭에 따라 crossing이 제거되기도 하고 새로
생기기도 한다. 따라서 다음 일반화는 채택하지 않는다.

```text
KoPubWorld바탕체 Light 사용
  -> exact hmtx가 존재
  -> 모든 layout에서 heuristic을 exact metric으로 교체
```

향후 재개하려면 일괄 face rule이 아니라 회귀를 만들지 않는 더 좁은 feature-detected cohort와 독립적인
correction hypothesis가 필요하다. 현재 증거만으로 `admitted`와 `rejected` 어느 한쪽에 exact metric을
자동 적용하는 정책도 승인하지 않는다.

## 5. Q4 미진입과 후속 이슈 없음

W8-Q4의 목적은 통과한 후보를 native·WASM·Canvas2D·CanvasKit online/offline에 어떻게 적용할지 정하고
FI-01~FI-14를 전항 판정하는 것이다. Q3R에서 제품 후보가 먼저 기각됐으므로 적용 정책을 설계할 대상이 없다.
Q4를 실행하는 것은 기각된 변경을 전제로 backend 정책과 시각 검증을 수행하는 불필요한 비용이며,
qualification 결과를 뒤집지도 못한다.

계획상 product-correction 자식 이슈 초안은 `qualified`일 때만 작성한다. 최종 상태가 `no-change`이므로
registry operation, acceptance matrix 또는 구현 계획을 담은 자식 이슈를 만들지 않는다. 향후 새 증거로
rank 8을 재개할 때는 #4967의 evidence-reopen lane에서 별도 계획과 승인을 먼저 받는다.

## 6. 보호 불변식과 변경 범위

| 불변식 | 결과 |
| --- | --- |
| 한 번에 한 decision plane만 판정 | 충족: `layout-metric`만 후보화 |
| layout metric과 paint/source identity 분리 | 충족 |
| stored LineSeg 존재와 validity 분리 | 충족: `admitted`·`rejected`·`unmodelled` 기능 탐지 |
| actual fixed geometry를 평균 폭으로 대체하지 않음 | 충족: line/frame crossing 판정 |
| private corpus identity·본문·hash 비공개 | 충족 |
| font bytes 비공개 | 충족 |
| 10k 전수·Hyper-V 불필요 재실행 금지 | 충족: 모두 0 |
| 제품 font 규칙·render 결과 변경 금지 | 충족: mutation 0 |

Q3R의 same-snapshot query와 공개 계약 테스트는 qualification 증거의 정합성을 보강한다. 이것은 제품 font
정책 변경이 아니며 기존 standalone Font Decision Trace의 출력·상한·hash를 유지한다.

## 7. 검증 결과

- 공개 same-snapshot 계약: current-thread page tree build 쪽당 1, trace↔line membership 전건 1:1
- cache disposition 공개 fixture: `admitted`·`rejected`·`unmodelled` 경계 통과
- 기존 Font Decision Trace 회귀: 불변
- integration regression suite 009: 140/140
- private projector 계약: 17/17
- 새 CLI Clippy: warning 0
- CLI `maxCharacters=4097` 거부, 상한 4,096 유지
- 공개 JSON privacy·canonical hash 검사 통과
- Markdown 링크 검사 통과
- `cargo fmt --all` 및 `cargo fmt --all -- --check` 통과

### 7.1 push-preflight 검증 정정

보고서 승인 뒤 기본 병렬 `regression_suite_009`를 재실행했을 때 최초 결과는 139/140이었다. 실패한
same-snapshot test는 process-global `PAGE_TREE_BUILDS`를 reset한 뒤 읽는 동안 다른 병렬 테스트의 build를
함께 세어 기대 1, 관찰 4가 됐다. 해당 테스트 단독 실행과 140건 직렬 실행은 모두 통과해 제품 query의
다중 build 회귀와 구분했다.

기존 전역 성능 counter와 전용 프로세스 테스트는 유지하고 generated suite용 current-thread counter를
추가했다. 정정된 테스트는 별도 스레드의 실제 build가 현재 스레드 count를 오염하지 않는지 결정적으로
검증한다. 이는 진단 계수기의 측정 범위 정정이며 query JSON·layout·font 판정과 본 보고서의 `no-change`
결론에는 영향이 없다.

fresh `--prepare` 뒤 #4967 원본이 포함된 `regression_suite_008`은 기본 병렬 3회 420/420, 직렬
140/140을 통과했다. 기존 process-global 성능 가드 focused 4/4, Python 17/17, Clippy `-D warnings`,
manifest·Markdown 링크·fmt·diff 검사와 Docker WASM 5분 56초도 통과했다. WASM `pkg` 추적 delta는 0이다.

공개 정본 결과는
[`rank8_private_qualification.json`](../../tech/investigations/issue-4967/rank8_private_qualification.json),
구현·재판정 과정은
[`task_m100_4967_w8_q3r.md`](../../working/archives/task_m100_4967_w8_q3r.md)에 있다.

## 8. 완료와 후속 절차

W8 rank 8 qualification은 메인테이너의 최종 보고서 승인으로 `no-change` 완료됐고, code candidate는
PR #6069로 제출됐다. self-review 기록, 최신 trailing head 검증과 merge는 각각 별도 gate로 유지한다.

#4967 tracker의 rank 7 작업은 section 10에서 독립 판정했다. rank 8·rank 1 결과를 같은 family·다른
localized face에 추정 적용하지 않았으며 세 face 모두 서로 다른 원인과 증거로 `no-change`에 도달했다.

## 9. rank 1 `문체부 바탕체` 최종 disposition

### 9.1 판정 계보

| Stage | 질문 | 결과 |
| --- | --- | --- |
| W8-R1-Q0 | 기존 W3·W4·W5·W7.5 증거와 exact source를 재사용할 수 있는가 | 22문서·위험 208,986자 재선정, exact name pair·`MBatang` entry 370 확인 |
| W8-R1-Q1 | W4 face miss가 계측 projection 오탐인가 | HWP/HWPX 각 1,556건에서 runtime도 unresolved, 첫 divergence `layout-name` |
| W8-R1-Q2 | 가상 name relation과 exact metric이 layout 이득을 만드는가 | 전체 layout-bearing domain과 fixed-frame 6축에서 advance·crossing delta 0, `no-change` |
| W8-R1-Q3 | private cohort actual reflow를 검증할 것인가 | candidate delta가 없어 불필요, 미진입 |
| W8-R1-Q4 | backend·portable 적용 정책을 설계할 것인가 | 제품 후보가 없어 불필요, 미진입 |
| W8-R1-Q5 | product-correction 후속 이슈를 제안할 것인가 | qualified가 아니므로 자식 이슈 초안·등록 0 |

단계별 local commit은 `857241ac1`, `9fc8110d3`, `e70e35ec3`에 고정했다. 제품 source는 변경하지 않았다.
증거·projector·계약 테스트를 포함한 최종 code candidate `b83d9d08e`는 PR #6081로 제출했다.

### 9.2 runtime miss와 correction 이득의 분리

Q1에서 raw·normalized·metric alias-resolved face는 전건 `문체부 바탕체`, layout-name step은 0,
metric entry는 `null`이었다. 따라서 W4의 face-miss 208,949자와 heuristic 37자는 runtime 경계와 같은
현상을 센 것이며 계측 lineage를 정정할 이유가 없다.

그러나 miss의 존재만으로 correction을 승인하지 않는다. Q2의 가상 `문체부 바탕체 -> MBatang` relation은
Hangul을 1,000 HWPUNIT, space를 500 HWPUNIT으로 선택하고 나머지는 현행 heuristic을 보존한다. exact
`MT.TTF`도 보유한 Hangul 2,350자와 space에서 같은 advance다.

| Q2 항목 | 결과 |
| --- | ---: |
| 공개 trace record | 1,556 |
| current transform replay mismatch | 0 |
| current → virtual relation delta | 0 HWPUNIT |
| virtual relation → exact delta | 0 HWPUNIT |
| 장평·자간·justification axis | 13축, 전부 delta 0 |
| fixed-frame | 6축, total advance·첫 crossing 전부 불변 |
| exact layout-bearing cmap mismatch | 0 / 2,351 |

generated entry는 exact cmap 밖 Hangul 8,822자에도 폭을 제공하고 item-level generator input manifest가
남아 있지 않다. 따라서 현행 entry를 exact source-derived라고 부르거나 font·glyph·paint identity를
주장하지 않는다. layout advance compatibility만 성립한다.

최종 audit에서는 문자-domain 외에 style-domain도 다시 확인했다. Q0 기존 aggregate의 target 209,066자 중
bold 요청은 38,090자, italic 요청은 0자다. `MBatang`에는 regular entry만 있어 bold 요청은 `nameFirst`
regular metric과 `boldFallback` metadata를 선택한다. 이 step은 faux bold paint를 나타내지만
`regularMetricAdvance`를 그대로 쓰므로 layout 폭에는 가산 보정이 없다. projector와 계약 테스트는
bold·italic 4개 조합에서 이 no-op을 고정했고 `no-change` 판정은 유지됐다.

### 9.3 Q3·Q4 미진입과 후속 이슈 없음

Q3은 최소 한 실제 문서의 설명 가능한 개선을 확인하는 단계다. Q2에서 전체 layout-bearing domain의 base
advance가 동치이고 같은 transform 뒤 결과도 같으므로 private 22문서를 다시 parse해도 검증할 candidate
delta가 없다. 기존 자료가 충분한데 bounded corpus를 다시 읽는 것은 증거를 늘리지 않으므로 Q3에 진입하지
않았다.

Q4도 살아남은 correction 후보의 backend·portable 정책을 정하는 단계다. exact file은 공식 artifact와
byte match가 확인되지 않았고 `OS/2.fsType=2`라 portable supply가 차단된 상태지만, 그 전에 layout 후보가
`no-change`로 종료됐다. 기각된 후보를 전제로 Canvas2D·CanvasKit supply 정책을 설계하지 않는다.

qualified 전용 product-correction 자식 이슈, registry operation과 acceptance matrix는 작성하지 않는다.
rank 1 evidence-reopen 조건은 현재와 다른 layout 이득 또는 다른 단일 decision plane을 증명하는 새 근거다.

### 9.4 tracker 상태와 보호 불변식

- rank 1 판정 시점의 #4967은 `OPEN`, 담당자 `edwardkim`, milestone `v1.0.0`이었다.
- 당시에는 rank 7 qualification이 남아 있어 이슈를 닫지 않았다. rank 7 완료 뒤의 최종 tracker 판정은
  section 11을 따른다.
- W4 계측은 runtime miss와 일치하므로 정정하지 않는다.
- metric compatibility를 font identity·paint identity·재배포 권한으로 승격하지 않는다.
- private corpus identity·본문·hash와 font bytes를 공개하지 않는다.
- 10k 전수·private bounded cohort·Hyper-V 재실행과 제품 mutation은 모두 0이다.
- Q0의 기존 style aggregate 대사 외에 private 문서 재parse는 없었고 식별 정보도 공개하지 않았다.

rank 1 공개 정본은
[`rank1_qualification_baseline.json`](../../tech/investigations/issue-4967/rank1_qualification_baseline.json),
[`rank1_runtime_boundary.json`](../../tech/investigations/issue-4967/rank1_runtime_boundary.json),
[`rank1_metric_hypothesis.json`](../../tech/investigations/issue-4967/rank1_metric_hypothesis.json)이다.

### 9.5 최신 devel 제출 전 재검증

작업 시작 뒤 `upstream/devel`이 51커밋 전진해 `ee7e8a6ed`가 됐다. merge commit `e10dd2258`은 충돌
없이 양쪽 기록을 보존했지만 `src/renderer/layout/text_measurement.rs`가 Q2의 추적 입력이므로 기존
증적을 그대로 재사용하지 않았다.

PR 생성 직전 devel이 다시 24커밋 전진해 `6240d255b`가 됐다. `mydocs/orders/20260826.md`의 add/add
충돌은 원격 통합 검토 절과 #4967 절을 모두 보존해 merge commit `1252c5e5b`로 해결했다. 이번 전진에는
font 전용 입력 source 변경은 없었지만 renderer·typeset 변경이 포함됐으므로 네이티브·WASM binary를 다시
빌드하고 Q1·Q2를 한 번 더 실행했다.

- 최신 소스 네이티브 `rhwp-q-font-trace` 재빌드 통과
- 표준 Docker WASM 최적화 빌드 통과, 추적 `pkg/` delta 0
- Q1 HWPX·HWP5 각 1,556건, native/WASM byte-exact, 첫 divergence `layout-name` 유지
- Q2 current→virtual relation 0 HWPUNIT, virtual→exact 0 HWPUNIT, 최종 `no-change` 유지
- Node 계약 8/8, Python 계약 12/12 통과
- 최신 Q1 canonical SHA-256:
  `b95e77121f939cfd58440afd941a100e1eb7539bf81b182e7ade708d7ea52720`
- 최신 Q2 canonical SHA-256:
  `d660fc18c1a7f3100c2cf3b1ce9e70f0c893f46ce8e3345c33bab7fb1a2fe23c`

따라서 두 차례 최신 base 변경은 입력 계보를 갱신했지만 rank 1 제품 correction의 이득·위험 판정을
바꾸지 않는다.

### 9.6 PR self-review 시점 최신 base 판정

PR #6081의 code candidate `b83d9d08e`에서 Full CI·CodeQL·Proptest·Adapter inter-diff가 모두 성공한
뒤 `upstream/devel`은 `35c270f47`까지 2커밋 전진했다. 전진 범위는 PR 시각 검증 절차 문서 3개뿐이며,
`git merge-tree --write-tree b83d9d08e upstream/devel`은 충돌 없이 tree
`55c6b3d48cbf2da2dace6cd5ee8ca48063ef334f`를 생성했다. rank 1의 source·test·fixture·판정 입력은
변하지 않았다.

따라서 review 기록만을 위해 base merge·rebase를 수행하지 않는다. self-review와 현재 상태 정정은 녹색
code candidate 뒤의 `mydocs/` 한정 single-parent trailing commit으로 제출하고, push 뒤 review-only
fast-pass의 최신 required aggregate와 `MERGEABLE/CLEAN`을 다시 확인한다.

## 10. rank 7 `KoPubWorld돋움체 Light` 최종 disposition

### 10.1 판정 계보

| Stage | 질문 | 결과 |
| --- | --- | --- |
| W8-R7-Q0 | 기존 W3·W4·W5·W7.5 증거와 cohort를 재사용할 수 있는가 | 5문서·위험 63,732자 재선정, exact TTF·supply-only registry 확인 |
| W8-R7-Q1 | HWP/HWPX current runtime의 첫 divergence는 어디인가 | 두 형식 layout projection 동치, metric entry 전건 `null`, `layout-metric` 확정 |
| W8-R7-Q2 | exact TTF와 CDN source가 metric-compatible하고 공개 fixture가 비회귀인가 | 공통 cmap 25,973자 mismatch 0, fixed-frame 6축 앞당김·신규 crossing 0, Q3 진입 |
| W8-R7-Q3 | 실제 5문서에서 개선과 모든 modelled line 비악화를 증명하는가 | HWPX admitted table-cell에 신규 0.707px overflow, `no-change` |
| W8-R7-Q4 | backend·portable·시각 정책을 설계할 것인가 | 제품 후보가 Q3에서 기각돼 미진입 |
| W8-R7-Q5 | 제품 교정과 tracker 후속을 제안할 것인가 | product-correction 0, tracker 완료·close 제안 |

단계별 local commit은 `331eb2366`, `3a6d97b5d`, `884e71be3`, `b6728a31f`에 고정했고 Q5 문서와 tracker
판정은 `3e705d120`에 고정했다. 이 code candidate는 PR #6106으로 제출됐으며 같은 SHA의 Full CI·CodeQL·
Proptest·Adapter inter-diff가 모두 성공했다. self-review 기록 뒤의 review-only fast-pass와 merge는 별도
gate로 유지한다. 제품 font rule·metric DB·fallback·paint·supply는 변경하지 않았다.

### 10.2 exact source와 runtime 경계

exact TTF와 현행 CDN OTF·WOFF2는 bytes·name·outline identity가 아니지만 공통 cmap 25,973자의 advance가
같다. 이 결과는 layout metric 비교를 허용할 뿐 font·paint identity나 재배포 권한을 승인하지 않는다.

HWPX는 document `substFont=KoPubWorld바탕체 Light`를 paint 후보에 보존하고 HWP5는 보존하지 않는다.
그러나 두 형식의 current source+layoutMetric projection과 실제 layout geometry는 같다. 따라서 substitution
metadata를 layout fallback으로 승격하지 않고 exact metric과 current heuristic만 비교했다.

### 10.3 공개 fixture와 실사용 결과의 차이

공개 fixture 1,556건의 current total 847,977 HWPUNIT는 exact 후보에서 807,233으로 40,744 줄었다.
fixed-frame 6축에서도 crossing 앞당김·신규 발생은 없었다. 하지만 record 726건은 개별적으로 넓어졌으므로
fixture 평균만으로 제품 변경을 승인하지 않고 실제 cohort로 진입했다.

Q3의 source usage 63,858자와 render observation 74,969자는 반복 story 때문에 다른 회계다. exact metric은
74,132자에 적용됐고 current transform replay mismatch는 0이었다. 개선 관찰도 있었지만 regression 171
line은 모두 table-cell이었다. 이 중 판단 가능한 51개 render observation은 다음 한 signature다.

```text
HWPX + table-cell + stored-row admitted
current overflow 0px -> candidate overflow 0.707px
line advance delta +162 HWPUNIT
bold false
```

51을 서로 다른 원본 결함 수로 해석하지 않는다. 동일 signature의 반복이어도 실제 modelled line에 신규
overflow가 발생했다는 사실은 일괄 exact metric 후보를 기각하기에 충분하다. cache-unmodelled 55,461자와
style 미조인 63,465자를 더 해석해도 이미 관찰한 결정적 회귀는 사라지지 않는다.

### 10.4 style과 보호 불변식

Q0 source usage의 bold 노출은 4,468자, italic은 0자다. render에서 style을 조인한 bold 1,269자에는
modelled regression이 없었고 결정적 signature도 non-bold였다. nested style 미조인 때문에 bold 전체의
dynamic completion을 주장하지 않지만, 이 open gap을 non-bold modelled regression보다 우선시키지 않는다.

rank 7 최종 disposition은 `no-change`다. qualified 전용 product-correction 자식 이슈, registry operation,
acceptance matrix와 Q4 시각 검증을 만들지 않는다. 재개하려면 일괄 face rule과 다른 좁은 feature-detected
cohort 또는 현재 회귀를 만들지 않는 새로운 한 decision-plane 가설이 필요하다.

공개 정본은
[`rank7_qualification_baseline.json`](../../tech/investigations/issue-4967/rank7_qualification_baseline.json),
[`rank7_runtime_boundary.json`](../../tech/investigations/issue-4967/rank7_runtime_boundary.json),
[`rank7_metric_hypothesis.json`](../../tech/investigations/issue-4967/rank7_metric_hypothesis.json),
[`rank7_private_qualification.json`](../../tech/investigations/issue-4967/rank7_private_qualification.json)이다.

## 11. #4967 tracker 운영 판정

2026-08-26 Q5 감사에서 #4967은 `OPEN`, 담당자 `edwardkim`, milestone `v1.0.0`이며 연결된 GitHub
sub-issue는 0개였다. 상위 #4960 본문은 #4967을 W8 실행 이슈로 가리키지만 실제 sub-issue 관계는 설정되지
않아 상위 완료 조건과 불일치한다.

현재 queue는 다음과 같다.

| lane | 상태 |
| --- | --- |
| rank 1·7·8 correction qualification | 모두 `no-change` 완료 |
| product-correction | qualified face 0, 후속 이슈 0 |
| 나머지 14개 rank | terminal disposition, 외부 evidence 변화 때만 재개 |
| W9 #4968 / W10 #4969 | 독립 OPEN, #4967 영구 OPEN 불필요 |

evidence-reopen은 현재 수행할 작업이 아니라 이벤트 기반 재개 조건이다. 이를 이유로 빈 tracker를 계속
OPEN으로 두면 완료된 W8과 실제 대기 작업을 구분하기 어렵다. 권고 후속은 현재 변경의 병합과 최종 comment
뒤에 다음 순서로 수행한다.

1. #4967을 #4960의 GitHub sub-issue로 연결해 계보를 정정한다.
2. #4960 본문의 W8 checkbox를 완료로 바꾼다.
3. #4967에 rank 1·7·8 `no-change`, product mutation 0과 명시적 reopen 조건을 남긴다.
4. #4967을 completed로 close한다.
5. 새 source·provider·localized identity·capability evidence가 실제로 생기면 #4967을 reopen하거나 해당
   face의 새 자식 이슈를 등록한다.

이 권고는 기여자의 제안 권리나 future evidence 제출을 제한하지 않는다. 현재 승인된 작업이 완료됐다는
상태 정산이며, GitHub mutation은 메인테이너의 별도 승인 전에는 수행하지 않는다.
