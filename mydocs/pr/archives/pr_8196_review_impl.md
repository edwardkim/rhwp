# PR #8196 구현 검토

검증 code SHA: `1a5cea08307132d3deba18a1a01b85869717b726`. 최종 판정과 CI는 [self-review](pr_8196_review.md)에 있다.

| 경계 | 실제 생산·소비 경로와 판정 |
| --- | --- |
| Undo 뒤 선택 | InputHandler가 document-layout-refreshed에서 현재 선택을 읽고 확정 쪽 원점·배율로 다시 투영한다. 일반 경로는 스크롤을 바꾸지 않고 기존 revealCaret 경로를 유지한다. |
| renderer 준비 | CanvasView의 loadDocument/selectNextDocumentRevision/selectMutationRevision → prepareRendererRevision → RendererSession.resolve 및 WasmBridge.prepareCanvasMetrics → applyRendererSelection → refreshPages/replay다. |
| 현재 준비 실패 | current epoch 확인 → invalidateCanvasMetrics → RendererSession.fallbackFromPreparationFailure → portable Canvas2D apply → 실제 페이지 repaint다. 반복 준비 호출 없이 pending을 해제한다. |
| stale·dispose 실패 | epoch/dispose guard가 fallback과 pending 변경 전에 반환한다. 새 준비가 대기 중이거나 이미 성공한 양쪽을 검사했다. 문서 교체도 검사했다. |
| backend context | PageRenderer.configure 변경 시 CanvasPool의 active/available canvas를 폐기한다. WebGL canvas를 2D로 재사용하지 않으며 같은 backend 재사용은 유지한다. |
| quiet·zoom transaction | 오래된 quiet timer를 취소하고 최종 page-view 배치가 끝난 뒤 선택 이벤트를 보낸다. 실제 줌 대화상자로 확인했다. |

실패 주입은 실제 클래스와 실제 Chrome/WASM을 사용했다. 자연 발생한 특정 사용자 파일의 오류로
보고하지 않는다. 내부 계산식을 복제한 mock 통과만으로 복구를 판정하지 않았으며 실제 표시 canvas와
본문·선택 보존을 검사했다. 기준값·허용치 완화는 없다.

원 구현 #2394/#7891/#6637/#6987, 통합 보정, 이전 재현 증거는
[기존 보고서](../../report/task_m100_8103_report.md)에 연결돼 있다.
보정 전 P2의 SSR base/head 비교와 실제 브라우저 실패 로그는 ignored `output/pr-review/8196/`에 보존했다.
소스·테스트 변경을 먼저 커밋하고 CI 성공 뒤 이 검토 문서만 trailing commit으로 기록한다.
