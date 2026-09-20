---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_6766_stage1.md
last_verified: 2026-09-20
---

# #6766 1단계: 빈 누름틀 캐럿의 문단 정렬 앵커 복원

## 분석과 계획

이슈 #6766은 빈 가운데 정렬 누름틀에 캐럿을 놓으면 캐럿이 문단의 본문 좌단으로
되돌아가는 문제를 보고했다. 최신 `upstream/devel`에서 Studio와 같은 공개 API로
빈 문단 세 개를 만들고 각각 left/center/right 정렬 및 ClickHere를 넣어 재현했다.
수정 전 `get_cursor_rect_native`의 x 값은 모두 `113.4`였다.

renderer는 빈 ClickHere에 대해 문단 정렬로 계산한 zero-width TextRun 앵커를 이미
만든다. 그러나 `cursor_rect.rs`의 빈 호스트 문단 빠른 경로가 `Control::Field`를
페이지 탐색 대상에서 제외하여 이 앵커를 찾지 못했다. 그 결과 TextLine 좌단
fallback을 반환했다. 빈 호스트 문단의 페이지 탐색 절약(#4126)은 유지하면서,
zero-length ClickHere 범위만 예외로 페이지 탐색에 포함한다.

## 결과보고

- `get_cursor_rect_native`가 zero-length `FieldType::ClickHere`를 가진 빈 문단에서는
  renderer의 페이지 앵커를 조회하도록 수정했다. 일반 빈 문단과 다른 필드는 기존
  빠른 경로를 그대로 사용한다.
- public `apply_para_format_native`와 `insert_click_here_field_at`로 좌·가운데·오른쪽
  정렬 ClickHere를 만드는 회귀 테스트를 추가했다. 수정 후 x가 엄격히
  `left < center < right`가 되는 것을 검증한다. 테스트에서 IR을 직접 고치지 않아
  문단 스타일 cache와 재조판 상태도 실제 Studio 경로와 같다.
- 수정 전 재현은 `left=113.4, center=113.4, right=113.4`였고, 수정 후 새 회귀는
  통과했다.
- 실행 검증:
  - 새 회귀 1 passed / 198 skipped (`regression_suite_004`)
  - #4126 빈 호스트 문단 페이지 탐색 보호 회귀 1 passed
  - 기존 ClickHere form mode 13 passed / 184 skipped
  - 전체 release-test nextest 10,098 passed / 50 skipped / 18 slow (977.504초)
  - `cargo fmt --all -- --check`, 네이티브 clippy, WASM clippy, workspace all-target
    clippy, workspace build, test-suite manifest check 통과
  - `rhwp-wasm-build` 성공; `pkg`만 생성하고 추적 파일은 바꾸지 않았다.
- 인쇄 조판이나 렌더 트리를 바꾸지 않고 편집기 캐럿 rectangle 조회만 고쳤으므로
  PDF Visual Sweep은 적용 대상이 아니다. 이슈에 언급된 guide overlay 자체의 잔상은
  #6862의 별도 레이어이며, 이번 변경은 그 위젯이 소비하는 caret x를 올바른 앵커로
  돌려준다.
- 첫 번째 연속 focused-test 명령은 두 번째 명령부터 검토 전용 target 환경 변수를
  상속하지 않아 macOS Xcode license 오류가 발생했다. 같은 두 보호 검증을 환경을
  고정해 재실행하여 모두 통과했고, 이 오류는 소스·테스트 실패가 아니다.

이 단계는 분석 → 코드 수정·검증 → 결과보고 순서를 완료했다. 원격 push, PR 생성,
이슈 comment 또는 close는 수행하지 않았다.
