---
kind: report
status: final
canonical: mydocs/report/task_m100_4967_report.md
last_verified: 2026-08-25
---

# Task M100 #4967 최종 보고서 — W8 rank 8 face 교정 qualification

## 1. 최종 판정

rank 8 `KoPubWorld바탕체 Light`에 exact `hmtx` advance를 일괄 적용하는 `layout-metric` 교정 후보는
**`no-change`**다. exact metric은 일부 실제 줄의 overflow를 제거하거나 slack을 늘렸지만, 현행 stored-row
cache policy가 실제로 수용한 줄에서도 신규 overflow를 포함한 회귀가 확인됐다. 평균 개선량으로 fixed-frame
회귀를 상쇄하지 않는 보호 불변식에 따라 제품 metric DB·registry·fallback·paint·supply는 변경하지 않는다.

이 판정은 rank 8 lane의 종결이다. #4967은 여러 face를 순차 판정하는 tracker이며 rank 1·7과
evidence-reopen lane이 남아 있으므로 **이슈는 닫지 않는다**. 작업 착수 시 #4967은 `OPEN`, 담당자
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

#4967 tracker의 다음 face 작업은 rank 8 결과를 rank 1·7에 추정 적용하지 않고, 각 face의 기존 증거와
실사용 cohort를 독립적으로 확인하는 새 절편으로 시작한다.
