---
kind: report
status: active
canonical: mydocs/plans/archives/task_m100_4960.md
last_verified: 2026-08-24
---

# Task M100 #4960 — Stage R3 정합성 감사와 PR 준비

## 1. 판정

W7 이후 로드맵의 로컬 정본과 GitHub issue topology는 같은 실행 graph를 표현한다. W0~W7의 완료
판정과 역사 증적은 유지됐고, W7.5는 미착수 선행 gate, W8~W10은 미착수 후행 작업으로 남아 있다.
이번 branch에는 Markdown 7개 외의 제품 source·registry·metric·fixture·asset 변경이 없다.

상위 #4960은 W7.5~W10을 계속 추적해야 하므로 이 문서 PR이 병합돼도 닫지 않는다. PR 관련 이슈는
`Refs #4960`으로 기록한다.

## 2. Git 기준선과 제출 범위

2026-08-24 KST에 `git fetch upstream devel --prune` 뒤 확인한 기준은 다음과 같다.

| 항목 | 값 |
| --- | --- |
| base | `upstream/devel@4e070c632dfc889f329be6a90ef2c1d35f8a7f12` |
| Stage R1 commit | `215d56f66` |
| Stage R2 commit | `ce374f29a` |
| divergence | base보다 behind 0, ahead 2 |
| merge risk | base 이동 없음, 현재 commit graph에 재동기화 필요 없음 |
| remote head | 아직 없음 |
| 기존 PR | 없음 |

제출 diff는 계획서, 원인 계보 보고서, W7 최종 보고서, canonical fallback 전략과 Stage R1~R3 증적
문서뿐이다. Rust·TypeScript·workflow·JSON registry·generated projection·font·sample·baseline은 0건이다.

## 3. 로컬 정본과 GitHub 상태 대사

최종 sub-issue 순서와 상태는 다음과 같다.

```text
#4961 W2     CLOSED
#4962 W3·W4  CLOSED
#4963 W5     CLOSED
#4964 W6     CLOSED
#4966 W7     CLOSED
#5955 W7.5   OPEN
#4967 W8     OPEN
#4968 W9     OPEN
#4969 W10    OPEN
```

- W7.5는 schema 1.0을 직접 수정하지 않고 lifecycle·migration·evidence delta를 준비한다.
- W8은 evidence-reopen, correction-qualification과 product-correction lane을 분리한다.
- W9는 W8 전체가 아니라 kerning cohort와 겹치는 face의 metric·selection 동결을 요구한다.
- W10은 W9와 대상 fixture face 집합의 metric·identity 동결을 요구한다.
- rank 8은 위험 순위 변경이나 구현 확정이 아니라 W8 qualification의 process canary 후보다.

W5 정본도 다시 대사했다. `oracle_stage5_queue_projection.json`은 `stage=W5-5C`,
`actionableRanks=[]`이며 SHA-256은
`7765e060982c672cac8fbd0700f73e21d7488ae2fb25144c8046a4e678e0002d`다. rank 8 acceptance ladder의
SHA-256은 `d6e8a4371dd049a899a88fb975d6499ed435154b72e1fc804b701addf3cb75ec`다. 기존 17개 disposition과
rank 1·7·8의 `complete-acceptance-ladder` 판정은 다시 쓰지 않았다.

## 4. GitHub 원문 재검증

Stage R2 뒤 각 issue와 comment를 REST API로 다시 읽었다. 아래 SHA-256은 게시된 body의 UTF-8 byte
기준이며 Stage R2 기록과 전부 일치한다.

| 이슈 | 상태 | body SHA-256 |
| --- | --- | --- |
| [#4960](https://github.com/edwardkim/rhwp/issues/4960) | OPEN | `bf7bd30fb65fa7e432f22701e8c586ad4a0171723dbe8dd94bbeb323c5ce8a31` |
| [#5955](https://github.com/edwardkim/rhwp/issues/5955) | OPEN | `8defc77b2b84faed3c8dfbebcebc3de957163905e5a52fac5846446f0d54ba8c` |
| [#4967](https://github.com/edwardkim/rhwp/issues/4967) | OPEN | `f7fdfc575a28ca99136d7538f7268308e208d3242515cb9614313725e0c4e2d4` |
| [#4968](https://github.com/edwardkim/rhwp/issues/4968) | OPEN | `c7771d31a3c52222c45b10862a3590b1c8b6b93b1cab1554525c221f4e3d0466` |
| [#4969](https://github.com/edwardkim/rhwp/issues/4969) | OPEN | `1cbe21c09ac1c63336603cf662db42f1de40adec93beffe837af7211e11711b1` |

| 이슈 | comment | body SHA-256 |
| --- | --- | --- |
| #4960 | [방향 수정 기록](https://github.com/edwardkim/rhwp/issues/4960#issuecomment-5386984278) | `f639dbba07934f91af359be618664982f4b71d78594cef06ead1b50720cc5b75` |
| #5955 | [W7.5 등록 근거](https://github.com/edwardkim/rhwp/issues/5955#issuecomment-5386984365) | `71480f7c282e5ef2f14ea76470c3d2f86512be69de051384d061c32290938ae6` |
| #4967 | [W8 gate 수정 기록](https://github.com/edwardkim/rhwp/issues/4967#issuecomment-5386984436) | `6b98c16ccaa5c17440917d6a963ca3f98696ac66ee125ca39f4910f4bd4cfa42` |
| #4968 | [W9 gate 수정 기록](https://github.com/edwardkim/rhwp/issues/4968#issuecomment-5386984522) | `3f36494366a1d5ec296f19d01f109adb699e52cc6db7abc0467f1a9581b7edad` |
| #4969 | [W10 gate 수정 기록](https://github.com/edwardkim/rhwp/issues/4969#issuecomment-5386984632) | `b2f600f39a99d6cdb286d57fa1fa1d973e05fdaf178cd12c58d0a4816f697655` |

본문과 comment에 선두 BOM, `??` 치환, literal `\\n`, private corpus identity·host path와 font bytes는
없다.

## 5. 검증 결과

| 검사 | 결과 |
| --- | --- |
| 변경 Markdown 상대 링크 | 통과 |
| `git diff --check` | 통과 |
| 변경 범위 | `mydocs/**`만 존재 |
| generated integration manifest | 검토 worktree에서 893 sources·4165 static test attrs·32 suites+9 exceptions 확인 |
| `cargo fmt --all`·`cargo fmt --all -- --check` | 검토 worktree의 `ce374f29a`에서 통과 |
| 문서 metadata 전수 검사 | 이번 변경 밖 4개 문서의 기존 오류 16건만 재현 |
| product build·test·visual sweep | 문서 전용 변경이므로 비대상 |

기존 metadata 오류는 `mydocs/tech/benchmark_vs_alternatives.md`,
`mydocs/tech/investigations/issue-4964/README.md`, `mydocs/tech/investigations/issue-5511/README.md`와
`mydocs/tech/investigations/issue-5511/task_m100_5511_cli_surface_inventory.md`의 필수 front matter
누락이다. 이번 diff에서 만들거나 수정한 오류가 아니므로 범위를 확장해 고치지 않았다.

## 6. PR 제출 초안

- 제목: `docs(font): #4960 W7 이후 로드맵을 재정렬한다`
- base: `devel`
- head: `task_m100_4960_direction_revision`
- 관련 이슈: `Refs #4960`
- 성능 영향: 문서·tracker 정정만 포함하므로 제품 성능 영향 없음

본문 핵심은 다음과 같다.

```markdown
## 변경 요약

- W7 schema 1.0과 실제 face 교정 사이에 W7.5 registry lifecycle gate를 추가했습니다.
- W8을 evidence 재개·qualification·제품 교정 lane으로 분리했습니다.
- W9·W10을 장기 W8 전체가 아니라 대상 cohort·face 동결 조건에 연결했습니다.
- GitHub sub-issue 순서와 본문을 같은 실행 graph로 현행화하고 검증 해시를 기록했습니다.

## 관련 이슈

Refs #4960

## 테스트

- [x] 변경 Markdown 링크 검사
- [x] `git diff --check`
- [x] 변경 범위가 `mydocs/**`뿐임을 확인
- [x] `cargo fmt --all -- --check` — generated suite를 준비한 검토 worktree에서 통과
- [x] GitHub issue·comment body 해시와 sub-issue 순서 재검증
- [ ] Cargo test·clippy·WASM·시각 검증 — 문서 전용 변경이므로 비대상

## 성능 영향 및 측정 결과

- 예상 영향: 없음
- 재현·측정: 제품 source와 runtime artifact 변경 0건

## 스크린샷

해당 없음
```

remote branch push와 `gh pr create`는 각각 별도 메인테이너 승인 뒤 수행한다.

## 7. 보호 불변식과 다음 게이트

- #4960은 닫지 않는다.
- #5955 수행계획·schema 구현을 시작하지 않는다.
- #4967 rank 8 qualification이나 제품 변경을 시작하지 않는다.
- #4968·#4969를 시작하지 않는다.
- remote push와 PR 생성은 자동 수행하지 않는다.

Stage R3 결과 승인 뒤 이 보고서와 계획 상태를 별도 commit으로 고정한다. 그 다음 최신
`upstream/devel`을 다시 fetch해 exact base를 확인하고, remote push 승인과 PR 생성 승인을 차례로 받는다.
