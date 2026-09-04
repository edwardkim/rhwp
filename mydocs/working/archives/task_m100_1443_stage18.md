# Task M100 #1443 Stage 18 작업 기록

- 이슈: #1443
- 브랜치: `local/task_m100_1443`
- 시작일: 2026-06-20
- 선행 커밋: `31005d6b task 1443: 표 이동과 속성 동기화 보정`

## 1. 목표

메뉴와 우클릭 컨텍스트 메뉴에 `모양 붙여넣기`를 명시적으로 추가한다.

## 2. 배경

현재 `edit:format-copy`는 내부 상태에 따라 다음 두 역할을 한다.

- 복사된 모양이 없거나 대상 선택이 없으면 현재 커서 위치의 모양 복사
- 복사된 모양이 있고 대상 선택이 있으면 선택 대상에 모양 적용

하지만 UI에는 `모양 복사`만 보여 사용자가 붙여넣기 동작을 찾기 어렵다.

## 3. 수정 계획

- `edit:format-paste` 커맨드 추가
- `InputHandler.performFormatPaste()` 추가
- `EditorContext`에 모양 복사 상태 여부 추가
- 편집 메뉴에 `모양 붙여넣기` 추가
- 기본/표/개체 컨텍스트 메뉴에 `모양 복사`, `모양 붙여넣기` 추가

## 4. 검증 계획

- `cd rhwp-studio && npx tsc --noEmit`
- `git diff --check`
- 수동 확인
  - 편집 메뉴에 `모양 붙여넣기` 표시
  - 우클릭 메뉴에 `모양 복사`, `모양 붙여넣기` 표시
  - 모양 복사 전 붙여넣기 비활성
  - 모양 복사 후 대상 선택 시 붙여넣기 활성

## 5. 수정 결과

- `EditorContext.hasCopiedFormat`을 추가했다.
- `InputHandler.hasCopiedFormat()`을 추가했다.
- `InputHandler.performFormatPaste()`를 추가했다.
- 기존 `performFormatCopy()`의 적용 로직을 `applyCopiedFormatToCurrentTarget()`으로 분리했다.
  - 기존 `Alt+C` 토글 동작은 유지한다.
  - 새 `모양 붙여넣기` 메뉴는 붙여넣기만 수행한다.
- `edit:format-paste` 커맨드를 추가했다.
- 편집 메뉴에 `모양 붙여넣기` 항목을 추가했다.
- 기본 컨텍스트 메뉴와 표 셀 컨텍스트 메뉴에 `모양 복사`, `모양 붙여넣기` 항목을 추가했다.

## 6. 검증 결과

- `cd rhwp-studio && npx tsc --noEmit`
  - 통과
- `git diff --check`
  - 통과

## 7. Hyper-Waterfall 회귀 테스트 해결 기록

### 7.1 목표

PR 준비 중 발견된 렌더링 회귀를 기존 golden 기준으로 해결한다.
golden SVG를 새로 뽑아 기준을 바꾸지 않고, #1443 변경이 기존 문서 렌더링을 깨뜨린 지점을 찾아 수정한다.

### 7.2 방법론 적용

`mydocs/manual/hyper_waterfall.md`의 품질 관문에 맞춰 다음 순서로 진행한다.

1. 계획/문서화
   - 현재 스테이지를 `task_m100_1443_stage18.md`에 기록한다.
   - 회귀 판단 기준은 기존 golden과 기존 이슈 테스트 통과로 고정한다.
2. 원인 추적
   - 실패한 테스트를 단독 실행한다.
   - 필요 시 stage별 커밋 또는 bisect로 회귀 도입 커밋을 확인한다.
3. 최소 수정
   - golden 갱신 없이 렌더링 로직을 좁게 보정한다.
   - #1443 셀 안여백/셀 보호 요구사항과 기존 문서 호환성을 동시에 만족시킨다.
4. 반복 검증
   - 실패 테스트 단독 실행 후 SVG snapshot 전체 실행.
   - 관련 회귀 테스트를 추가로 실행한다.
5. 정리
   - `.actual.svg` 산출물은 테스트 산출물이므로 삭제한다.
   - 남은 변경 범위와 검증 결과를 스테이지 문서에 반영한다.

### 7.3 발견된 회귀와 조치

- `issue_1073_nested_table_split`
  - 증상: page-larger 중첩 표 분할 시 저장된 공통 높이 보정이 분할 컷 계산에 들어가 행 단위 분할이 흔들림.
  - 조치: 중첩 표 분할 컷은 실제 콘텐츠 행 높이를 기준으로 계산하도록 분리.
- `svg_snapshot::issue_617_exam_kor_page5`
  - 증상: 셀 내부 텍스트 오버플로우 판단이 너무 넓게 적용되어 기존 exam_kor 보기 박스 렌더링이 달라짐.
  - 조치: 셀 내부 오버플로우 자간 억제를 `available_width * 1.15` 초과 케이스로 제한.
- `svg_snapshot::issue_677_bokhakwonseo_page1`
  - 증상: 복학원서 영문 문장이 기존 golden보다 덜 압축되어 오른쪽으로 밀림.
  - 조치: 비정렬 문단 오버플로우 분기도 동일한 억제 기준을 사용하도록 수정.
- `svg_snapshot::issue_267_ktx_toc_page`
  - 증상: 표 높이 공통 보정이 행을 축소하면서 KTX 목차 표 높이가 기존 golden과 달라짐.
  - 조치: 저장된 공통 표 높이보다 실제 행 합계가 큰 경우 행을 축소하지 않도록 보정.

### 7.4 검증 결과

- `cargo test --profile release-test --test svg_snapshot -- --nocapture`
  - 통과: 8 passed
- `cargo test --profile release-test --test issue_1073_nested_table_split -- --nocapture`
  - 통과: 3 passed
- `cargo test --profile release-test --test issue_1100_exam_social_hwpx_header -- --nocapture`
  - 통과: 3 passed
- `cargo test --profile release-test --test issue_493_cell_attrs -- --nocapture`
  - 통과: 10 passed
- `cargo test --profile release-test --test issue_493_hwpx_cell_field_name -- --nocapture`
  - 통과: 1 passed
- `cargo test --profile release-test --test issue_915_charshape_cell_font_size -- --nocapture`
  - 통과: 1 passed
- `git diff --check`
  - 통과

### 7.5 현재 판단

현재까지 확인된 렌더링 회귀는 golden 기준으로 해결되었다.
다음 스테이지에서는 전체 `cargo test --profile release-test --tests`, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` 순서로 최종 회귀 검증을 이어간다.
