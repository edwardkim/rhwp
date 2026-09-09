---
kind: pr_review
status: approved
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-03
pr: 6677
issue: 6628
author: edwardkim
---

# PR #6677 self-review — Gym 오라클·전수 계약 핵심 정상화

## 결론

**승인.** PR #6677은 Gym 기준풀이 통과를 제품 정답으로 확대 해석하던 경계를 바로잡고,
1,035개 task의 정답 권위·기준풀이 출처, 오답 배제력, 수행 경로 필요성을 서로 분리해
결정론적으로 감사한다. Gym은 제품 CI·main 승격·릴리스·게시의 게이트가 아니라 Gym 관련 PR의
빠른 계약과 메인테이너가 명시적으로 시작하는 전수 벤치마크로 운영한다.

최종 code candidate `3eb2e0d5a07bcf94809f05099fd9205af360ef09`에서 로컬 Gym 계약과
전수 원장 감사를 통과했고, GitHub Full CI도 성공 33개, 실패·대기 0개로 완료됐다. self-review
중 발견한 trajectory timeout 분류 불일치는 같은 후보에서 수정·회귀 검증됐다. 이 review와 오늘
작업 기록만 추가하는 후행 head의 review-only checks와 mergeability를 다시 확인한 뒤 정상 merge할
수 있다.

이 문서의 `승인`은 작성자 self-review 판정이다. 자기 PR이므로 reviewer 지정이나 GitHub approve
review event를 만들지 않으며, remote push와 merge는 각각 별도 사용자 승인 게이트다. PR이 병합돼도
부모 #6628은 닫지 않고 sub-issue #6669를 완료한 뒤 최종 정산한다.

## 라우팅과 메타데이터

- 기본 경로: `collaborator_self_merge.md`
- 보조 경로: `intake_and_review.md`, `local_validation.md`, `review_only_fast_pass.md`,
  `rework_and_exceptions.md`
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`와 위 자식 문서
- 시각 검증 보조 경로는 적용하지 않는다. renderer·sample·PDF·golden을 바꾸거나 시각 동등성을
  주장하는 PR이 아니다.
- `review_impl`은 추가하지 않는다. 승인된 정본 계획서가 core 병합 뒤 #6669 독립 수행과 부모
  최종 정산 순서를 이미 고정한다.

| 항목 | 값 |
| --- | --- |
| PR / 작성자 | [#6677](https://github.com/edwardkim/rhwp/pull/6677) / @edwardkim |
| 관련 이슈 | [#6628](https://github.com/edwardkim/rhwp/issues/6628) (`Refs #6628`) |
| 완료된 sub-issue | [#6641](https://github.com/edwardkim/rhwp/issues/6641) / PR #6673 |
| 병합 뒤 수행할 sub-issue | [#6669](https://github.com/edwardkim/rhwp/issues/6669) |
| base | `devel@edeaeb28910f1b84f005aabb4ec0d0f183adc2a1` |
| code candidate | `3eb2e0d5a07bcf94809f05099fd9205af360ef09` |
| 규모 | 188 files, `+5,961/-655`, 22 commits |
| 작성 시점 GitHub 상태 | Open, 비 Draft, `MERGEABLE`, `CLEAN`; 모든 실행 대상 check 완료 |
| reviewer | self PR이므로 지정하지 않음 |

1,000줄을 넘으므로 대형 PR 경로를 적용했다. 변경 수가 큰 이유는 1,035개 task/reference 전수에서
약한 검사와 stale 명령을 실제 위치 단위로 고치고, 각 분류를 재계산할 수 있는 원장·계보 문서를
함께 제출하기 때문이다. authority/task/reference를 일부만 떼면 전수 개수와 원장의 완전성이 깨지므로
논리적으로 원자적이다. Stage별 커밋과 작업 중 최신 `devel`을 보존한 merge commit은 이력을 유지하며,
현재 base는 PR head의 조상이다.

## 코드 검토와 보호 불변식

### 채점 연산자와 schema

- `text_file_envelope_eq`는 현재 rhwp 봉투의 지목 문자열과 제출 파일 전체를 BOM·개행·모든 셀까지
  비교한다. expected/actual 원문은 재출력하지 않고 SHA-256과 UTF-8 바이트 수만 보고한다.
- expected 봉투 텍스트는 8 MiB를 초과하면 실패한다. 제출 파일은 크기가 다르면 읽기 전에 실패하므로
  문자 수 무제한·대형 untrusted content로 인한 메모리 확대를 허용하지 않는다.
- 등록 연산자의 필드 힌트를 schema에 완성했고, 부재 좌표·형식 오류·인코딩 오류를 성공으로 접지
  않는다. 편집 task의 전역 탐색 연산자 금지 계약도 유지한다.

### 기준풀이와 trajectory

- `build_baseline.py`는 기준풀이의 `{file:}` 사용을 거부하고 제출 폴더 내부 `{sub:}`만 허용한다.
  missing reference·missing artifact·failed score·build error는 JSON 결과 행과 0이 아닌 종료 코드로
  보존한다.
- `--json` stdout에는 고정 kind/schemaVersion과 전 task 결과만 출력하고 기존 진행 로그는 stderr로
  분리한다. task가 없거나 하나라도 skip/fail이면 `ok=false`다.
- trajectory는 마지막 의미 step을 제거한 뒤 종점 check만 직접 호출하지 않는다. 기준풀이 검증과
  같은 `inspect_built_task()`로 선언 산출물 부재를 먼저 확인해, check 명령이 빠진 산출물을 다시
  만들거나 오류 봉투 일부만 보고 통과하는 경로를 막는다.
- self-review에서 `TimeoutError`가 중간에는 `timeout`이지만 최종 카탈로그 밖이라 `unexpected`로
  접히는 불일치를 발견했다. `timeout`을 고정 카탈로그에 넣고 타입 매핑의 모든 출력이 카탈로그에
  속하는지 회귀 검사를 추가했다. 실패 판정은 계속 fail-closed이며 #6669가 원인별 집계를 잃지 않는다.

### authority ledger와 운영 경계

- 원장은 task check가 현재 rhwp를 호출하면 무조건 `self-live`로 분류하고, 명시 선언으로
  `independent-fixture`나 `external-oracle`로 승격하지 못하게 한다. 분류마다 source path와 JSON
  pointer를 남겨 집계가 아니라 원문에서 재계산할 수 있다.
- task/reference 짝, ID, check op, live command, evidence 경로, baseline source, entry/summary 개수를
  fail-closed로 감사한다. 현재 결과는 1,035 task/reference/entry와 issue 0이다.
- `.github/workflows/gym-release-gate.yml`은 `Gym Benchmark Validation`으로 역할을 바꾸고 Gym 관련
  PR의 portable 계약만 자동 실행한다. 전수 positive/discrimination/trajectory와 JSON artifact는
  수동 `workflow_dispatch`에서만 생성한다.
- devel/main push와 `v*` tag trigger를 제거했고 Release Binary·npm·extension 게시 artifact와 연결하지
  않는다. CI classifier는 `gym/**`와 `test_gym_*.py`를 별도 benchmark 영역으로 인식하되 workflow
  자체 변경은 보수적으로 Full CI를 유지한다.

코드 diff에서 제품 검증기 완화, release workflow 우회, Rust 제품 변경 중복, 사설 코퍼스·비밀값,
생성 submission이나 임시 전수 산출물은 발견되지 않았다. 새 blocker도 발견되지 않았다.

## 로컬·전수 검증

[최종 보고서](../../report/task_m100_6628_report.md),
[Stage 4 전수 판정](../../working/task_m100_6628_stage4.md),
[Stage 6 제출 감사](../../working/task_m100_6628_stage6.md)에 다음 결과를 고정했다.

| 검증 | 결과 |
| --- | --- |
| 전체 Gym Python discover | 3,151건 통과, 정책상 skip 1건 |
| workflow exact Gym contracts | 2,125건 통과, 정책상 skip 1건 |
| workflow Python 계약 | 148건 통과 |
| CI classifier/policy Node 계약 | 78건 통과 |
| Gym audit | 21 pack, 1,035 task, issue 0 |
| authority ledger | 1,035/1,035/1,035, issue 0 |
| authority 분포 | self-live 987, contract-constant 28, independent-fixture 20, external-oracle 0 |
| baseline source | self-live 1,031, contract-constant 4 |
| exact product candidate positive | 1,035/1,035, fail·skip 0 |
| discrimination | 1,035 task, 1,511 control, false-pass 0 |
| trajectory | 239/239 load-bearing, theater·예외·tool 오류 0 |
| 변경 Markdown 링크·`git diff --check` | 통과 |

전수 세 축은 #6641의 exact product candidate `7f1174f1d59bc020aaa38ceb7e148a8ae77b2784`와
runner `374c7416a6c2d9abe7c2701969de5f377b71183f`에서 실행했다. binary SHA-256은
`4334e35e3bfb7e892416ee663c7a52dd055a7d82040d2a89447f32d25ccd02f34`다. 현재 code
candidate까지 관련 실행 source diff가 0임을 대조했으므로 동일한 38분 전수 실행을 중복하지 않았다.

## GitHub Full CI

code candidate `3eb2e0d5a07bcf94809f05099fd9205af360ef09`에서 실행 대상 check 33개가 모두
성공했고, 조건상 비대상 5개만 skip됐다.

- [CI 33711810184](https://github.com/edwardkim/rhwp/actions/runs/33711810184): preflight,
  Lint, Native Skia, frontend package, 네 architecture build/archive와 Build & Test 성공
- [CodeQL 33711810101](https://github.com/edwardkim/rhwp/actions/runs/33711810101): Rust 15분 6초,
  Python, JavaScript/TypeScript 분석과 최종 집계 성공
- [Gym Benchmark Validation 33711809945](https://github.com/edwardkim/rhwp/actions/runs/33711809945):
  Gym benchmark contracts 성공, 수동 전수 job은 조건상 skip
- [Render Diff 33711809932](https://github.com/edwardkim/rhwp/actions/runs/33711809932): preflight와
  Canvas diff 성공
- [Proptest 33711810124](https://github.com/edwardkim/rhwp/actions/runs/33711810124): preflight와
  prop roundtrip 성공
- [Adapter inter-diff 33711810106](https://github.com/edwardkim/rhwp/actions/runs/33711810106): preflight와
  본 검사 성공
- [CI Impact Policy Controller 33711808523](https://github.com/edwardkim/rhwp/actions/runs/33711808523):
  `mode=full`, `rfp=0`, workflow/CI surface 포함 판정과 최종 정책 성공

## 시각 검증 판정

시각 검증은 요구하지 않는다. 이 PR은 renderer/layout, 제품 출력, sample·기준 PDF·golden을 바꾸지
않고 Gym 내부 채점·감사·운영 경계를 수정한다. 따라서 판정 근거는 특정 페이지의 pixel fidelity가
아니라 task/reference 전수 완전성, 양성·음성·경로 감사와 fail-closed JSON/종료 코드다. Render Diff가
Full CI에서 통과했지만 이를 한컴 조판 동등성이나 일반 renderer 개선의 증거로 확대 해석하지 않는다.

## 잔여 위험과 후속 경계

- external oracle은 0개다. 현재 결과는 공개 Gym 자체의 정합성과 판별력 증거이지 한컴 조판 동등성의
  독립 증명이 아니다.
- API 호출 순서 교환을 검사하는 order-dependency audit은 부모 이슈에 기록된 후속 백로그이며
  PR #6677 범위가 아니다.
- JSON 정본을 사람이 한눈에 읽을 deterministic offline 시각화는 sub-issue #6669에서 구현한다.
  시각화는 원본 JSON을 바꾸거나 제품 release gate가 되지 않는다.
- 대형 PR이므로 review 기록만 추가한 trailing head라도 trusted review-only 판정, 최신 base 관계와
  mergeability를 다시 확인한다. 자동 merge나 admin 우회는 사용하지 않는다.

## 최종 판정과 다음 조건

- 판정: **승인**
- 판정 대상: code candidate `3eb2e0d5a07bcf94809f05099fd9205af360ef09`
- trailing 조건: 이 review·오늘할일만 추가한 최신 head에서 review-only checks 성공,
  `MERGEABLE`·`CLEAN` 및 최신 base 재확인
- merge 조건: 최신 head SHA 고정, 사용자 merge 승인, `--admin` 우회 없는 정상 2-parent merge commit
- GitHub review: self PR이므로 approve event와 reviewer 지정 없음
- merge 후: #6628은 OPEN으로 유지하고 최신 `devel`에서 #6669 독립 브랜치를 시작한다. #6669
  병합·종료 뒤 부모 완료 조건과 후속 백로그를 최종 정산한다.
