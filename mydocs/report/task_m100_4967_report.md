---
kind: report
status: final
canonical: mydocs/report/task_m100_4967_report.md
last_verified: 2026-08-26
---

# Task M100 #4967 최종 보고서 — W8 rank 8·rank 1 face 교정 qualification

## 1. 최종 판정

rank 8 `KoPubWorld바탕체 Light`에 exact `hmtx` advance를 일괄 적용하는 `layout-metric` 교정 후보는
**`no-change`**다. exact metric은 일부 실제 줄의 overflow를 제거하거나 slack을 늘렸지만, 현행 stored-row
cache policy가 실제로 수용한 줄에서도 신규 overflow를 포함한 회귀가 확인됐다. 평균 개선량으로 fixed-frame
회귀를 상쇄하지 않는 보호 불변식에 따라 제품 metric DB·registry·fallback·paint·supply는 변경하지 않는다.

rank 1 `문체부 바탕체`를 `MBatang`에 연결하는 `layout-name` 후보도 **`no-change`**다. localized name의
runtime miss는 실제지만, 가상 relation과 exact `MT.TTF hmtx`가 전체 layout-bearing 문자 영역에서 현행
advance와 동치여서 layout 이득이 없다. 이름 match metadata만 만들고 조판을 바꾸지 않는 제품 rule은
추가하지 않는다.

두 판정은 rank 8·rank 1 lane의 종결이다. #4967은 여러 face를 순차 판정하는 tracker이며 rank 7과
evidence-reopen lane이 남아 있으므로 **이슈는 닫지 않는다**. 2026-08-26 재확인 시 #4967은 `OPEN`, 담당자
`edwardkim`, milestone `v1.0.0`이고 연결된 열린 PR은 없었다. 현재 rank 8 결과는
[PR #6069](https://github.com/edwardkim/rhwp/pull/6069)로 제출됐으며 tracker 상태는 계속 `OPEN`이다.

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
[`rank8_private_qualification.json`](../tech/investigations/issue-4967/rank8_private_qualification.json),
구현·재판정 과정은
[`task_m100_4967_w8_q3r.md`](../working/task_m100_4967_w8_q3r.md)에 있다.

## 8. 완료와 후속 절차

W8 rank 8 qualification은 메인테이너의 최종 보고서 승인으로 `no-change` 완료됐고, code candidate는
PR #6069로 제출됐다. self-review 기록, 최신 trailing head 검증과 merge는 각각 별도 gate로 유지한다.

#4967 tracker의 다음 face 작업은 rank 8·rank 1 결과를 rank 7에 추정 적용하지 않고, 해당 face의 기존
증거와 실사용 cohort를 독립적으로 확인하는 새 절편으로 시작한다.

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

단계별 local commit은 `857241ac1`, `9fc8110d3`, `e70e35ec3`에 고정했다. 제품 source와 원격 repository는
변경하지 않았다.

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

- #4967은 2026-08-26 기준 `OPEN`, 담당자 `edwardkim`, milestone `v1.0.0`이다.
- rank 7 `KoPubWorld돋움체 Light` qualification과 evidence-reopen lane이 남아 있어 이슈를 닫지 않는다.
- W4 계측은 runtime miss와 일치하므로 정정하지 않는다.
- metric compatibility를 font identity·paint identity·재배포 권한으로 승격하지 않는다.
- private corpus identity·본문·hash와 font bytes를 공개하지 않는다.
- 10k 전수·private bounded cohort·Hyper-V 재실행과 제품 mutation은 모두 0이다.
- Q0의 기존 style aggregate 대사 외에 private 문서 재parse는 없었고 식별 정보도 공개하지 않았다.

rank 1 공개 정본은
[`rank1_qualification_baseline.json`](../tech/investigations/issue-4967/rank1_qualification_baseline.json),
[`rank1_runtime_boundary.json`](../tech/investigations/issue-4967/rank1_runtime_boundary.json),
[`rank1_metric_hypothesis.json`](../tech/investigations/issue-4967/rank1_metric_hypothesis.json)이다.

### 9.5 최신 devel 제출 전 재검증

작업 시작 뒤 `upstream/devel`이 51커밋 전진해 `ee7e8a6ed`가 됐다. 변경 파일 교집합은
`mydocs/orders/20260825.md`뿐이었고 merge-tree와 실제 merge 모두 충돌 없이 양쪽 기록을 보존했다. 다만
최신 devel의 `src/renderer/layout/text_measurement.rs`가 Q2의 추적 입력이므로, merge commit
`e10dd2258` 뒤 기존 증적을 그대로 재사용하지 않았다.

- 최신 소스 네이티브 `rhwp-q-font-trace` 재빌드 통과
- 표준 Docker WASM 최적화 빌드 통과, 추적 `pkg/` delta 0
- Q1 HWPX·HWP5 각 1,556건, native/WASM byte-exact, 첫 divergence `layout-name` 유지
- Q2 current→virtual relation 0 HWPUNIT, virtual→exact 0 HWPUNIT, 최종 `no-change` 유지
- Node 계약 8/8, Python 계약 12/12 통과
- 새 Q1 canonical SHA-256:
  `b0e22b0a76c5e5c940459eeb9b599ee5e5e962f1c985b38e799623fdd09cced4`
- 새 Q2 canonical SHA-256:
  `f50ac03f28a4b7fd53a437b9187352160a04414afaee15f932cf0a7d110be3cd`

따라서 최신 base 변경은 입력 계보를 갱신했지만 rank 1 제품 correction의 이득·위험 판정을 바꾸지 않는다.
