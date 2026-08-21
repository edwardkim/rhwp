---
kind: report
status: completed
canonical: mydocs/report/task_m100_4962_report.md
last_verified: 2026-08-22
---

# Task M100 #4962 — 실제 renderer coverage와 조판 위험 순위 최종 보고

## 1. 결론

#4962의 W3·W4 기술 산출물을 완료했다.

- private 10k 입력을 실제 renderer decision으로 2회 계측해 110,097,106-byte aggregate가 byte exact임을
  확인했다.
- 실제 layout 문자 54,399,371자와 coverage 54,326,042자의 분모를 손실 없이 대사했다.
- metric 위험 문자 2,061,732자를 face-miss·char-miss·heuristic으로 분리했다.
- 장평·자간·fixed-context proxy와 같은 usage row에서 함께 나타난 위험만 8,656,719의 base risk mass로
  계산했다.
- 351개 위험 face를 A 6개, B 11개, C 32개, D 302개의 cumulative band로 분리했다.
- W5 1차 queue는 A+B 17개이며 base risk mass의 810,374 ppm을 설명한다.
- 각 queue 후보에 #4963 controlled ladder의 다섯 질문을 만들었다.
- 제품 renderer·metric DB·fallback·font asset 변경은 0이다.

이 보고서와 W5 action queue는 2026-08-22 메인테이너의 W4 최종 승인을 받았다. 원격 push·PR,
#4960·#4962 GitHub 상태 갱신과 #4963 착수는 각각 후속 승인 대상이다.

## 2. 산출물과 authority

| 산출물 | 역할 |
| --- | --- |
| [W3 수행계획](../plans/task_m100_4962.md) | 실제 renderer coverage 범위·보안·실행 게이트 |
| [W4 수행계획](../plans/task_m100_4962_w4.md) | 위험 mass·민감도·evidence·W5 인계 계약 |
| [W3 결정성 보고](../working/task_m100_4962_stage4d_determinism.md) | 10k r2/r3 byte exact 근거 |
| [W4 evidence 보고](../working/task_m100_4962_w4_stage3.md) | exact join·민감도 band·promotion gate |
| [W4 공개 단계 보고](../working/task_m100_4962_w4_stage4.md) | public projection·GitHub 인계 경계 |
| [공개 ranking JSON](assets/task_m100_4962/font_typesetting_risk_rank.json) | 351개 ranking과 17개 W5 queue의 기계 정본 |

원인 계보와 보호 불변식 FI-01~FI-14의 정본은
[`font_metrics_fallback_causal_lineage_20260816.md`](font_metrics_fallback_causal_lineage_20260816.md)다.

## 3. W3 실제 renderer coverage

### 3.1 실행과 결정성

| 항목 | 결과 |
| --- | ---: |
| attempted / success / failure | 10,000 / 9,909 / 91 |
| layout / coverage / not-applicable 문자 | 54,399,371 / 54,326,042 / 73,329 |
| paragraph / source run | 3,764,899 / 5,602,126 |
| joined / layout-only / excluded | 54,399,371 / 0 / 0 |
| legacy / decision usage row | 66,447 / 138,965 |
| r2/r3 JSON bytes | 110,097,106 / 110,097,106, exact |
| aggregate SHA-256 | `37867105…c2888` |
| file SHA-256 | `24eb1d15…352a1` |

10,000개 document status·failure 또는 aggregate token과 checkpoint chain이 모두 exact였다. elapsed·RSS
같은 운영 metric만 결정성 hash에서 제외했다. 지원 대상 실패 6개는 encrypted 5개와 resource limit
1개이며 parser·cancelled·truncated는 0이다.

### 3.2 문자별 coverage

| category | 문자 | coverage 대비 |
| --- | ---: | ---: |
| exact-hit | 48,071,620 | 88.4872% |
| measured-overlay | 4,192,417 | 7.7171% |
| metric-surrogate | 273 | 0.0005% |
| identity-alias-hit | 0 | 0.0000% |
| face-miss | 1,642,295 | 3.0230% |
| char-miss | 391,018 | 0.7198% |
| heuristic | 28,419 | 0.0523% |

성공 계열은 52,264,310자, 96.2049%다. 이는 기존 POC의 face-level `metric mapped 91.16%`와 분모가
다르므로 개선율로 비교하지 않는다.

## 4. W4 조판 위험 계약

W4는 위험 category의 문자 수 `n`에 같은 usage row에서 실제 관찰한 편집축만 적용한다.

```text
compressionFactor = 1
  + I(ratio < 100)
  + I(ratio <= 90)
  + I(spacing < 0)
  + I(spacing <= -5)

frameFactor = fixedFrameContextProxy ? 2 : 1
rowRiskMass = n * compressionFactor * frameFactor
```

category별 주관 가중치와 문서 선언 빈도는 사용하지 않는다. stored LineSeg는 validity나 버전 분기가
아니며 stored/fresh lane을 분리할 뿐 multiplier가 아니다.

| 위험 분모 | 값 |
| --- | ---: |
| 위험 문자 | 2,061,732 |
| compressed fixed-context 교집합 | 779,851 |
| stored risk mass | 7,975,624 |
| fresh-candidate risk mass | 681,095 |
| base risk mass | 8,656,719 |
| observed / ranked document face | 407 / 351 |

base cumulative mass의 candidate 시작점을 기준으로 A 0–50%, B 50–80%, C 80–95%, D 95–100% band를
사용한다. A+B 마지막 후보가 경계를 넘기므로 17개 queue의 실제 base mass 비중은 810,374 ppm이다.

전역 가중치 변형에서 같은 band를 유지한 face는 329/351개, stored/fresh lane까지 포함하면 292/351개다.
모든 변형에서 exact rank까지 같은 face는 0개다. 따라서 band는 공표 단위이고 정밀 rank는 조사 순서다.

## 5. W5 1차 action queue

Queue는 A+B 17개다. 위험 문자 1,562,076자(757,652 ppm), base risk mass 7,015,182(810,374 ppm),
compressed fixed-context 위험 문자 635,638자를 포함한다.

| action | base | face | band | 위험 문자 | base mass | 압축·fixed 문자 | stored / fresh mass | exact | Canvas2D / CanvasKit | supply |
| ---: | ---: | --- | :---: | ---: | ---: | ---: | ---: | --- | --- | --- |
| 1 | 1 | `문체부 바탕체` | A | 208,986 | 1,328,953 | 153,061 | 1,328,953 / 0 | unknown | unknown / unknown | not-found |
| 2 | 2 | `-윤명조120` | A | 249,518 | 1,017,585 | 45,415 | 1,016,033 / 1,552 | unknown | unknown / unknown | unknown |
| 3 | 3 | `-윤고딕110` | A | 90,537 | 773,758 | 74,358 | 152,202 / 621,556 | unknown | unknown / unknown | unknown |
| 4 | 4 | `산돌명조 L` | A | 83,616 | 638,770 | 78,589 | 638,770 / 0 | unknown | unknown / unknown | not-found |
| 5 | 5 | `-윤고딕310` | A | 62,658 | 483,916 | 58,514 | 483,916 / 0 | unknown | unknown / unknown | unknown |
| 6 | 6 | `-윤고딕120` | A | 75,526 | 459,021 | 66,268 | 458,238 / 783 | unknown | unknown / unknown | unknown |
| 7 | 7 | `KoPubWorld돋움체 Light` | B | 63,732 | 439,387 | 51,924 | 439,387 / 0 | available | available / available | available |
| 8 | 13 | `KoPubWorld바탕체 Light` | B | 43,374 | 141,412 | 9,034 | 141,412 / 0 | available | available / available | available |
| 9 | 15 | `맑은 고딕` | B | 35,379 | 127,542 | 11,781 | 127,301 / 241 | unknown | available / unavailable | not-found |
| 10 | 8 | `한양신명조` | B | 110,353 | 288,407 | 17,538 | 286,610 / 1,797 | unknown | unknown / unknown | not-found |
| 11 | 9 | `-윤명조320` | B | 124,877 | 281,525 | 839 | 281,525 / 0 | unknown | unknown / unknown | unknown |
| 12 | 10 | `-윤명조130` | B | 239,971 | 278,909 | 712 | 278,909 / 0 | unknown | unknown / unknown | unknown |
| 13 | 11 | `휴먼명조` | B | 70,866 | 217,791 | 14,405 | 193,612 / 24,179 | unknown | unknown / available | not-found |
| 14 | 12 | `-윤고딕320` | B | 20,945 | 177,119 | 19,260 | 177,119 / 0 | unknown | unknown / unknown | unknown |
| 15 | 14 | `-윤고딕130` | B | 34,672 | 138,553 | 10,851 | 138,553 / 0 | unknown | unknown / unknown | unknown |
| 16 | 16 | `한컴 윤고딕 230` | B | 27,303 | 114,056 | 11,552 | 114,056 / 0 | unknown | unknown / available | not-found |
| 17 | 17 | `산돌고딕 M` | B | 19,763 | 108,478 | 11,537 | 108,478 / 0 | unknown | unknown / unknown | not-found |

Band A는 evidence 때문에 순서가 바뀌지 않았다. B에서는 backend profile 근거가 있는 KoPubWorld 바탕과
맑은 고딕만 같은 band 안에서 앞당겨졌다. `available` 상태 자체는 promotion 근거로 쓰지 않았다.

이 표에서 다음 네 지점이 W5 전략에 중요하다.

1. 17개 중 exact bytes verified 후보가 0개다. 먼저 source identity를 확보하거나 `unknown`으로 회수해야 한다.
2. `문체부 바탕체`는 base mass 1위이고 모두 stored lane이므로 저장된 조판과 exact source가 첫 질문이다.
3. `-윤고딕110`은 fresh mass 621,556이므로 버전이 아니라 LineSeg 상태별 divergence를 우선 비교한다.
4. KoPubWorld 2개와 맑은 고딕은 backend 상태·profile을 합치지 않은 controlled ladder가 필요하다.

## 6. 후보별 W5 질문

공개 ranking JSON은 위 17개 각각에 다음 다섯 질문을 face 이름과 현재 readiness에 맞춰 완전한 배열로
기록한다.

1. **exact 설치**: exact bytes와 face index를 고정했을 때 glyph outline, `hmtx` advance와 첫 조판
   divergence는 무엇인가?
2. **exact 제거**: exact font만 제거했을 때 한컴 PDF가 선택하는 subset font와 첫 divergence는 무엇인가?
3. **문서 `substFont`만 제공**: exact를 제거하고 문서 substitution만 제공할 때 layout metric과 paint
   관계는 무엇인가?
4. **official successor만 설치**: 직접 공개 anchor가 있는 successor만 설치했을 때 identity가 아닌
   successor 관계와 첫 divergence는 무엇인가? 근거가 없으면 `notProvided`다.
5. **모두 미설치**: exact·substFont·검증된 successor를 모두 제거했을 때 한컴 missing-font 선택과
   backend별 재현 차이는 무엇인가?

각 profile은 최소한 입력 SHA-256, 한컴 버전, PDF producer, 설치 font digest, exact/missing 상태,
subset font name, glyph outline digest, `hmtx` advance, 첫 조판 divergence와 relation evidence를 가져야 한다.
이 필드는 #4963 Oracle Profile schema의 입력 요구이며 W4가 schema를 선행 구현한 것은 아니다.

## 7. Evidence와 해석 한계

| evidence | face 수 |
| --- | ---: |
| W1 ledger exact join | 115 |
| historical supply exact join | 281 |
| exact source verified / available / unavailable | 7 / 6 / 0 |
| backend selection divergence | 61 |
| explicit unknown relation | 30 |

- KoPub local name table과 digest로 verified한 7개 face는 전체 ranking에 남지만 A+B에는 없다.
- KoPubWorld 6개는 공급 경로가 있어도 bytes 미검증이므로 `available`까지만 기록한다.
- 정부상징 legacy exact 이름은 351개 후보에 없으므로 ROKG로 치환하거나 공공 flag로 승격하지 않았다.
- supply survey는 historical availability일 뿐 metric compatibility나 현재 가용성의 증거가 아니다.
- fixed-context는 context token proxy이며 실제 frame geometry나 overflow 판정이 아니다.
- stored LineSeg는 존재 상태이지 유효성 판정이 아니다.
- face identity, successor, document substitution, metric surrogate와 Hancom missing-font는 W5 evidence 없이
  확정하지 않는다.

## 8. 종료 게이트와 검증

| gate | 결과 |
| --- | ---: |
| W3 category·join·document reconciliation 오류 | 0 |
| W3 r2/r3 semantic·byte drift | 0 |
| W4 base risk mass 변경 | 0 |
| 근거 없는 promotion | 0 |
| identity guess | 0 |
| cross-band promotion | 0 |
| queue A+B 밖 후보 | 0 |
| 공개 privacy finding | 0 |

공개 ranking은 892,640 bytes이며 두 번 생성한 파일 SHA-256이
`6947e9e8a6c67a60a54b04dc6f1abf75e3cc66d9096a978d301ba2c10bb4ee3a`로 exact다. canonical output hash는
`95e7a41d1ed92a60cb66e1705b038c3e9086829b3c8aee48af57e8c2da111a68`이다.

최종 검증은 W4 test, W3 contract·checkpoint 회귀, JSON Schema strict validation, `.mjs` syntax,
Markdown 링크, `cargo fmt --all -- --check`, `git diff --check`를 포함한다.

## 9. GitHub 후속 후보

2026-08-22 read-only 확인에서 #4960·#4962·#4963은 모두 OPEN이다.

- **#4960**: W3+W4 체크 후보. 통합 뒤 갱신한다.
- **#4962**: 기술 완료 후보. 이 branch의 PR merge와 후속 검증 전에는 close하지 않는다.
- **#4963**: A+B 17개 queue와 후보별 질문을 입력으로 삼는다. 별도 수행계획 승인 전에는 착수하지 않는다.

메인테이너가 2026-08-22 risk contract, 안정성 분석, W5 action queue와 질문을 승인해 W4 종료 조건을
충족했다. 다음 절차는 #4962 통합 준비, 별도 승인을 받은 원격 push·PR, #4963의 독립 수행계획 수립이다.
