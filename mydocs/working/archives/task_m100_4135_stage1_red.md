# M100 #4135 Stage 1 — RED 및 현행 재현

## 목표

F5 셀 블록 문맥의 `Ctrl/Cmd+Shift+S`가 일반 저장 단축키와 충돌하는 현재 동작을
고정하고, 수정 전 실패 계약을 남긴다.

## 자동 테스트 RED

실행:

```bash
cd rhwp-studio
node --test tests/issue-4135-contextual-shortcut.test.ts tests/shortcut-map.test.ts
```

결과: 9 pass, 3 fail.

- `src/command/contextual-shortcut.ts`가 아직 없어 문맥 라우팅 계약이 실패한다.
- `Process/KeyS` 형태의 한글 IME 이벤트가 `file:save-as`와 매칭되지 않는다.
- 전역 단축키 맵에 `file:save-as`와 `table:block-sum`의 동일 슬롯이 함께 있어
  첫 항목이 뒤 항목을 가리는 충돌 검사가 실패한다.

## 실제 브라우저 재현

환경:

- Vite 개발 서버: `http://127.0.0.1:7715/`
- 새 문서에서 2행 3열 표 생성
- 셀 안에서 `F5`, `F5`, `ArrowRight`로 복수 셀 블록 선택

관찰:

1. 셀 블록에서 `Ctrl+Shift+S`를 누르면 **셀 나누기** 대화상자가 열린다.
2. 대화상자를 취소하고 `Escape`로 셀 블록을 해제한 뒤 같은 단축키를 누르면
   **다른 이름으로 저장** 대화상자가 열린다.

따라서 문제는 저장 명령 자체가 아니라, 셀 블록 분기에서 modifier를 확인하지 않은
`S` 처리와 전역 단축키 맵의 중복 슬롯이 결합한 문맥 라우팅 결함으로 재현된다.

## 수정 계약

- full 모드 + F5 셀 블록: `table:block-sum`
- full 모드 + 셀 블록 밖: 기존 `file:save-as`
- 한글 IME `ㄴ` 및 `Process/KeyS`: 위와 같은 문맥 규칙
- modifier 없는 `S`: 기존 `table:cell-split`
- embed 모드: 파일/표 명령으로 fall-through하지 않고 이벤트만 소비
