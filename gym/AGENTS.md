# Gym 작업 지침

이 파일은 `gym/**`에 적용되는 범위 지침이다. 저장소 루트 `AGENTS.md`를 먼저 적용하고,
수동 벤치마크 운영은
[`mydocs/manual/gym_benchmark_operations.md`](../mydocs/manual/gym_benchmark_operations.md)를
정본으로 삼는다. 개별 도구 동작은 [`docs/`](docs/)의 해당 규약을 따른다.

## 역할을 먼저 고정한다

- **참가자 모드**: 주어진 task를 스스로 풀고 제출한다.
- **메인테이너 감사 모드**: task/reference/check/runner와 벤치마크 판별력을 검증한다.

“Gym 과제를 풀어라”는 참가자 모드다. 참가자는 `packs/<id>/tasks/`, 공개 `assets/`,
`gym/README.md`만
읽고 `reference/`, 기준풀이, 채점 기대값, 정답 권위 원장을 보지 않는다. 이를 읽으면 점수는
나올 수 있어도 경로 탐색 능력 측정은 무효다. 참가자 모드에서 `build_baseline.py`를 답안
생성기로 사용하지 않는다.

Gym 구현·감사·정상화 요청은 메인테이너 감사 모드다. 이 모드에서만 `reference/`와 감사
도구를 읽고 실행한다. 역할이 불명확하면 답안을 만들기 전에 어느 모드인지 확인한다.

## 메인테이너 감사 불변식

1. Gym은 에이전트 능력 벤치마크이지 한컴 조판 또는 제품 정확성의 독립 정답지가 아니다.
2. 일반 PR·`devel`·`main`·tag·Release Binary·npm·extension 게시를 Gym 결과에 의존시키지
   않는다. 그런 변경은 별도 승인된 이슈 없이는 제안하거나 구현하지 않는다.
3. 양성 기준풀이, 음성 판별력, 경로 필요성을 서로 대체하지 않는다. 세 축을 각각 판정한다.
4. 오류·누락·skip을 성공으로 접지 않는다. `ok` 하나만 보지 말고 정본 매뉴얼의 봉투
   불변식을 재계산한다.
5. 실패를 통과시키려고 task/reference/check/`allowExits`/권위 분류를 약화하지 않는다.
6. Gym에서 발견한 Rust 제품 결함은 별도 이슈·브랜치·제품 검증으로 분리한다.
7. 공개 Gym은 tracked 공개 자산만으로 재현한다. 사설 코퍼스나 비밀값을 읽거나 증적에
   포함하지 않는다.

## 수동 전수 실행 순서

1. exact Gym runner SHA/tree와 제품 source SHA를 고정한다.
2. 주 checkout을 보존하고 disposable worktree와 외부 target/output 경로를 만든다.
3. `test_gym_*.py`, `audit.py`, oracle structural/selftest를 먼저 실행한다.
4. 이슈에 알려진 실패가 있을 때만 pack canary를 실행한다. canary를 전수 통과로 부르지 않는다.
5. positive `build_baseline.py --json`을 전건 실행한다.
6. `discriminate.py --json`을 전건 실행한다.
7. `trajectory.py --json`을 전건 실행한다.
8. source·runner·binary 신원, 종료 코드, JSON 집계, 시간, 오류 분류와 cleanup을 보고한다.

정확한 명령과 판정 필드는
[`Gym 벤치마크 수동 운영 매뉴얼`](../mydocs/manual/gym_benchmark_operations.md)을 복사하지
말고 그 문서에서 읽는다. 과거 보고서의 pack/task 수를 현재 기대값으로 하드코딩하지 않는다.

## 산출물과 안전

- `gym/submissions/`의 baseline·negative-control·trajectory 생성물은 source가 아니다.
  커밋하거나 PR에 stage하지 않는다.
- 전수 실행은 disposable worktree에서 수행한다. 메인 checkout의 기존 제출물과 사용자 WIP를
  삭제하거나 덮어쓰지 않는다.
- 정리는 자신이 생성한 exact 경로만 대상으로 한다. 넓은 경로, 해석되지 않은 변수, glob을
  재귀 삭제에 사용하지 않는다.
- JSON stdout과 진단 stderr를 분리하고, 프로세스 종료 코드를 즉시 기록한다.
- `discrimination.scoreErrors`는 의도된 음성 거부일 수 있다. 비어 있지 않다는 이유만으로
  성공·실패를 정하지 말고 control별 원인을 정산한다.
- `trajectory.ok=true`라도 `trusted=false`, 예외, tool 오류가 있으면 완료가 아니다.

## 실패 처리와 변경 절차

- 제품 결함, 벤치마크 결함, 환경 결함을 분리해 보고한다. 한 분류의 수정으로 다른 분류의
  실패를 숨기지 않는다.
- Gym 도구나 task를 바꾸면 관련 단위 계약과 구조 감사를 먼저 실행한다. 판정 의미가 바뀌면
  양성·음성·경로 세 축을 다시 실행한다.
- 문서만 바꾸면 링크·metadata·`git diff --check`를 적용하고 Rust 전체 lint를 형식 조건으로
  오용하지 않는다.
- 이슈, 브랜치, 단계 계획·보고, GitHub comment, push, PR, merge의 승인 경계는 루트
  `AGENTS.md`와 프로젝트 워크플로우를 그대로 따른다.
