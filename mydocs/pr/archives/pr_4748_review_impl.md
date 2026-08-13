# PR #4748 구현 검토 - 개발용 핫패치 경계 정리

## 대상과 code candidate

| 이슈 | 구현 범위 | code candidate |
| --- | --- | --- |
| #4636 | DEV 동적 import와 realm 단위 런타임 수명 | `6c51110474d505d2e9eff9353b4a4e2c2d01c4b0` |
| #4641 | WasmBridge/CanvasView의 개발 전용 소유 제거와 배포 chunk 재귀 검사 | 동일 |
| #4642 | Rust 렌더 패치 경계 도메인화·공용 vendor helper 제거·BBox 값 보존 | 동일 |

## 적용 순서와 rollback 경계

1. `subsecond-runtime.ts`가 realm마다 단 하나의 소켓·감시자를 만들고 stop closure로 정리한다.
   `main.ts`는 DEV 동적 import 성공 뒤에만 이를 시작하며, 실패해도 일반 Studio 초기화를 중단하지 않는다.
2. `WasmBridge`는 WASM export와 선형 메모리 조회만 제공하고, `CanvasView`는 일반 화면 갱신만 소유한다.
   코드 리비전 알림은 `canvasView?.refreshPages()`를 직접 호출한다.
3. deployment `dist` 하위 모든 JavaScript를 검사해 중첩 chunk가 검사 밖으로 빠지지 않게 한다.
4. Rust의 경계 모듈을 `render_patch_boundary`로 바꾸고, 적용된 함수 주소를 boundary macro 내부에서만
   읽는다. 이 코드는 점프 테이블의 현재 주소를 읽어야 하므로 일반 함수 포인터 cast로 되돌리지 않는다.

rollback이 필요하면 이 PR의 단일 code commit을 되돌린다. 문서 trailing commit은 런타임 동작을 바꾸지
않으므로 별도로 되돌릴 필요가 없다.

## 검증과 후속 조건

Studio 단위·번들·headless 재도색 검증과 Rust 경계 unit/feature/wasm32 검증을 모두 완료했다. 전체
layout fidelity 변경이 아니므로 PDF 기준 visual sweep은 이번 PR의 판정 근거가 아니다. 최신 PR head의
GitHub Actions, CodeQL, Render Diff가 통과하고 작업지시자 승인 뒤에만 merge한다.
