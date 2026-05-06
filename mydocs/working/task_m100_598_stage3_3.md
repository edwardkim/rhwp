# Task #598 Stage 3-3 완료보고서 — WasmBridge + mouse 연결

## 작업 개요

- **Issue**: [#598](https://github.com/edwardkim/rhwp/issues/598)
- **브랜치**: `local/task598`
- **기준 커밋**: `upstream/devel` `9b49063`
- **단계 범위**: rhwp-studio 에서 본문 각주 마커 클릭 시 각주 편집 모드로 진입하도록 연결

본 단계에서는 Stage 3-1/3-2에서 추가한 Rust/WASM API를 rhwp-studio 입력 흐름에 연결했다. 삭제 API/UI는 아직 구현하지 않았다.

## 변경 파일

| 파일 | 변경 내용 |
|------|-----------|
| `rhwp-studio/src/core/types.ts` | `BodyFootnoteMarkerHit` 타입 추가 |
| `rhwp-studio/src/core/wasm-bridge.ts` | `hitTestBodyFootnoteMarker(pageNum, x, y)` 래퍼 추가 |
| `rhwp-studio/src/engine/input-handler-mouse.ts` | 본문 각주 마커 클릭 처리 분기 추가 |

## 구현 내용

### 1. WasmBridge 래퍼 추가

`WasmBridge.hitTestBodyFootnoteMarker()` 를 추가했다.

반환 타입은 다음 필드를 가진다.

```ts
{
  hit: boolean;
  sectionIndex?: number;
  paragraphIndex?: number;
  controlIndex?: number;
  footnoteNumber?: number;
  footnoteIndex?: number;
  bbox?: { x: number; y: number; w: number; h: number };
  cursorRect?: CursorRect;
}
```

현재 `pkg/` 바인딩이 갱신되지 않은 환경에서도 앱이 즉시 깨지지 않도록, 메서드가 없으면 `{ hit: false }` 를 반환하게 했다.

### 2. 마우스 입력 처리 순서 변경

`input-handler-mouse.ts` 의 클릭 처리 순서를 다음처럼 확장했다.

```text
1. 각주 편집 모드 내부/외부 클릭 처리
2. 본문 각주 마커 hit test
3. 각주 영역 클릭 → 각주 편집 모드 진입
4. 일반 본문 hitTest
```

본문 각주 마커 hit 시 다음을 수행한다.

1. `cursor.enterFootnoteMode(sectionIndex, paragraphIndex, controlIndex, footnoteIndex, pageIdx)` 호출
2. `footnoteModeChanged` 이벤트 emit
3. 각주 내부 커서를 첫 문단 offset 0으로 설정
4. caret 갱신 후 입력 포커스 유지

### 3. WASM 바인딩 재생성

브라우저 런타임에서 새 Rust export 를 실제로 호출할 수 있도록 Docker WASM 빌드를 실행해 로컬 `pkg/` 를 갱신했다.

확인 결과:

```text
pkg/rhwp.js
pkg/rhwp.d.ts
pkg/rhwp_bg.wasm.d.ts
```

위 파일들에 `hitTestBodyFootnoteMarker` / `hwpdocument_hitTestBodyFootnoteMarker` 가 생성됐다.

참고: 현재 저장소 설정상 `pkg/` 는 Git 추적 대상이 아니므로 `git status` 에는 표시되지 않는다.

## 검증

실행 결과:

```bash
docker-compose --env-file .env.docker run --rm wasm
cd rhwp-studio && npm run build
cargo build
git diff --check
curl -I http://localhost:7700/
```

결과:

- Docker WASM 빌드 통과
- `pkg/` 바인딩에 `hitTestBodyFootnoteMarker` 생성 확인
- `npm run build` 통과
- `cargo build` 통과
- `git diff --check` 통과
- Vite dev server 응답 확인: `HTTP/1.1 200 OK`

## 수동 확인 요청

Vite dev server 를 다음 주소로 실행해 두었다.

```text
http://localhost:7700/
```

브라우저에서 확인할 절차:

1. `samples/footnote-01.hwp` 를 연다.
2. 1페이지 본문 각주 마커 `1)` 또는 `2)` 를 클릭한다.
3. 하단 각주 영역으로 caret 이 이동하고 각주 편집 모드가 켜지는지 확인한다.
4. 각주 영역 밖 본문을 클릭하면 각주 편집 모드가 해제되는지 확인한다.

## 남은 작업

다음 승인을 받은 뒤 Stage 3-4에서 진행한다.

1. 가능한 e2e 또는 브라우저 수동 검증 결과 반영
2. 1차 작업 최종 회귀 검증
3. Stage 3 전체 완료보고서 정리
