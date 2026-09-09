---
kind: working
status: completed
canonical: mydocs/working/task_m100_6662_stage2_signed_table_offset.md
last_verified: 2026-09-05
---

# 열린 이슈 재검증 2단계: 표 오프셋 부호

## 범위

- Issue: #6714. 전체 조사 범위는 #6662의 [1단계](task_m100_6662_stage1_jeongsik_recheck.md)를 따른다.
- 1단계 자료 commit: `877ad8157`. 제품 코드는 `1f861362a`와 같다.
- 기존 수정의 focused 검증은 7개 suite에서 12 passed, 0 failed, 1180 filtered.
  #6389·#6646·#6656·#6660·#6681·#6704 및 #3931을 실행했고 빌드 6분 58초, 테스트 0.919초,
  exit 0이다. 전체 회귀 실행은 아니다.
- 원본 확보와 현행 상태 분류가 끝났으므로 이 단계에서는 #6714만 독립적으로 수정한다.

## 원인과 적용 경로

`pagination/engine.rs`의 다섯 양수 판정은 raw `u32`로 비교한 뒤 `i32`로 계산한다.
음수 오프셋이 양수 전용 선행 텍스트·표 위치·분할 가용 높이 처리로 진입할 수 있다.
이 코드의 `get_table_vertical_offset`에서 signed 해석을 통일하고 직접 필드 비교 두 곳도
같은 accessor를 사용한다. 모델·파서·직렬화의 raw 필드 타입은 바꾸지 않는다.

`MeasuredSection`과 `DocumentCore` 호출부를 확인한 결과 일반 문서 페이지네이션은 TypesetEngine을
쓰고 이 경로는 fallback이다. 기존 8개 파일의 픽셀이 안 바뀌었다는 작성자 관측을 코드가 올바르다는
증거로 쓰지 않는다. 반대로 이 수정이 #6712의 3쪽 문제를 해결한다고 주장하지도 않는다.

## 검증 계획

1. `tests/cases/`에 공개 `Paginator::paginate_with_measured` API를 쓰는 합성 계약을 만든다.
2. 음수에서 양수 전용 pre-table text/가용 높이 증가가 발생하는 기존 실패를 확인한다.
3. 0과 양수의 기존 결과도 고정한다. 음수 범위에는 -1과 최소 i32 표현을 포함한다.
4. accessor와 양수 조건을 보정한 뒤 같은 테스트를 통과시킨다.
5. 파생 suite는 커밋하지 않는다. code 후보의 전체 lint/회귀와 시각 비교는 최종 PR 준비에서 실행한다.

## 종료 판단 경계

- 사내 문서 ID 8개 원본의 로컬 대응이 확보되지 않은 상태에서 그 8개를 재검증했다고 쓰지 않는다.
- fallback 계약 통과와 사용자-visible 그림 어울림 개선은 다른 성과다.
- 아직 remote push, 신규 PR 생성 또는 #6714 종료는 수행하지 않았다.

## 실행 결과

- 수정 전 제품 코드(`1f861362a`): 3개 중 1 passed, 2 failed, exit 100.
  - raw `-1`에서 표보다 본문이 먼저 놓였다. 0 오프셋의 표-본문 순서와 달랐다.
  - `-750HU`에서 첫 쪽에 55px 행 두 개가 들어갔다. 남은 100px에는 한 행만 들어가야 한다.
  - 양수 오프셋 대조군은 통과했다.
- 수정 후: 3 passed, 0 failed, 180 filtered, exit 0. 빌드 5분 43초, 테스트 0.011초.
- 명령: `node scripts/run-rust-test.mjs issue_6714_signed_table_offset -- --cargo-profile release-test --target-dir target/pr-review --build-jobs 2 --test-threads 3 --no-fail-fast`.
- `-1`, `-750`, `i32::MIN`의 음수 해석과 0/양수 동작을 고정했다. raw 저장 필드는 그대로다.
- 이 단계의 focused 검증 완료이지 전체 lint/회귀 완료가 아니다. 임시 로그와 파생 suite는 커밋하지 않는다.
