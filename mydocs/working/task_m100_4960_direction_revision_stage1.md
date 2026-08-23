---
kind: report
status: active
canonical: mydocs/plans/task_m100_4960.md
last_verified: 2026-08-24
---

# Task M100 #4960 — Stage R1 W7 이후 방향 계약 문서화

## 1. 판정

#4960 수정 수행계획의 Stage R1 범위인 로컬 권위 문서 현행화를 완료했다. W0~W7의 완료 결과와
historical 수치·hash는 보존하고, W7 이후의 직선 경로를 다음 세 게이트로 수정했다.

1. 제품 규칙 변경 전 W7.5 registry evolution contract
2. W5 disposition을 실제 제품 delta 가설로 바꾸는 W8 correction qualification
3. W8 tracker 전체가 아닌 관련 metric·face를 동결하는 W9 kerning cohort gate

이번 단계는 Markdown만 변경했다. registry schema 1.0, generated projection, metric·fallback·paint,
font asset과 renderer output은 변경하지 않았다.

## 2. 근거 대사

| 근거 | Stage R1 해석 |
| --- | --- |
| W3·W4 54,326,042 coverage 문자, 위험 face 351개 | A/B band는 유지하고 exact rank를 강제 구현 순서로 쓰지 않음 |
| W5 `complete-acceptance-ladder` 3개 | rank 1·7·8을 qualification 가능 후보로 유지 |
| W5 source unavailable 10개 | source discovery 사건 전 반복 계측·제품 이슈 생성 금지 |
| W5 protected partial 3개 | provider를 손상하지 않는 새 제어 능력이 생길 때만 재개 |
| W5 capability mismatch 1개 | localized document face와 export subset 연결 전 exact 승격 금지 |
| W7 registry 830 active rule | schema 1.0은 read-only로 보존하고 다음 판을 별도 승인 |

W5의 `actionableRanks=[]`는 추가 Oracle 실행 queue가 끝났다는 뜻으로 유지했다. W8 후보가 0이라는
주장이나 17개 모두가 구현 준비됐다는 주장으로 바꾸지 않았다.

## 3. 문서별 변경

### 원인 계보 보고서

[`font_metrics_fallback_causal_lineage_20260816.md`](../report/font_metrics_fallback_causal_lineage_20260816.md)는
W0~W7의 역사와 FI-01~FI-14를 그대로 유지하고 다음만 현행화했다.

- 단일 임계 경로를 W7.5·W8 qualification·kerning cohort 분기 graph로 교체
- W7 schema 1.0 read-only 경계와 W7.5 입력·산출물·완료 조건 추가
- W8을 evidence reopen·qualification·product correction lane으로 분리
- W9를 장기 W8 전체가 아닌 겹치는 face 동결 조건으로 변경
- W10을 W9 shaping 경계와 대상 fixture face 집합 안정화에 연결
- 우선순위를 완료 W0~W7과 현재 P0~P3으로 재기록

### W7 최종 보고서

[`task_m100_4966_report.md`](../report/task_m100_4966_report.md)는 다음 현재 사실을 반영했다.

- `status: completed`, canonical self-reference, `last_verified: 2026-08-24`
- PR #5950 일반 merge와 merge commit `5057a7fcaf055b928e76115cdee4bc20bf0936f9`
- Issue #4966 자동 종료
- W8 직접 진입 대신 W7.5의 의미 불변 schema 이행 선행
- W9의 kerning cohort 동결 조건

### canonical fallback 전략

[`font_fallback_strategy.md`](../tech/font_fallback_strategy.md)는 schema 1.0의 운영 경계를 장기 정책에
반영했다.

- schema 1.0 JSON과 generated projection 직접 수정 금지
- 다음 schema 판, migration manifest, lifecycle과 semantic delta 선행
- W1 historical snapshot 대신 새 evidence parent·digest 연결
- retirement rule을 삭제하지 않고 trace·감사 계보에 보존

## 4. 수정된 의존성

```text
W0~W7 완료
  → W7.5 registry evolution
      ├─ W8 correction qualification → face별 product correction
      └─ kerning cohort metric·face freeze → W9 → W10

새 source·provider·identity evidence
  → 해당 W5 disposition만 재개
  → W8 qualification
```

W8 face correction이 kerning cohort와 겹치지 않으면 W9는 다른 blocked face의 source 발견을 기다리지
않는다. 겹치면 해당 face의 correction 완료 또는 no-change disposition까지 동결한다.

## 5. 변경 범위 감사

| 항목 | 결과 |
| --- | --- |
| 계획서 | `mydocs/plans/task_m100_4960.md` 신규 |
| 기존 조사·정책 문서 | 3개 수정 |
| Stage 보고서 | 이 문서 신규 |
| Rust·TypeScript·JavaScript source | 변경 0 |
| registry·projection·metric data | 변경 0 |
| fixture·font asset | 변경 0 |
| private corpus·host path·font bytes 공개 | 0 |
| GitHub mutation | 0 |

## 6. 검증 결과

| 검사 | 결과 |
| --- | --- |
| 변경 5개 Markdown 링크 | 통과, 내부 상대 링크 이상 0 |
| `git diff --check` | 통과 |
| 변경 범위 | 계획·조사·정책·보고 Markdown 5개뿐 |
| 저장소 전체 metadata | 기존 4개 문서의 누락 16건으로 실패, 이번 변경 경로 신규 오류 0 |

기존 metadata 오류는 `mydocs/tech/benchmark_vs_alternatives.md`,
`mydocs/tech/investigations/issue-4964/README.md`, `mydocs/tech/investigations/issue-5511/README.md`와
`mydocs/tech/investigations/issue-5511/task_m100_5511_cli_surface_inventory.md`의 필수 필드 누락이다.
이번 Stage R1에서 만든 오류가 아니며 범위를 넓혀 임의 정정하지 않았다.

commit·push 직전 필수 format gate는 별도 승인 뒤 같은 workspace 범위로 실행한다.

## 7. 다음 게이트

메인테이너가 Stage R1 결과를 승인하면 다음 단계는 Stage R2 GitHub topology 수정이다. 별도 원격
mutation 승인 전에는 다음을 수행하지 않는다.

- W7.5 이슈 생성과 metadata 지정
- #4960·#4967~#4969 본문·sub-issue·댓글 변경
- commit, remote push 또는 PR 생성
- W7.5 schema 구현이나 W8 rank 8 qualification
