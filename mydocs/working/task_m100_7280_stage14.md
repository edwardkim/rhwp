# Task #7280 Stage 14 — R2l 빈 문단 조기 반환 책임 분리

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 이전 절편: [R2k](task_m100_7280_stage13.md), 시작 head `2621b115c`.
- 고정 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: 구현·집중 검증 진행 중. 제출 준비 완료가 아니다.

## 1. 책임과 보존 범위

이번 절편은 문단 진입 뒤 빈 문단의 조기 반환 세 경로를 분리한다. 저장 강제 경계·최종 fit
선택을 동시에 옮기지 않는다. 빈 문자열·가시 텍스트·공백 제외 텍스트 판정을 통합하지 않으며,
각 기존 조건·상수·항목 소유·숨김 횟수·높이 전진 여부를 그대로 유지한다.

- RowBreak 뒤 guide 흡수는 다단 처리 **전**에 판단하며 숨김 목록만 기록한다.
- `hide_empty_line` 처리는 다단 처리 **후**에만 실행한다. 옵션이 켜졌을 때 페이지별 횟수를
  먼저 초기화하고 판정한 뒤, 횟수 증가 → 숨김 기록 → FullParagraph 추가 순서로 반영한다.
- 구역 끝 빈 문단은 앞선 drift 조건에서 숨김 목록만 기록하거나, 안전여백/각주 차감 조건에서
  높이 전진 없이 항목만 남긴다. 두 효과를 동일한 숨김 처리로 합치지 않는다.

`paragraph/empty.rs`는 읽기 전용 입력으로 판단하고, paragraph 조정자가 결과별 command를
호출한다. state는 확정 항목·숨김 집합·횟수만 변경한다. tail 관측값의 base 높이는 부수효과
없는 조회이며, 실제 available은 이전 절편의 고정 예산을 그대로 소비한다.

소비 경로: 기존 fit 예산 → guide 조기 반환 → 기존 다단 경로 → 옵션/구역 끝 판정 →
숨김 집합·항목 반영 → 반환 또는 기존 저장 경계/fit/분할. 컷·높이·paint 재계산은 바꾸지 않는다.
이동 대상은 기존 호환 처리이며 올바른 한컴 규칙으로 재승인하거나 빈 줄 버그를 수정하지 않는다.
IR/API, 테스트 assertion·ID, baseline·golden·ignore, CI 정책은 수정하지 않는다.

## 2. 검증 계획

원래 block으로 복원하여 판단식·쓰기·조기 반환 순서를 대조한다. 다단 분기 사이의 호출
위치, 기존 높이 budget과 나머지 parent 불변을 확인한다. 별도 review worktree에서
파생 suite 준비 → fmt·고정 baseline 대비 manifest/unit-tier → native Clippy → 집중 nextest를
순차 실행한다. typeset/composer의 빈 문단·다단 계약과 기존 빈 문단 고아 페이지/fit 계약을
포함하되, 기존 테스트가 세 경로의 모든 조합을 직접 보호한다고 주장하지 않는다.

전체 회귀·WASM/workspace lint·Native Skia·fresh Docker WASM/직접 시각 대조는 통합 게이트에
남긴다. 원격 push·PR·댓글은 수행하지 않는다.
