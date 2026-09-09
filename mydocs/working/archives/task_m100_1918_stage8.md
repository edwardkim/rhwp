# Stage 8 완료 보고서 - Task #1918

**이슈**: #1918 표 입력/삭제 시 워터마크·정적 이미지 레이어 포함 페이지가 매 입력마다 고비용 재렌더링됨
**브랜치**: `local/task1918`
**작성일**: 2026-07-05

---

## 1. 목적

Stage 7에서 빠른 연속 입력 성능은 충분히 개선되었지만, PR 전 리뷰에서 text-edit fast path의
부작용 가능성이 두 가지 남았다.

1. 같은 셀 `insertText`/`deleteText`가 항상 페이지 로컬 변경이라는 보장은 없다.
2. 정적 레이어 재사용은 개체 위치 변화까지 즉시 감지하지 못할 수 있다.

Stage 8은 이 PR 안에서 처리할 수 있는 최소 안전장치를 추가하고, 더 큰 일반화 작업은 후속 이슈 후보로
분리하기 위한 마감 단계다.

## 2. 구현 내용

- `rhwp-studio/src/engine/input-edit-invalidation.ts`
  - page-local text edit 최대 길이를 `MAX_PAGE_LOCAL_TEXT_EDIT_CHARS = 8`로 제한했다.
  - 삽입 텍스트가 줄바꿈/탭을 포함하거나 길이가 큰 경우 full refresh로 보낸다.
  - 삭제 길이가 큰 경우 full refresh로 보낸다.
  - 편집 전후 커서 page index가 바뀌면 full refresh로 보낸다.
- `rhwp-studio/src/engine/command.ts`
  - `InsertTextCommand`, `DeleteTextCommand`가 page-local 판정용 payload hint를 노출하도록 했다.
- `rhwp-studio/src/engine/input-handler.ts`
  - command 실행 전후 page index와 payload hint를 page-local 판정에 전달한다.
- `rhwp-studio/src/view/page-renderer.ts`
  - text-edit 렌더가 static layer 검증을 필요로 하는지 `PageRenderResult`로 반환한다.
- `rhwp-studio/src/view/canvas-view.ts`
  - 정적 overlay/static flow를 재사용한 text-edit 렌더 후 800ms idle 검증 렌더를 예약한다.
  - 새 text-edit, full refresh, zoom/resize/reset, canvas release에서는 검증 타이머를 취소한다.
  - 검증 렌더는 `reason: 'unknown'`, `allowStaticOverlayReuse: false`로 실행되어 stale layer를 회복한다.
- 테스트
  - 긴 paste, 줄바꿈/탭 삽입, 큰 삭제, 페이지 이동 가드 테스트를 추가했다.
  - CanvasView idle 검증 렌더 계약과 PageRenderer 결과 계약을 추가했다.

## 3. 검증

통과한 명령:

```bash
cd rhwp-studio
npm test
npm run build
```

```bash
CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" \
node e2e/renderer-contract.test.mjs --mode=headless
```

빠른 입력 probe:

```bash
CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" \
node /private/tmp/rhwp_task1918_rapid_input_probe.mjs --mode=headless
```

20회 연속 입력 + `document-page-invalidated` 결과:

| 샘플 | Stage 7 key 평균 | Stage 8 key 평균 | Stage 8 filtered render |
|------|------------------|------------------|--------------------------|
| `samples/복학원서.hwp` | 0.12ms/key | 0.24ms/key | `flow` 2회, `background/behind/front` 각 1회 |
| `samples/253E164F57A1BC6934-empty.hwp` | 0.04ms/key | 0.13ms/key | `flow-dynamic` 1회, `flow-static` 1회, idle `flow` 1회 |
| `samples/143E433F503322BD33.hwp` | 0.09ms/key | 0.24ms/key | `flow-dynamic` 1회, `flow-static` 1회, idle `flow` 1회 |
| `samples/통합재정통계(2011.10월).hwp` | 0.30ms/key | 0.15ms/key | `flow` 1회 |

Stage 8에서 정적 레이어 문서는 입력이 잠잠해진 뒤 검증 렌더가 추가되지만, key handler 평균은 여전히
1ms 미만이다. 따라서 Stage 7에서 제거한 입력 backlog 병목은 재도입되지 않았다.

## 4. 남은 후속 이슈 후보

이번 PR 범위에서는 일반화 설계를 넣지 않았다. 후속 이슈로 분리할 후보는 다음과 같다.

- 같은 셀 편집이 실제 페이지 로컬인지 layout/page signature로 판정하는 일반화
- static layer summary key에 bbox/signature를 포함해 위치 변화까지 감지하는 정밀 캐시 키
- 다양한 HWP fixture 기반의 자동 성능/시각 회귀 probe 정식화

## 5. 판단

Stage 8은 PR 전 안전성 마감으로 충분하다.

긴 paste/페이지 이동/큰 삭제는 full refresh로 되돌렸고, 정적 레이어 재사용은 idle 검증 렌더로 회복 경로를
갖게 되었다. 더 정밀한 레이아웃 diff 기반 일반화는 이 PR의 성능 개선 범위를 넘어서므로 후속 이슈로
분리하는 것이 적절하다.
