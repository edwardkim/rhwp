# Task M100 #6628 — Gym 오라클·전수 계약 핵심 정상화 보고서

- **Issue**: #6628
- **브랜치**: `task_m100_6628`
- **최신 devel 기준**: `upstream/devel@edeaeb28910f1b84f005aabb4ec0d0f183adc2a1`
- **동기화 merge**: `3950ca15738311ef23e87233d981dd2d6197b953`
- **보고일**: 2026-09-03 KST
- **판정**: `ready-for-core-pr; parent-remains-open-for-6669`

## 1. 결론

Gym의 기준풀이가 통과하는지만 보던 상태를 정답 권위, 오답 배제력, 수행 경로 필요성을 서로
분리해 감사하는 구조로 정상화했다. 같은 rhwp가 만든 결과를 외부 정답으로 오인하지 않도록
1,035개 task의 권위를 전수 분류했고, 알려진 19 task·28 false-pass를 포함한 약한 oracle을
검사를 느슨하게 하지 않고 제거했다.

제품 결함으로 남았던 BO05·BO15는 별도 sub-issue #6641과 PR #6673에서 수정했다. 그 exact
제품 후보로 positive 1,035/1,035, discrimination false-pass 0, trajectory 239/239를 통과했다.
Gym은 일반 제품 CI·main 승격·릴리스·게시를 막지 않으며, Gym 관련 PR의 빠른 계약과 사람이
명시적으로 시작하는 전수 벤치마크로 운영한다.

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

JSON은 판정 정본이지만 사람이 네 축과 오류·예외·계보를 한눈에 대조할 파생 보고서는 아직 없다.
이 산출물은 sub-issue #6669에서 구현한다. #6669는 이 핵심 구현이 devel에 들어간 뒤 최신 devel의
독립 작업 브랜치에서 수행한다.

따라서 이 보고서의 PR이 병합돼도 #6628을 닫지 않는다. #6669의 결정론적·offline 시각화,
fail-closed 판정, 원문 SHA-256 계보와 재현 문서를 병합한 뒤 부모 완료 조건을 최종 감사한다.
