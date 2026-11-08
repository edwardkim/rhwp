# 편람 필수 검사의 명시적 글꼴 환경 전달

2026-11-04, planet6897 원 PR 39건 통합 검증의 필수 편람 검사에서 확인했다.

## 독립 근거와 원인

- 원본 `samples/2025 행정업무운영 편람(최종).hwpx`의 `정부상징 부처명_16040911` 글꼴은
  언어 슬롯별로 `한컴바탕` 또는 `함초롬바탕` 대체를 선언한다.
- 기존 #8389·#8397 검증의 `handbook-font-environment.json`은 관측된 Print에 맞춰 이 이름을
  `Haansoft Batang`으로 표시 치환하고 `preserveSourceMetrics: true`를 유지한다.
- 현재 호스트에는 실제 정부상징 글꼴도 설치되어 있다. 기본 설치 글꼴을 선택한 145쪽은
  기존 기준 100.00%에 대해 99.95%였다. 새 base `1924d371d6`와 후보 제품 `40778b67d6`의
  같은 쪽 PNG는 SHA-256 `082f3b6abdac017550ef0e9fc09217d1fe093a46271f0833bb3b85cd0e3937e8`로
  바이트가 같았다. 이번 표 보정의 출력 회귀가 아니다.
- 실제 정부상징 글꼴만 일시적으로 가린 진단 환경에서는 원본의 대체 글꼴 선택으로 100.00%가
  됐다. 이 진단은 호스트 글꼴을 삭제·변경하지 않았으며 최종 전쪽 통과 증거로 사용하지 않는다.

Visual Sweep의 기존 공개 옵션 `--font-environment`는 일반 대상에는 전달됐지만
`--handbook-gate` 단독·추가 실행에서는 `None`으로 버려졌다. 설치 글꼴 전체를 유지하면서
기존 독립 Print와 같은 승인된 환경을 지정하려면 이 전달 누락을 보정해야 한다.

## 수정과 반례

`required_handbook_gate`와 `enforce_required_handbook`가 선택적인 환경 경로를 받고,
단독 실행·일반 대상 뒤 추가 실행·PNG pair 뒤 추가 실행이 같은 인자를 전달한다.
공통 렌더러가 실제 환경과 해시를 provenance에 기록한다. 옵션 생략은 종전 기본 동작을 유지한다.

383쪽 전체 선택, Native Print 96 DPI, 90% 하한과 #8389 페이지별 표시값 무하락 판정은
바꾸지 않는다. 특정 문서의 글자나 좌표·점수를 보정하지 않는다. 글꼴 파일도 추가하지 않는다.

수정 전 두 API 계약 검사는 CLI가 요청한 환경을 버리는 assertion 실패 1건과 기존 함수가
환경 인자를 받지 못하는 오류 1건으로 실패했다. 수정 뒤 편람 검사 모듈 17건은 모두 통과했다.
일반 Visual Sweep 회귀 전체와 실제 전쪽 gate는 다음 검증 단계로 연결한다.

## 검증 기록

임시 실행 기록은 `output/pr-review/planet6897-20261103/logs/`에 있다.

- `handbook-font-environment-before.log`: 수정 전 FAIL·ERROR.
- `handbook-font-environment-after.log`: 편람 모듈 17 passed.
- `final-visual-sweep-python-font-environment.log`: Visual Sweep Python 전체 검사.
- `handbook-p145-base-current-byte-parity.json`: 새 base·후보의 실제 PNG 동일성.
- `handbook-p145-declared-substitute-retry.log`: 글꼴 선택 원인 확인용 진단 100.00%.

최종 필수 검사는 호스트의 모든 시스템 글꼴을 그대로 제공하며, 기존
`mydocs/pr/assets/planet6897_integration_final/pr8389_8397/handbook-font-environment.json`을
명시해 실행한다. 격리한 글꼴 환경의 성공이나 단일 페이지를 383쪽 통과로 확대하지 않는다.
이 Python 전달 보정은 Rust 제품·WASM 바이트를 바꾸지 않아 진행 중인 Rust 검증을 재시작하지 않는다.
