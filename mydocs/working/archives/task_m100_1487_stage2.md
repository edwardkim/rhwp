# Task M100 #1487 Stage 2

- 이슈: #1487 확장 옵션에 외부 웹폰트 사용 안 함 설정 추가
- 브랜치: `task_m100_1487`
- 작성일: 2026-06-23
- 상태: 구현/검증 완료

## 배경

Stage 1 구현 후 Chrome 확장 시각 검증 과정에서 빈 viewer 탭을 클릭하면
Chrome 확장 오류 목록에 다음 경고가 기록되는 것을 확인했다.

```text
[InputHandler] hitTest 실패: Error: 문서가 로드되지 않았습니다
```

`viewer.html` 자체는 정상 로드되지만, 문서가 없는 초기 화면에서도
`InputHandler`의 `mousedown` 핸들러가 `wasm.hitTest()`까지 진입하면서
불필요한 오류 기록을 남긴다.

## 구현 방향

- 문서가 로드되지 않은 상태(`wasm.pageCount <= 0`)에서는 container 클릭 입력을 무시한다.
- 빈 viewer 클릭이 Chrome 확장 오류 목록에 기록되지 않도록 한다.
- 실제 문서가 로드된 뒤의 클릭/드래그/객체 선택 동작은 유지한다.

## 검증 계획

```bash
cd rhwp-studio && npm run build
cd rhwp-studio && npm test
cd rhwp-chrome && node build.mjs
git diff --check
```

## 구현 내용

- `input-handler-mouse.ts`의 `onClick` 진입점에서 `wasm.pageCount <= 0`이면 즉시 반환하도록 했다.
- 빈 viewer 클릭이 `wasm.hitTest()`까지 진입하지 않도록 소스 기반 회귀 테스트를 추가했다.

## 검증 결과

- `cd rhwp-studio && npm test`: 통과 (128 passed)
- `cd rhwp-studio && npm run build`: 통과
- `cd rhwp-chrome && node build.mjs`: 통과
- `git diff --check`: 통과
