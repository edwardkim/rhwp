# Task M100 #6628 — Gym 오라클·전수 계약 핵심 정상화 보고서

- **Issue**: #6628
- **브랜치**: `task_m100_6628`
- **최신 devel 기준**: `upstream/devel@edeaeb28910f1b84f005aabb4ec0d0f183adc2a1`
- **동기화 merge**: `3950ca15738311ef23e87233d981dd2d6197b953`
- **핵심 구현 병합**: PR #6677 / `d770ef80ed5ccc82a834558355b6786213ca2e05`
- **보고일**: 2026-09-03 KST
- **판정**: `core-merged; ready-to-close-after-6669-merge`

## 1. 결론

Gym의 기준풀이가 통과하는지만 보던 상태를 정답 권위, 오답 배제력, 수행 경로 필요성을 서로
분리해 감사하는 구조로 정상화했다. 같은 rhwp가 만든 결과를 외부 정답으로 오인하지 않도록
1,035개 task의 권위를 전수 분류했고, 알려진 19 task·28 false-pass를 포함한 약한 oracle을
검사를 느슨하게 하지 않고 제거했다.

제품 결함으로 남았던 BO05·BO15는 별도 sub-issue #6641과 PR #6673에서 수정했다. 그 exact
제품 후보로 positive 1,035/1,035, discrimination false-pass 0, trajectory 239/239를 통과했다.
Gym은 일반 제품 CI·main 승격·릴리스·게시를 막지 않으며, Gym 관련 PR의 빠른 계약과 사람이
명시적으로 시작하는 전수 벤치마크로 운영한다.

핵심 구현 PR #6677은 `devel`에 병합됐다. 마지막 sub-issue #6669의 JSON 증적 시각화도 PR #6686의
code candidate `c5efd5240e79a8c862df859dd6a505c55031e8ab`에서 구현·검증·self-review 승인을 마쳤다.
따라서 부모 #6628의 기술적 완료 조건은 충족됐으며, 남은 운영 조건은 PR #6686 정상 병합과
#6669 종료 확인 뒤 부모를 최종 close하는 것이다.

## 2. 주요 결과

- authority ledger: task/reference/entry 1,035/1,035/1,035, issue 0
- authority: self-live 987, contract-constant 28, independent-fixture 20, external-oracle 0
- baseline source: self-live 1,031, contract-constant 4
- positive: 21 pack, 1,035/1,035, 실패·skip·누락·score/build 오류 0
- discrimination: 1,035 task, 1,511 control, false-pass 0
- trajectory: 239/239 load-bearing, theater·예외·tool 오류 0
- 현재 tree 빠른 계약: Python 3,151건, exact Gym workflow 2,125건,
  workflow Python 148건, CI classifier/policy Node 78건 통과

external oracle은 아직 0개다. 따라서 이 결과는 공개 Gym 자체의 정합성과 판별력 증거이지,
한컴 조판 동등성이나 제품 릴리스 적합성의 독립 증명이 아니다.

## 3. 구현 범위

### 정답 권위와 채점

- `authority_ledger.py`와 기계 판독 원장 계약을 추가했다.
- 파일 존재·크기·동일 해시처럼 일을 하지 않은 제출을 허용한 검사를 전체 의미 검사로 바꿨다.
- untrusted 텍스트에는 크기 상한과 digest 보고를 적용했다.
- 오류 봉투, 빈 artifact, missing artifact, 잘못된 인코딩을 성공으로 접지 않는다.
- `TimeoutError`의 `timeout` 분류가 최종 예외 행까지 보존되며, 예외 타입 매핑이 카탈로그 밖
  값을 만들지 못하도록 계약화했다.

### 기준풀이와 trajectory

- 현재 CLI와 어긋난 reference 명령과 결과 좌표를 현행화했다.
- 다단계 reference의 마지막 의미 step을 제거하는 necessity audit을 전수 정산했다.
- API 호출 순서를 서로 바꾸는 order-dependency audit은 부모 이슈의 명시적 후속 백로그이며
  이번 제출 범위가 아니다.

### 실행·운영 경계

- workflow 표시 이름을 `Gym Benchmark Validation`으로 바꿨다.
- Gym 관련 PR은 구조·단위 계약만 자동 실행한다.
- 전수 positive/discrimination/trajectory는 `workflow_dispatch` 또는 메인테이너 로컬 실행으로만 둔다.
- devel/main push, v* tag, Release Binary, npm·extension 게시와 Gym artifact를 분리했다.
- 인간 개발자용 수동 운영 매뉴얼과 `gym/**` 범위 AI 에이전트 지침을 추가했다.

## 4. 검증과 증적 계보

전수 증적은 #6628 runner `374c7416a`와 #6641 product candidate `7f1174f1d`를 분리해
실행했다. binary SHA-256은
`4334e35e3bfb7e892416e663c7a52dd055a7d82040d2a89447f32d25ccd02f34`다.
현재 최신 devel까지 관련 실행 source diff가 0이고, 부모 브랜치 병합 뒤 빠른 계약을 다시
통과했으므로 전수 실행을 중복하지 않았다.

상세 근거:

- [Stage 4 전수 판정](../working/task_m100_6628_stage4.md)
- [Stage 6 제출 감사](../working/task_m100_6628_stage6.md)
- [#6641 제품·Gym 인계 보고](task_m100_6641_report.md#5-6628-gym-인계-전수)
- [Gym 벤치마크 수동 운영 매뉴얼](../manual/gym_benchmark_operations.md)

## 5. 잔여 경계와 부모 이슈 상태

JSON 판정 정본을 사람이 대조할 파생 보고서는 sub-issue #6669와 PR #6686에서 구현했다. 정확히
41개 입력을 fail-closed로 검증·seal하고, 원문 SHA-256과 실행 신원을 고정한 뒤 JavaScript·외부
resource가 없는 결정론적 HTML을 생성한다. HTML은 정답이나 판정을 새로 만들지 않는 비권위 뷰이며,
PASS/FAIL/INCOMPLETE와 trajectory `ok`/`trusted`, single-step N/A를 축약하지 않는다.

PR #6686의 검증 결과는 focused 20/20, 전체 Gym 빠른 계약 3,171건과 조건부 skip 1건, 구조 audit
21 pack·1,035 task/reference·issue 0, oracle selftest 14건·실패 0, authority ledger
1,035/1,035/1,035·issue 0이다. 공개 fixture manifest와 HTML은 byte-identical 재생성이 가능하고,
host Chrome desktop·mobile·offline·접근성 판정을 통과했다. 상세 근거는
[#6669 최종 보고서](task_m100_6669_report.md)와
[#6686 self-review](../pr/archives/pr_6686_review.md)에 있다.

2026-09-03 종료 감사 시점의 최신 `upstream/devel`은
`bd72886c02d301ff796b6b5c55a452a870cf317a`다. PR #6686과 새 renderer PR #6591의 변경 경로는
겹치지 않고 exact current-base merge-tree도 성공했다. 현재 sub-issue는 #6641 closed, #6669 open이며,
부모 #6628은 PR #6686 병합과 #6669 자동 close를 확인할 때까지 open으로 유지한다.

AI 에이전트 온보딩 문서 품질 후속 #6684는 독립 backlog다. 이 후속은 #6628의 완료 범위를 자동으로
확장하지 않으며 부모 종료를 막지 않는다.
