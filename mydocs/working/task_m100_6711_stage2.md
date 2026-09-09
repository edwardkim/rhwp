---
kind: snapshot
status: active
canonical: mydocs/plans/task_m100_6711.md
issue: 6711
last_verified: 2026-09-04
---

# #6711 Stage 2-A — orders/plans 월별 archive 이동

## 1. 결과

Stage 1 dry-run에서 원래 첫 PR 범위인 `orders/plans/pr/report`의 보수적 변경 경로가 3,394개로
측정됐다. GitHub PR file API의 3,000개 한도를 넘으므로 승인된 fail-closed 규칙에 따라 다음처럼
분리했다.

| batch | 이동 후보 | rename-aware 예상 | 보수적 예상 |
| --- | ---: | ---: | ---: |
| Stage 2-A `orders/plans` | 773 | 1,074 | 1,730 |
| Stage 2-B `pr/report` | 832 | 1,012 | 1,741 |

현재 브랜치에는 Stage 2-A만 적용했다. 추가 이슈나 자식 이슈는 만들지 않았다.

## 2. 이동 결과

| 구분 | 수 |
| --- | ---: |
| 이동 후보 | 773 |
| Git rename으로 이동 | 771 |
| 동일 archive가 있어 root만 제거 | 2 |
| 서로 다른 충돌을 suffix 경로로 보존 | 2 |
| root에 남은 cutoff 이전 문서 | 0 |

root 결과:

- `mydocs/orders/*.md`: 4개, 모두 9월 생성
- `mydocs/plans/*.md`: 기준선 9월 문서 12개 + #6711 계획서 1개

동일본 중복은 다음 두 root만 제거하고 기존 archive bytes를 유지했다.

- `mydocs/plans/task_m100_1363.md`
- `mydocs/plans/task_m100_1363_v2.md`

서로 다른 문서는 기존 archive를 덮어쓰지 않고 다음 경로에 보존했다.

- `mydocs/plans/archives/task_m100_1880_archived_20260705_fb8827e.md`
- `mydocs/plans/archives/task_m100_2214_archived_20260712_60d8480.md`

## 3. 링크와 canonical 갱신

이동 전에 모든 변경 내용을 메모리에서 계산하고 다음 조건을 확인한 뒤 파일을 옮겼다.

1. 773개 source가 모두 존재하고 목적지가 충돌 원장과 일치
2. 기존에 유효한 링크의 논리 target이 이동 뒤에도 존재
3. relative link는 이전 source에서 해석한 target을 새 source 기준으로 재계산
4. byte-identical 중복은 기존 archive를 canonical content로 유지

적용 결과:

- link destination 재계산: 948개
- 전체 수정(M) 문서: 281개 — 링크·canonical 갱신과 거버넌스·오늘할일 포함
- `canonical:` 이동 경로 갱신: 118개 문서
- `mydocs` 밖 incoming link 갱신: 2개
  - `npm/hwpctrl-ocx/README.md`
  - `tools/hwpctrl_compat/README.md`

변경 문서 검사가 기존에 숨어 있던 잘못된 Markdown 해석 10건도 노출했다.

- `셀[11](r=4)`, `record[0](BOTH)`처럼 설명 괄호가 링크로 해석되던 표현 5건
- 존재하지 않는 옛 source를 현재 링크로 표현한 2건
- 이미 제거된 과거 release guide를 현재 링크로 표현한 2건
- root 기준 상대 깊이가 처음부터 잘못된 `rhwp-studio` 링크 1건

오인식 표현은 일반 문장으로 바꾸고, 사라진 과거 경로는 `당시` inline code로 보존했으며, 실제
존재하는 파일은 정확한 현재 상대 경로로 연결했다.

## 4. 오류 집합 비교

대량 이동 직후 전체 저장소 Markdown을 비교했을 때 정규화된 깨진 링크 집합은 이동 전후 모두
560건으로 같았고 신규 오류는 0건이었다. 이후 변경 문서 전용 gate가 노출한 앞의 10건을 정정해
다섯 대상 폴더의 historical 전수 오류는 526건에서 516건으로 줄었다.

- canonical 기본 링크 검사: 오류 0건
- `--changed-from upstream/devel --forbid-redirect-references`: 오류 0건
- metadata: 기존 오류 16건, 신규 오류 0건
- `git diff --check`: 통과

기존 metadata 16건은 `mydocs/tech`의 선행 누락이며 이번 이동 대상이 아니다.

## 5. PR 크기와 범위

Stage 2-A 보고서 추가 전 staged 실측:

| 측정 | 파일 수 |
| --- | ---: |
| rename-aware | 1,056 |
| rename 비의존 old/new 경로 합산 | 1,827 |
| Git이 인식한 rename | 771 |
| 동일본 root delete | 2 |

Stage 2 보고서 한 파일이 추가되어도 보수적 수는 1,828개로 3,000개보다 충분히 작다.

이동은 `mydocs/orders`, `mydocs/plans`에 한정했다. 두 외부 README와 `feedback/report/tech/working`
변경은 이동된 계획서로 들어오는 Markdown 링크 또는 `canonical:` target 갱신뿐이다. Rust source,
Cargo, WASM, workflow는 변경하지 않았다.

## 6. 다음 절차

1. 최종 문서-only 검증과 staged scope를 다시 확인한다.
2. Stage 2-A commit을 만든다.
3. 메인테이너 승인 뒤 원격 push와 PR 생성을 수행한다.
4. PR merge와 최신 `devel` 동기화 뒤 Stage 2-B `pr/report`를 별도 branch에서 시작한다.
