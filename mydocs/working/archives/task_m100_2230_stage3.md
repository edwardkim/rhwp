# 3단계 완료보고서 — Task M100 #2230: studio 그림 지정 UI

- 이슈: #2230 / 구현계획서: `task_m100_2230_impl.md` / 브랜치: `local/task2230`
- 작성일: 2026-07-12

## 수행 내용

placeholder 선택 상태에서 더블클릭 → 파일 선택 → 그림 지정 흐름을 연결했다.

### 변경 파일 6개 (rhwp-studio)

1. **`src/core/wasm-bridge.ts`** — `assignPictureImage(...)` 래퍼 추가
   (cellPathJson 규약은 insertPicture 와 동일).
2. **`src/core/types.ts`** — `ControlLayoutItem.missing?: boolean` (1단계
   Rust 방출 필드의 타입 반영).
3. **`src/engine/input-handler-picture.ts`**
   - `PictureObjectRef.missing` 필드 + hit 반환 3경로(controlToRef /
     nestedPic 우선 패스 / behindText 2차 패스) 모두 전파.
   - `promptAssignPictureImage(ref)` 신설: 파일 선택(`input type=file`,
     insert:image 커맨드와 동일 accept 목록) → 자연 크기 측정 → **선택 해제
     선행** → `executeOperation({kind:'snapshot',
     operationType:'assignPictureImage', ...})` 로 `wasm.assignPictureImage`
     호출. 스냅샷 경로의 `refreshAfterOperation('full')` 이 전면 재렌더를
     수행하므로 수동 재렌더 없음. **Undo/Redo 는 스냅샷 패턴으로 자동
     정합.** 실패 시 토스트 안내.
4. **`src/engine/cursor.ts`** — `PictureSelectionRef.missing` +
   `enterPictureObjectSelectionDirect` 말미 옵션 파라미터 추가. **정찰에서
   발견한 소실 지점 정정**: 선택 저장이 위치 인자로 ref 를 재구성하므로
   파라미터를 뚫지 않으면 더블클릭 시점에 missing 이 사라진다.
5. **`src/engine/input-handler-mouse.ts`**
   - `enterPictureObjectSelectionDirect` 호출 3곳(본문 이미지 메인 경로 /
     글상자 안 picture / 머리말·꼬리말 그림)에 missing 전달.
   - `onDblClick`: 개체 선택 중 `ref.type === 'image' && ref.missing` 이면
     그림 지정 진입 — **수식(수식 편집)·글상자(텍스트 편집) 분기보다 앞**.
     일반 그림 더블클릭 동작은 불변(missing 만 분기).
6. **`src/engine/input-handler.ts`** — `promptAssignPictureImage` 위임
   메서드 (기존 _picture 모듈 위임 패턴).

### 클릭 선택 자체는 TS 무수정 성립 (1단계 설계 확인)

placeholder 가 `type:"image"` 컨트롤로 방출되므로 findPictureAtClick →
enterPictureObjectSelectionDirect 선택·테두리·핸들은 기존 로직 그대로
동작한다. 3단계의 TS 변경은 missing 마커 전파와 더블클릭 진입만이다.

## 검증

- `npx tsc --noEmit` 클린 (중간 오류 1건 — 위임 메서드 ref 타입 누락 —
  즉시 정정)
- `npm run build` 성공 (vite + PWA)
- `npm test` (node --test): **206 passed / 0 failed**

## 다음 단계

4단계 — HWPX/HWP 저장 왕복 표적 테스트 + 전체 게이트(fmt/clippy/전수
테스트/golden/OVR) + WASM 빌드 → 작업지시자 실사용 판정 (심볼 placeholder
클릭 선택 → 더블클릭 → 그림 선택 → 표시 → 저장 → 재열기 유지).
