---
kind: working-note
status: completed
issue: 4963
stage: W5-5A
last_verified: 2026-08-22
---

# Task M100 #4963 W5 Stage 5A — 17-face reuse action matrix

- **이슈**: [#4963](https://github.com/edwardkim/rhwp/issues/4963)
- **계획**: [`task_m100_4963.md`](../plans/task_m100_4963.md)
- **선행 단계**: [`task_m100_4963_w5_stage4b.md`](task_m100_4963_w5_stage4b.md)
- **단계 상태**: 전건 reuse·terminal·actionable 분류 완료, W5-5B로 이관

## 1. 목적

W5-5A는 17개 face를 다시 전수 실행하는 단계가 아니다. W5-0~4에서 이미 고정한 readiness, historical
profile, acceptance ladder와 blocked 관찰을 결합해 다음 네 가지를 분리한다.

1. hash가 맞아 그대로 재사용할 profile
2. source 부재로 수치를 만들지 않고 종료할 terminal disposition
3. system font·bundled HFT·unmanaged exact를 손상시키지 않기 위한 terminal disposition
4. 기존 근거만으로 닫을 수 없어 실제 후속 실행이 필요한 candidate

10k private corpus 재계측과 제품 metric DB·fallback·paint 변경은 수행하지 않았다.

## 2. 전건 판정

| 분류 | rank | 수 | 판정 |
| --- | --- | ---: | --- |
| acceptance ladder 완료 | 1, 7 | 2 | 8개 primary profile 재사용 |
| source unavailable terminal | 2–6, 11, 12, 14, 15, 17 | 10 | source discovery 전까지 exact anchor를 만들지 않음 |
| protected/immutable partial terminal | 9, 10, 13 | 3 | Windows system font·HFT·unmanaged provider를 제거하지 않음 |
| controlled ladder 대기 | 8 | 1 | exact와 다른 explicit substFont 계약 필요 |
| read-only exact profile 대기 | 16 | 1 | 설치 변경 없이 exact PDF profile만 추가 |

기계 판정 정본은
[`oracle_stage5_queue_projection.json`](../tech/investigations/issue-4963/oracle_stage5_queue_projection.json)이다.
11개 기존 profile의 실제 file SHA-256을 다시 연결했고, 재계측 허용 조건은 입력/font bytes, Oracle 환경
identity, schema/canonicalization 또는 blocked provider의 안전한 inventory·복구 가능성이 달라질 때로
제한했다.

## 3. rank 13 blocked 증거 정규화

rank 13 `휴먼명조`는 관리 related font가 0개인 `none-related` 상태에서도 exact TTF readback이 남았다.
이 관찰은
[`oracle_stage4_rank13_blocked_disposition.json`](../tech/investigations/issue-4963/oracle_stage4_rank13_blocked_disposition.json)으로
경로 없이 정규화했다. 한컴 bundled HFT 또는 관리 범위 밖 provider를 제거해 빈칸을 채우지 않는다.

이 기록은 이전 한컴 build에서 얻은 blocking observation이다. updated-base의 rank 1·7 수치와 직접
비교하지 않고, “관리 집합만으로 missing 상태를 만들 수 없다”는 정지 근거로만 사용한다.

## 4. 남은 두 candidate

### 4.1 rank 16 — 먼저 수행할 무변경 절편

`한컴 윤고딕 230`은 Stage W5-3 selection probe에서 exact TTF readback이 이미 관찰됐지만 PDF profile은
없다. 다음 절편은 updated-base restore 뒤 font 설치·제거 없이 exact-installed fixture를 한 번 export하고
path-free profile로 투영한다. ambient exact provider를 제거해야 하는 나머지 질문은
`blocked-protected-ambient-exact`로 유지한다.

### 4.2 rank 8 — 별도 계약이 필요한 controlled ladder

rank 8 exact face는 `KoPubWorld바탕체 Light`다. W5-4의 공통 substFont도 같은 face였으므로 그대로
재사용하면 `exact-only`와 `subst-only`가 같은 bytes가 되어 독립 상태가 아니다.

권고안은 rank 7에서 공식 bytes와 export를 검증한 `KoPubWorld돋움체 Light`를 rank 8 fixture에만
명시적인 document substitution으로 선언하는 것이다. 둘을 identity·alias·successor로 합치지 않고
fixture-declared relation으로만 다룬다. 이 계약을 승인한 뒤 exact-only, subst-only, none-related 세
상태를 실행한다.

## 5. 다음 게이트

안전성과 비용에 따라 실행 순서는 **rank 16 read-only exact profile → rank 8 distinct-substitution
controlled ladder**로 한다. 이 순서와 rank 8 substitution 계약에 대한 메인테이너 승인 전에는 VM font
상태를 다시 변경하지 않는다.

공개 projection과 blocked disposition에는 raw VM/checkpoint 이름, 절대 경로, font bytes, private corpus
문서·식별자를 포함하지 않는다.

후속 rank 16 기능 탐지 결과와 queue 정정은
[`task_m100_4963_w5_stage5b.md`](task_m100_4963_w5_stage5b.md)에 기록한다.
