# Task #598 Stage 4-2 완료보고서 — rhwp-studio 각주 삭제 UI 연결

## 작업 개요

- **Issue**: [#598](https://github.com/edwardkim/rhwp/issues/598)
- **브랜치**: `local/task598`
- **단계 범위**: 본문 각주 마커 삭제 API를 rhwp-studio Backspace/Delete 입력에 연결

본 단계에서는 Stage 4-1에서 추가한 `getFootnoteAtCursor` / `deleteFootnote` WASM API를 rhwp-studio 입력 처리에 연결했다.

## 구현 내용

### WASM Bridge / 타입

- `FootnoteAtCursorResult` 타입 추가
- `DeleteFootnoteResult` 타입 추가
- `WasmBridge.getFootnoteAtCursor()` 추가
- `WasmBridge.deleteFootnote()` 추가

### 키보드 입력 연결

- 본문 모드 Backspace 처리 전에 커서 바로 앞 각주 마커를 조회한다.
- 본문 모드 Delete 처리 전에 커서 바로 뒤 각주 마커를 조회한다.
- 각주 마커가 있으면 일반 `DeleteTextCommand` 대신 `SnapshotCommand` 로 `deleteFootnote` 를 실행한다.
- 삭제 후 커서는 삭제된 각주 마커 위치(`charOffset`)로 이동한다.
- 셀/글상자 내부 위치에서는 이번 범위에서 제외하고 기존 동작을 유지한다.

## 검증 결과

실행 명령:

```bash
cd rhwp-studio && npm run build
docker-compose --env-file .env.docker run --rm wasm
cd rhwp-studio && npm run build
git diff --check
```

결과:

- `npm run build`: 통과
- `docker-compose --env-file .env.docker run --rm wasm`: 통과
- 새 WASM `pkg/` 반영 후 `npm run build`: 통과
- `git diff --check`: 통과

참고:

- `npm run build` 에서 Vite chunk size warning 이 출력됐다. 기존 번들 크기 경고이며 빌드는 성공했다.

## 수동 검증

작업지시자가 macOS 환경에서 `http://localhost:7700/` 로 rhwp-studio 를 실행해 Backspace 및 `Fn+Delete` 경로를 확인했다.

확인 내용:

- `samples/footnote-01.hwp` 로드
- 본문 각주 마커 뒤 커서 위치에서 Backspace 실행
- 본문 각주 마커 앞 커서 위치에서 `Fn+Delete` 실행
- 각주 삭제 동작 확인
- `Fn+Delete` 삭제 후 첫 번째 각주 본문 제거 및 기존 두 번째 각주가 `1)` 로 재번호화됨

수동 테스트용 Vite 서버는 확인 후 종료했다.

## 남은 작업

다음 단계(Stage 4-3)에서는 전체 검증과 PR 전 정리를 진행한다.

- Rust/Studio 빌드 재확인
- 최종 보고서 작성
