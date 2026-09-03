# Task M100 #6041 Stage 2 보고 — Canvas 통합과 페이지별 surface 보정

- **상태**: 구현·검증 완료
- **code candidate**: `e37d483fd`

## 결과

- CanvasView가 retained/visible/focus 변화마다 plan을 갱신하고 DPR/tier가 바뀐 active 페이지만 다시 그린다.
- `PageRenderer`의 콘텐츠 plane 요약으로 페이지별 Canvas surface 수를 계산한다.
- 최초 candidate의 고정 4-layer 추정은 단일 Canvas 일반 문서를 4배 과대 계산했다. 실제 DOM 조사로 이를
  발견해 페이지별 `layerCount` 입력으로 교체했다.
- backend/profile, 문서/revision과 flow split 실패 경계에서 layer count 캐시를 무효화한다.
- 포커스 페이지와 CanvasKit/출력 profile의 기존 해상도 경로는 보존한다.

## 검증

- `node --test tests/render-surface-budget.test.ts`: 13 pass
- `npx tsc --noEmit`: pass
- `npm test`: 1,290 pass, 1 skip, 0 fail
- `npm run build`: 244 modules pass, 기존 대형 chunk 경고만 확인
- `git diff --check`: pass
