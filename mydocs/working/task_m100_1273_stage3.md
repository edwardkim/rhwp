# Task M100 #1273 Stage 3 완료 보고서

E2E lifecycle 테스트 + 테스트 커버리지 감사 + 재발 방지 문서.

## 1. 테스트 커버리지 감사 (선행)

우리가 수정한 영역(InputHandler 드래그 경로·Move/Resize 커맨드 by-path)을 기존 테스트와 매핑한 결과 **명확한 공백** 확인:

| 레이어 | 기존 테스트 | 커버 |
|--------|-------------|------|
| WASM by-path API **직접 호출** | `textbox-picture-1171.test.mjs`, Rust `issue_1171_textbox_picture_cellpath.rs` | round-trip·hit-test |
| 삽입 | `textbox-picture-insert-1171.test.mjs` | 본문 sibling 삽입 |
| **InputHandler 드래그 lifecycle** | **없음** | — |
| **Move/Resize 커맨드 by-path·undo/redo** | **없음** | — |

기존 E2E 는 `wasm.setCellPicturePropertiesByPath` 를 **직접** 호출하여 드래그 staging 경로를 우회 → 버그가 있어도 통과(교훈 #3).

## 2. 신규 E2E — `rhwp-studio/e2e/textbox-picture-ops-1273.test.mjs`

InputHandler 의 **실제 드래그 경로**를 구동(onClick → mousemove → mouseup, 실제 핸들 좌표).
대상: `tac-img-02.hwp` 섹션0 문단25 글상자 picture(cellPath sentinel, 페이지5).

검증 항목:
- **리사이즈**: se 핸들 드래그 → `pictureResizeState.ref.cellPath` 보존(핵심 회귀) + 폭 15040→18038 by-path 반영 + **undo 원복**(→15040).
- **회전**: rotate 핸들 드래그 → `pictureRotateState.ref.cellPath` 보존 + 각도 0°→29° 반영.
- **공통**: 조작 중 콘솔 오류('실패/범위 초과/그림이 아님') **0건**.
- 대상 picture 는 `treat_as_char=true`(글상자 내 인라인)이라 이동 드래그는 N/A — 이동 by-path(Stage 2)는 리사이즈 undo 의 `setCellPicturePropertiesByPath` 동일 경로로 간접 커버.

### 구현 메모 (하니스 특이점)
- 합성 MouseEvent 에 `target`(=container) 명시 — `onClick` 의 `e.target.closest('#menu-bar')` 가드 대응.
- 대상 picture 가 페이지5(오프스크린)라 `onClick` 의 스크롤바영역 가드(`localY >= clientHeight`)에 막힘 → `ih.container.scrollTop` 으로 핸들을 뷰포트 안으로 스크롤 후 클릭.

## 3. red-green 검증

- **green**: 현재 코드(Stage1+2)로 신규 테스트 전 항목 PASS.
- **red**: Stage1 의 리사이즈 ref 보존을 임시로 되돌리면, `stateCellPath: null` + 폭 미변경 +
  콘솔 `개체 리사이즈 실패: 렌더링 오류: 지정된 Shape 컨트롤이 그림이 아닙니다`(사용자 보고와 동일)로 FAIL.
- 즉 본 테스트는 회귀를 실제로 잡는다. 복원 후 재실행 PASS 확인.
- **회귀 안전**: 기존 `textbox-picture-1171`/`textbox-picture-insert-1171` E2E 모두 PASS 유지.

## 4. 재발 방지 문서

`mydocs/troubleshootings/textbox_picture_drag_ops_cellpath_1273.md` 신설 —
"드래그 상태 staging ref 소비처 누락" 사례 + 체크리스트(공유 ref 소비처에 staging 포함, 위치 식별자 전부 복사, 검증은 연산 표면으로). 메인테이너 `nested_picture_selection_ref_consumers_1171.md` 교훈 #2/#3 의 심화.

## 5. 실행 방법

```bash
cd rhwp-studio
npx vite --host 127.0.0.1 --port 7700 &
CHROME_PATH="<chrome>" VITE_URL=http://127.0.0.1:7700 \
  node e2e/textbox-picture-ops-1273.test.mjs --mode=headless
```

수동 시각 확인(글상자/머리말꼬리말 picture 드래그)은 작업지시자 환경에서 권장.
