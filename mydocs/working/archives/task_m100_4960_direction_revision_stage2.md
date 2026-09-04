---
kind: report
status: active
canonical: mydocs/plans/archives/task_m100_4960.md
last_verified: 2026-08-24
---

# Task M100 #4960 — Stage R2 GitHub 이슈 topology 수정

## 1. 판정

Stage R1에서 승인한 W7 이후 방향 계약을 GitHub issue topology에 적용했다. registry schema 1.0과 제품
source는 바꾸지 않고, W7.5 선행 이슈를 등록해 #4960의 W7과 W8 사이에 배치했다. #4967~#4969는 W5
disposition, W8 qualification과 cohort별 동결 조건을 현재 본문에 반영했다.

본문 4개와 maintainer comment 5개는 게시 뒤 API 원문과 로컬 UTF-8 body를 대사했다. 한글 손상, 선두
BOM, `??` 치환과 literal `\\n`은 0건이다.

## 2. 중복·taxonomy 사전 감사

다음 검색어로 열린·닫힌 이슈와 열린 PR을 확인했다.

- `font rule registry`
- `font registry schema`
- `font rule lifecycle`
- `font rule retirement`

#4939는 historical Font Rule Ledger, #4966은 schema 1.0 canonical projection이다. 추가·수정·retirement의
lifecycle·migration·evidence delta를 소유하는 동일 이슈나 열린 PR은 없었다.

W7 #4966과 같은 taxonomy를 재사용했다.

| metadata | 값 |
| --- | --- |
| assignee | `edwardkim` |
| milestone | `v1.0.0` |
| labels | `enhancement`, `rust`, `rendering`, `typescript` |

## 3. W7.5 이슈와 sub-issue 관계

신규 이슈 [#5955](https://github.com/edwardkim/rhwp/issues/5955)를 생성했다.

- 제목: `[font][W7.5] canonical registry 규칙 생명주기와 evidence delta 계약`
- 상태: OPEN
- 범위: 기존 830개 active rule의 의미 불변 이행, lifecycle, migration manifest와 evidence delta
- 비범위: 개별 face metric·fallback·paint·supply 변경
- issue body SHA-256: `8defc77b2b84faed3c8dfbebcebc3de957163905e5a52fac5846446f0d54ba8c`

#5955를 #4960의 sub-issue로 추가한 뒤 priority API로 #4966 W7과 #4967 W8 사이에 배치했다. 최종 순서는
다음과 같다.

```text
#4961 W2  CLOSED
#4962 W3·W4  CLOSED
#4963 W5  CLOSED
#4964 W6  CLOSED
#4966 W7  CLOSED
#5955 W7.5  OPEN
#4967 W8  OPEN
#4968 W9  OPEN
#4969 W10  OPEN
```

## 4. 기존 issue 본문 수정

| 이슈 | 변경 | body SHA-256 |
| --- | --- | --- |
| [#4960](https://github.com/edwardkim/rhwp/issues/4960) | 단일 임계 경로를 W7.5·W8 qualification·W9 cohort gate로 교체 | `bf7bd30fb65fa7e432f22701e8c586ad4a0171723dbe8dd94bbeb323c5ce8a31` |
| [#4967](https://github.com/edwardkim/rhwp/issues/4967) | 17개 disposition, 세 lane, 자식 생성 조건과 rank 8 process canary 반영 | `f7fdfc575a28ca99136d7538f7268308e208d3242515cb9614313725e0c4e2d4` |
| [#4968](https://github.com/edwardkim/rhwp/issues/4968) | W8 전체 대신 kerning cohort의 겹치는 face 동결을 선행 조건으로 지정 | `c7771d31a3c52222c45b10862a3590b1c8b6b93b1cab1554525c221f4e3d0466` |
| [#4969](https://github.com/edwardkim/rhwp/issues/4969) | W7.5·W9와 대상 fixture face 집합 동결을 선행 조건으로 지정 | `1cbe21c09ac1c63336603cf662db42f1de40adec93beffe837af7211e11711b1` |

적용 직전 `updatedAt`을 Stage R2 사전 조회값과 대사했다. 네 이슈 모두 collaborator의 동시 변경 없이
계획에서 확인한 본문을 기반으로 수정했다.

## 5. maintainer comment

| 이슈 | comment | body SHA-256 |
| --- | --- | --- |
| #4960 | [방향 수정 기록](https://github.com/edwardkim/rhwp/issues/4960#issuecomment-5386984278) | `f639dbba07934f91af359be618664982f4b71d78594cef06ead1b50720cc5b75` |
| #5955 | [W7.5 등록 근거](https://github.com/edwardkim/rhwp/issues/5955#issuecomment-5386984365) | `71480f7c282e5ef2f14ea76470c3d2f86512be69de051384d061c32290938ae6` |
| #4967 | [W8 gate 수정 기록](https://github.com/edwardkim/rhwp/issues/4967#issuecomment-5386984436) | `6b98c16ccaa5c17440917d6a963ca3f98696ac66ee125ca39f4910f4bd4cfa42` |
| #4968 | [W9 gate 수정 기록](https://github.com/edwardkim/rhwp/issues/4968#issuecomment-5386984522) | `3f36494366a1d5ec296f19d01f109adb699e52cc6db7abc0467f1a9581b7edad` |
| #4969 | [W10 gate 수정 기록](https://github.com/edwardkim/rhwp/issues/4969#issuecomment-5386984632) | `b2f600f39a99d6cdb286d57fa1fa1d973e05fdaf178cd12c58d0a4816f697655` |

각 comment는 변경 이유와 범위만 한 번 기록했다. W7.5 구현이나 W8 제품 변경을 착수했다고 주장하지
않는다.

## 6. 보호 불변식 결과

| 불변식 | 결과 |
| --- | --- |
| W0~W7 완료·historical evidence 보존 | 충족 |
| schema 1.0·generated projection 변경 | 0 |
| metric·fallback·paint·font asset 변경 | 0 |
| W5 blocked face 재계측·추정 | 0 |
| private corpus identity·host path·font bytes 공개 | 0 |
| body/comment 한글·BOM·치환 오류 | 0 |
| sub-issue 누락·순서 오류 | 0 |

## 7. 다음 게이트

Stage R2 결과 승인 뒤 Stage R3에서 로컬 문서와 GitHub checklist·선행·후행 관계를 최종 대사하고 PR
제출 구조를 준비한다. 별도 승인 전에는 다음을 수행하지 않는다.

- 이 Stage R2 보고서 commit
- remote branch push와 PR 생성
- #5955 수행계획·schema 구현
- #4967 rank 8 qualification 또는 제품 변경
- #4968·#4969 착수
