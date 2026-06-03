# Task #1129 Stage 2 - 격자 설정 대화상자 확장

- 이슈: [#1129](https://github.com/edwardkim/rhwp/issues/1129)
- 브랜치: `local/task_m100_1129`
- 일자: 2026-05-26

## 작업 내용

- 기존 `격자 설정` 대화상자를 한컴오피스식 설정 항목으로 확장했다.
- 설정 항목:
  - 격자 보기
  - 격자 모양: 점, 가로선, 세로선, 가로/세로선
  - 격자 위치: 글 뒤, 글 앞
  - 격자 방식: 상관 없이, 자석 효과, 격자에만 붙이기
  - 격자 간격: 가로/세로 mm
  - 격자 기준 위치: 쪽, 종이
  - 기준 오프셋: 가로/세로 mm
  - 표/개체 이동 간격

## 기준 위치 동작

- `쪽` 기준은 본문 쪽 영역 기준이며, 격자 표시 범위를 본문 영역으로 clip 한다.
- `종이` 기준은 종이 전체 기준이며, 격자 표시 범위를 페이지 전체로 둔다.
- `종이` 기준으로 전환하면 현재 쪽의 종이 기준 보정값을 mm로 환산해 기본 오프셋으로 넣는다.
- 사용자가 직접 오프셋을 수정한 경우에는 기준 위치 전환 때 사용자 값을 보존한다.
- 로컬 기능 검증 샘플 `samples/exam_kor.hwp`에서는 `쪽 -> 종이` 전환 결과가 다음처럼 확인됐다.

```text
쪽 입력값:   3, 3 / overlay clip-path: inset(...) / background-position: 128.539px 223.039px
종이 입력값: 43, 35 / overlay clip-path: none / background-position: 162.52px 132.283px
```

## 변경 파일

- `rhwp-studio/src/ui/grid-settings-dialog.ts`
- `rhwp-studio/src/command/commands/view.ts`
- `rhwp-studio/src/view/grid-settings.ts`
