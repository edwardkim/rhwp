# Task M100 #2069 Stage 5: OLE Backspace 후 첫 줄 Enter 재진입 보정

## 목표

`samples/한셀OLE.hwp`와 `samples/한셀OLE.hwpx`에서 처음 로드 직후 Enter는 한컴처럼 동작하지만, Enter로 만든 빈 문단을 Backspace로 지운 뒤 첫 줄에서 다시 Enter를 누르면 OLE가 새 문단으로 밀려나는 문제를 보정한다.

## 원인

Stage 3에서 비-TAC Square OLE 앵커 문단의 Enter는 원본 `LINE_SEG`의 wrap-zone 값을 보존하도록 보정했다.

하지만 Backspace 병합 경로(`merge_paragraph_native`)는 실제 내용이 바뀌지 않는 빈 문단 제거임에도 이전 OLE 앵커 문단을 일반 본문 폭으로 `reflow`했다. 그 결과 OLE 앵커 문단의 `column_start`/`segment_width`가 원래 OLE 옆 wrap-zone 값이 아니라 일반 본문 흐름 값으로 바뀌고, 다음 Enter가 OLE 전용 분기가 아니라 일반 문단 split 경로로 내려가 OLE control을 새 문단으로 이동시켰다.

## 수정

- 병합 대상 문단이 텍스트·컨트롤이 없는 빈 문단이고, 이전 문단이 비-TAC Square OLE wrap chain에 속하면 이전 문단의 `LINE_SEG`를 reflow하지 않는다.
- 이 경우 병합은 문단 제거만 수행하고, vpos 재계산·composed 문단 제거·이전 문단 재조판·페이지네이션만 수행한다.
- 샘플명이나 텍스트 내용이 아니라 문단 구조, OLE control 종류, `TextWrap::Square`, `treat_as_char=false`, 저장된 wrap-zone `LINE_SEG` 조건으로 제한했다.

## 검증 계획

- Rust 통합 테스트에서 `Enter -> Backspace -> Enter` 순서를 직접 실행해 OLE가 계속 0번 문단에 남는지 확인한다.
- Studio E2E에서 실제 키 입력으로 같은 순서를 실행해 OLE 내부 셀 편집으로 들어가지 않고 OLE 오른쪽 wrap-zone에 caret이 남는지 확인한다.
- 기존 OLE 내부 클릭 개체 선택 E2E도 함께 유지한다.
