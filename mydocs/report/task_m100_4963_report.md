---
kind: report
status: completed
canonical: mydocs/report/task_m100_4963_report.md
last_verified: 2026-08-22
---

# Task M100 #4963 — W5 Font Oracle Profile·controlled ladder 최종 보고

Issue: #4963

## 1. 결론

#4963의 W5 기술·시각 검증을 완료했다.

- W4가 넘긴 조판 위험 상위 17개 face를 재계측·terminal·blocked로 빠짐없이 분류했다.
- rank 1 `문체부 바탕체`, rank 7 `KoPubWorld돋움체 Light`, rank 8
  `KoPubWorld바탕체 Light`는 disposable 한컴 환경에서 acceptance ladder를 완료했다.
- source를 확보할 수 없는 10개는 값을 추정하지 않고 terminal로, system font·HFT·관리 밖 provider를
  손상시켜야 하는 3개는 protected partial로 닫았다.
- rank 16 `한컴 윤고딕 230`은 영문 alias 선택과 실제 HWPX 조판이 다름을 확인해 capability mismatch로
  닫았다.
- 최종 queue의 actionable rank는 0개다. 이는 모든 후보가 5-state 수치를 가졌다는 뜻이 아니라,
  관찰값과 관찰 불가능·보호·source 부재 상태가 검증 가능한 disposition으로 정리됐다는 뜻이다.
- 제품 font metric DB·fallback·paint·layout 코드는 변경하지 않았다.

가장 중요한 제품 인사이트는 문서의 `substFont` 선언이나 설치된 face의 선택 가능 여부만으로 실제
조판 글꼴을 결정할 수 없다는 점이다. 한컴의 PDF export가 사용한 subset·glyph·advance를 직접 확인하는
기능 탐지가 fallback 정책의 근거가 되어야 한다.

## 2. 산출물과 authority

| 산출물 | 역할 |
| --- | --- |
| [W5 수행계획](../plans/task_m100_4963.md) | 범위·재계측 금지선·controlled ladder·보호 불변식 |
| [Oracle investigation index](../tech/investigations/issue-4963/README.md) | schema·fixture·profile·validator·재생성 명령 |
| [17-face queue projection](../tech/investigations/issue-4963/oracle_stage5_queue_projection.json) | 최종 disposition 기계 정본 |
| [rank 8 acceptance ladder](../tech/investigations/issue-4963/oracle_stage5_rank8_acceptance_ladder.json) | distinct substitution three-state 계보 |
| [rank 16 disposition](../tech/investigations/issue-4963/oracle_stage5_rank16_read_only_disposition.json) | 문서 face 기능 불일치 정지 근거 |
| [W5-4B 실행 보고](../working/task_m100_4963_w5_stage4b.md) | disposable VM canary·복원·rank 1·7 관측 |
| [W5-5C 실행 보고](../working/task_m100_4963_w5_stage5c.md) | rank 8 기계·시각 판정 |

기계 정본의 현재 file SHA-256은 다음과 같다.

| 파일 | SHA-256 |
| --- | --- |
| `oracle_stage5_queue_projection.json` | `7765e060982c672cac8fbd0700f73e21d7488ae2fb25144c8046a4e678e0002d` |
| `oracle_stage5_rank8_acceptance_ladder.json` | `d6e8a4371dd049a899a88fb975d6499ed435154b72e1fc804b701addf3cb75ec` |
| `oracle_stage5_rank16_read_only_disposition.json` | `dbf596fd79aa4b0d55a00f5761d8e93b7a006606eddc44455937815e9bc1eeda` |

## 3. 계획 대비 실행

W5는 W4의 10k corpus ranking을 다시 전수 계측하지 않았다. 기존 W3·W4·한컴 2022 evidence는 입력과
contract hash가 일치할 때 재사용하고, 실제 상태가 비어 있던 후보만 제한 실행했다.

| 계획 항목 | 실행 결과 |
| --- | --- |
| 17개 readiness·disposition | 17/17 완료 |
| Oracle Profile schema·negative contract | 완료 |
| deterministic public fixture·SFNT/PDF 관측기 | 완료 |
| read-only canary | 한컴 2020 build `11.0.0.9136`에서 완료 |
| disposable controlled canary | rank 1·7 완료, rank 13 보호 정지 |
| actionable queue 확대 | rank 16 기능 불일치 정지, rank 8 ladder 완료 |
| side-by-side 시각 판정 | 메인테이너 승인 |
| 제품 구현 | 범위 밖으로 유지, 변경 0 |

계획과 달리 모든 face에 동일한 5-state 수치를 채우지 않았다. source 부재나 HFT·system provider 보호를
무시하고 빈 값을 만드는 것은 fail-closed 원칙에 어긋나므로, blocker와 재개 조건을 가진 terminal
disposition을 정상 완료로 인정했다.

## 4. 17개 face 최종 disposition

| disposition | 수 | rank | 의미 |
| --- | ---: | --- | --- |
| `complete-acceptance-ladder` | 3 | 1, 7, 8 | exact/subst/missing 물리 상태와 semantic profile 완료 |
| `terminal-protected-partial` | 3 | 9, 10, 13 | system font·HFT·관리 밖 provider를 손상시키지 않고 기존 관측 재사용 |
| `terminal-read-only-capability-mismatch` | 1 | 16 | alias 선택은 가능하지만 문서 face와 PDF subset exact 연결 실패 |
| `terminal-source-unavailable` | 10 | 2–6, 11, 12, 14, 15, 17 | source identity 없이 metric·glyph를 추정하지 않음 |

`oracle_stage5_queue_projection.json`은 `stage=W5-5C`, `actionableRanks=[]`이며 세부 profile file hash와
재개 조건을 함께 가진다. source나 안전한 provider 제어 능력, schema 또는 환경 identity가 실제로
달라지기 전에는 terminal 후보를 반복 계측하지 않는다.

## 5. acceptance ladder가 확인한 것

rank 1·7·8은 같은 HWPX bytes를 상태 사이에서 유지하고 대상 related font 집합만 바꿨다. exact font가
있을 때는 해당 source subset을 PDF에서 확인했지만, fixture가 선언한 별도 substitution font만 설치한
상태는 세 후보 모두 한컴의 `HCRBatang-Bold` fallback으로 귀결됐다. `substFont`의 존재와 설치 가능성은
실제 사용을 보장하지 않았다.

raw PDF hash는 생성시각 같은 실행 metadata 때문에 달라질 수 있다. 따라서 동일 조판 판정은 file
hash가 아니라 font·glyph·advance·position·line projection으로 수행했다. 이 원칙으로 rank 8
subst-only와 none-related의 typesetting projection은
`59801255a8663ef14c9796be76942bc95b2fa46f80bc6b0631bf5bd220c827be`로 동일했다.

## 6. rank 16이 보여준 기능 탐지 원칙

`한컴 윤고딕 230`의 영문 SFNT alias `Haan YGodic 230`은 exact TTF로 선택 가능했다. 그러나 복원된
기준선에서 HWPX를 실제로 열면 문서의 한글 face는 `함초롬바탕`으로 readback됐고, PDF에는
`HCRBatang-Bold`만 들어갔다.

따라서 단발 선택 probe나 한컴 빌드 번호를 정책 분기로 사용하지 않는다. 다음 기능을 각각 독립적으로
탐지해야 한다.

1. font bytes가 설치되어 있는가
2. 요청한 localized/English face가 선택 가능한가
3. 문서의 face 이름이 exact로 readback되는가
4. 실제 export subset이 그 source bytes와 연결되는가
5. glyph·advance·line·page 관측이 같은가

앞 단계의 약한 probe와 현재의 강한 문서 open·export 결과가 충돌하면 후자를 우선하고, exact profile을
발행하지 않는다.

## 7. rank 8 시각·조판 인사이트

메인테이너가 승인한 local-only side-by-side 이미지는 exact-only, subst-only, none-related 순서다.
exact-only는 더 가늘고 폭이 다른 KoPubWorld 바탕 조판을 보였으며, 뒤의 두 상태는 서로 같은 HCR 바탕
fallback 외형이었다. 이 시각 판정은 subset과 typesetting projection 결과와 일치했다. 이미지 자체는
실행 증적이며 저장소에 추적하지 않고, SHA-256
`9d4da59dfaba6f4dcb0fd06e1268fcb490690c66998bd0574df607f376bb90cb`만 기록한다.

| 상태 | PDF subset | U+AC00 source `hmtx` | PDF advance |
| --- | --- | ---: | ---: |
| exact-only | `KoPubWorldBatangLight` | 936/1000 | 7.454008 |
| subst-only | `HCRBatang-Bold` | 연결하지 않음 | 7.774934 |
| none-related | `HCRBatang-Bold` | 연결하지 않음 | 7.774934 |

fallback의 U+AC00 PDF advance는 exact보다 약 4.3% 컸다. 한 글자 차이만으로 페이지 증가를 주장할 수는
없지만, 공공 HWP 문서에서 흔한 장평 100/90/80, 자간 0/-5/-10, kerning on/off와 고정 폭 표 셀·글상자에
이 차이가 누적되면 줄 경계와 overflow 위험이 커질 수 있다. 그러므로 fallback 후보는 이름 유사성이나
평균 폭 하나가 아니라 다음 축을 함께 비교해야 한다.

- 실제 export-selected subset과 glyph outline
- 문자별 advance 및 weight·width 차이
- 장평·자간·kerning 변화에 대한 응답
- 본문·머리말·꼬리말·표 셀·글상자별 line boundary
- stored/fresh LineSeg 상태에서 최초 divergence 위치

이번 synthetic fixture는 세 상태 모두 30개 visual line과 1쪽이었다. 따라서 실제 줄 증가·overflow·페이지
변화는 이번에 관찰한 결과가 아니라, 위 4.3% advance 차이와 편집축의 결합에서 도출한 후속 검증
가설이다. 시각 차이는 본문뿐 아니라 머리말·꼬리말·표·글상자 문맥에서도 보였지만 문맥별 실패율을
수치화한 것은 아니다.

## 8. 보호 불변식과 복구 증거

- 세 상태의 HWPX bytes를 동일하게 유지했다.
- related font 이외 ambient projection은
  `437a36e513cce9d2909d904f3d07d2341051cc017e21be9ec6d35bbb9d87bc78`로 복구했다.
- 최종 baseline font manifest는
  `3bcd379d1f7fc217aad47a0b44b952d993c86ebbfabf46009386e4b3de768b40`이었다.
- 최종 복원 뒤 managed font 0개, HWP process 0개, 일회성 task 0개를 확인했다.
- font bytes, VM·checkpoint 이름, 절대 경로, private corpus 문서·식별자는 공개 산출물에 넣지 않았다.

첫 rank 8 전건 orchestration은 none-related의 빈 managed set을 `Compare-Object`에 넘기는 guard 오류로
중단됐다. 외부 `finally`가 기준선을 복원했고 이를 독립 검증했다. 이미 완료된 exact/subst 상태를 다시
실행하지 않고 그 증거를 재사용했으며, none-related만 빈 집합 전용 guard로 재실행한 뒤 마지막 복원도
검증했다. 실패를 숨기지 않고 복구 불변식이 실제 장애 경로에서도 작동했다는 증거로 보존한다.

## 9. 검증 결과

| 검증 | 결과 |
| --- | --- |
| Oracle Node contract·profile tests | 13/13 통과 |
| Stage 2·3·4·4 profile·5 queue Python tests | 39/39 통과 |
| queue candidate/disposition reconciliation | 17/17, 오류 0 |
| rank 8 input·profile·ladder hash 연결 | 통과 |
| baseline/unrelated font 복원 | 통과 |
| private corpus 접근·식별 정보 공개 | 0 |
| font bytes·절대 VM path 공개 | 0 |
| 메인테이너 side-by-side 시각 판정 | 통과 |
| 변경 Markdown 내부 상대 링크 | 5개, 이상 없음 |
| `cargo fmt --all -- --check`·diff check | 통과 |

`cargo fmt --all`을 먼저 적용한 뒤 같은 전체 workspace 범위의 `--check`를 수행했다.

## 10. 제품 후속 후보

#4963은 Oracle evidence를 만드는 이슈이므로 다음 제품 변경을 함께 구현하지 않는다. 별도 이슈 후보는
우선순위대로 다음과 같다.

1. fallback 결정에 export-selected font/glyph/advance evidence를 연결하는 진단 계층
2. font 후보를 weight·width·문자별 advance와 장평·자간 응답으로 비교하는 metric 정책
3. document `substFont`를 자동 성공이 아니라 검증해야 할 관계 hint로 취급하는 fail-closed resolver
4. body/header/footer/table/text-box와 stored/fresh LineSeg를 포함한 overflow 회귀 fixture
5. 새 font incident에서 exact/subst/missing ladder를 재사용하는 maintainer 절차

이 후보는 #4960의 W5 완료 상태를 갱신한 뒤 독립 범위·승인·PR로 진행한다. source가 새로 확보된 terminal
face도 기존 queue를 폐기하지 않고 같은 Oracle 계약으로 재개한다.

## 11. 완료와 남은 절차

기술 완료 조건인 17개 최종 disposition, 실행 profile 계보, blocker·재개 조건, privacy 경계와 시각
판정을 모두 충족했다. 이 최종 보고서에 대한 메인테이너 승인 뒤 #4963 통합 준비, 승인된 원격 push·PR,
self-review, CI, merge와 이슈 close를 각각 프로젝트 절차에 따라 진행한다. #4960의 W5 상태와 제품 후속
이슈 후보도 통합 결과를 확인한 뒤 별도 갱신한다.
