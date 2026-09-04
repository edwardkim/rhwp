---
kind: working-note
status: completed
issue: 4962
stage: W4-3
last_verified: 2026-08-22
---

# Task M100 #4962 W4 Stage 3 — evidence 상승 조건·민감도 band

- **이슈**: [#4962](https://github.com/edwardkim/rhwp/issues/4962)
- **계획**: [`task_m100_4962_w4.md`](../plans/archives/task_m100_4962_w4.md)
- **선행 단계**: [`task_m100_4962_w4_stage2.md`](task_m100_4962_w4_stage2.md)
- **브랜치**: `task_m100_4962`
- **단계 상태**: W4-3 완료, W4-4 승인 대기

## 1. 결론

W4-2의 위험 문자 2,061,732자와 base risk mass 8,656,719를 바꾸지 않고, W1 ledger·정부상징
successor matrix·KoPub 검증·과거 공급 조사를 document face exact 이름으로만 결합했다. 비슷한 이름,
공급 가능성, 파일명만으로 font identity나 metric 호환성을 추정하지 않았다.

기본·무가중·frame-neutral·non-extreme·stored/fresh lane의 6개 순위를 비교했다. 정확한 단일 순위는
계수와 lane에 민감하므로 base cumulative risk mass의 시작점 기준으로 A 0–50%, B 50–80%, C 80–95%,
D 95–100% band를 정본으로 삼았다. evidence는 base mass를 바꾸지 않으며 같은 band 안에서만 action rank를
정렬한다. 근거 없는 promotion, identity 추정, cross-band 이동은 모두 0건이다.

## 2. 계약과 구현

### 2.1 동결 evidence

계약에는 다음 tracked 입력의 repository-relative 경로와 SHA-256을 고정했다.

| 역할 | 입력 | SHA-256 앞 12자리 |
| --- | --- | --- |
| backend·relation | W1 `font_rule_ledger.json` | `284afd72259e` |
| 정부 successor | #4739 government matrix | `7e5028e703c3` |
| 과거 공급 보조 | font/jsDelivr survey TSV | `8235c6974590` |
| KoPub·정부 검증 | #4739 Stage 5 | `09f09079d8fb` |
| backend capability | #4741 Stage 5 | `55779150ac67` |
| KoPub backend format | #4764 Stage 1 | `044d078ed49d` |

`scripts/font_typesetting_risk_evidence.mjs`는 실행 전에 이 여섯 hash와 W4-2 canonical output hash
`d81e9a34…30c595e`를 확인한다. W3 aggregate를 다시 생성하지 않고 기존 `decisionUsage`만 bounded
streaming으로 읽어 다섯 민감도 질량을 계산한다.

### 2.2 이름과 상태의 분리

- W1과 survey join key는 document face 문자열의 byte-exact 값이다.
- `exactSource.status`는 `verified`·`available`·`unavailable`·`unknown` 중 하나다.
- Canvas2D와 CanvasKit은 각각 availability·profile·evidence status·rule ID를 가진다.
- survey의 `document_count`는 위험량·도달률에 사용하지 않고 출력에서도 제외했다.
- `unknown-relation`은 W1이 명시한 `relationType=unknown`에서만 켠다.
- `exact-source-available`과 `unknown-relation`은 상태·후속 질문용이며 action promotion 근거가 아니다.

action rank의 우선순위는 계획서가 허용한 세 근거만 사용한다.

1. `exact-source-verified`
2. `government-or-legal-core`
3. `backend-selection-divergence`
4. 기존 base rank

각 band가 원래 차지한 rank 최소·최대 구간 밖으로 action rank가 나가면 hard failure다.

## 3. 민감도와 안정 band

### 3.1 변형별 질량과 band 크기

| 변형 | 전체 질량 | A | B | C | D |
| --- | ---: | ---: | ---: | ---: | ---: |
| base | 8,656,719 | 6 | 11 | 32 | 302 |
| unweighted | 2,061,732 | 7 | 14 | 36 | 294 |
| frame-neutral | 5,425,368 | 6 | 11 | 34 | 300 |
| non-extreme | 6,658,421 | 6 | 12 | 33 | 300 |
| stored-line-lane | 7,975,624 | 6 | 12 | 31 | 302 |
| fresh-candidate-lane | 681,095 | 1 | 0 | 2 | 348 |

전역 가중치 네 변형에서 같은 band를 유지한 face는 329/351개다. stored/fresh lane까지 모두 포함해 같은
band를 유지한 face는 292/351개다. 반면 여섯 변형에서 rank 숫자까지 완전히 같은 face는 0개이고 최대
rank span은 345다. 특히 fresh lane 질량은 base의 작은 일부이므로 0 질량 동률을 base rank로 결정해도
정밀 순위가 크게 벌어진다. 따라서 **band는 안정된 공표 단위지만 단일 rank 숫자는 안정성 주장이
아니다**.

base band의 범위는 A 1–6, B 7–17, C 18–49, D 50–351이다. action rank도 각 범위를 exact하게
유지했다.

## 4. evidence join 결과

| 항목 | face 수 |
| --- | ---: |
| 위험 후보 | 351 |
| W1 ledger exact join | 115 |
| 과거 supply survey exact join | 281 |
| exact source verified | 7 |
| exact source available·bytes 미검증 | 6 |
| exact source unavailable | 0 |
| backend selection divergence | 61 |
| explicit unknown relation | 30 |

survey 상태는 `not-found` 217, `available` 48, `license-review` 16, exact row 없음 70이다. 이는 2026-08-15
시점의 공급 보조 정보일 뿐 metric 호환성·현재 가용성·corpus 도달률을 뜻하지 않는다.

### 4.1 KoPub exact source 경계

로컬 검증한 여섯 TTF의 name table과 SHA-256을 읽었다. 위험 후보와 exact하게 일치한 이름은 다음 7개다.

- `KoPubBatangLight` — PostScript name exact
- `KoPub돋움체 Bold`, `KoPub돋움체 Light`, `KoPub돋움체 Medium`
- `KoPub바탕체 Bold`, `KoPub바탕체 Light`, `KoPub바탕체 Medium`

이름과 digest는 계약에 기록했지만 로컬 절대 경로는 기록하지 않았다. KoPubWorld의 `_Pro`가 아닌 6개
이름은 W1·공급 경로가 존재하지만 이번 local name-table 검증 묶음의 bytes가 아니므로 `available`까지만
허용했다. `_Pro` 이름은 비슷해 보여도 join하거나 successor로 추정하지 않았다.

정부상징 legacy exact face인 `정부상징 부처명_16040911`과 `Government_16040911`은 정부 핵심·exact
source unavailable 계약에 남아 있다. 그러나 351개 위험 후보에는 exact 이름이 없으므로 정부 flag나
action promotion은 0건이다. ROKG는 공식 successor이지만 legacy와 byte·metric identity가 아니므로
후보 이름으로 치환하지 않았다.

### 4.2 backend 상태

Canvas2D/CanvasKit availability 조합은 다음과 같다.

| Canvas2D / CanvasKit | face 수 |
| --- | ---: |
| unknown / unknown | 288 |
| available / available | 14 |
| unknown / available | 2 |
| available / unavailable | 47 |

두 backend에 exact W1 rule이 있고 availability 또는 profile이 다른 61개만
`backend-selection-divergence`로 표시했다. Canvas2D CSS 사용 가능성을 CanvasKit SFNT 가능성으로
승격하지 않았다.

## 5. action rank와 종료 게이트

evidence 정렬 결과 58개가 같은 band 안에서 앞당겨졌고, 그 영향으로 260개가 뒤로 밀렸으며 33개는
그대로였다. Band A의 6개는 모두 그대로다. B·C·D도 각각 원래 7–17, 18–49, 50–351 범위 안에서만
재배열됐다. 모든 이동 후보는 direction과 적용 evidence 또는 peer promotion 이유를 기계 판독 가능한
배열로 가진다.

| hard gate | 결과 |
| --- | ---: |
| base risk mass 변경 | 0 |
| 근거 없는 promotion | 0 |
| identity guess | 0 |
| cross-band promotion | 0 |
| privacy finding | 0 |

이 action rank는 W4-3의 evidence 순서 투영이며 W5 실행 큐 확정이 아니다. 후보별 exact 설치·missing·
`substFont`·official successor·모두 미설치 질문은 W4-4에서 붙인다.

## 6. 반복 실행과 검증

두 local-only 결과는 다음과 같이 exact했다.

| 항목 | r1 | r2 |
| --- | ---: | ---: |
| mode | `0600` | `0600` |
| bytes | 599,952 | 599,952 |
| file SHA-256 | `087bebf1…60ff98` | `087bebf1…60ff98` |
| canonical output hash | `671e7174…c91d06` | `671e7174…c91d06` |

결과는 `output/poc/font-typesetting-risk/rank-stage-w4-3-r{1,2}.json`에 있으며 gitignored local-only
파생물이다. tracked 파일에는 corpus path·filename·document hash·raw row·문자 trace가 없다.

최종 검증은 W4 test 12/12, W3 focused test 12/12, JSON Schema Draft 2020-12 strict compile·instance
validation, 두 `.mjs` syntax, Markdown link, `cargo fmt --all -- --check`, `git diff --check`를 포함한다.

## 7. 다음 승인 지점

Stage W4-4에서는 이 결과를 공개 가능한 집계 보고서와 W5 인계 후보로 정리하고, 후보별 검증 질문을
작성한다. 메인테이너가 W4 최종 보고와 action queue를 승인하기 전에는 W5 controlled ladder,
metric DB·fallback·font asset 변경, 원격 push·PR·GitHub 본문 변경을 시작하지 않는다.
