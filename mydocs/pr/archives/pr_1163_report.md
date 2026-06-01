# PR #1163 검토 — PageLayerTree BehindText/InFrontOfText z-order 합성 순서

## 1. PR 정보

| 항목 | 내용 |
|------|------|
| 번호 | #1163 |
| 제목 | Task #1017: PageLayerTree BehindText/InFrontOfText z-order 합성 순서 적용 |
| 작성자 | [@postmelee](https://github.com/postmelee) (Taegyu Lee) — 기존 핵심 컨트리뷰터 |
| base ← head | `devel` ← `postmelee:local/task1017` (cross-repo) |
| 상태 | OPEN, **CONFLICTING / DIRTY** |
| 변경 | +1814 / -22, 18 파일 |
| 라벨 | enhancement / 마일스톤 v1.0.0 |
| 연결 | Closes #1017 (#1016 resolved payload 후속) |
| CI | 이전 SHA 기준 전부 pass (Build&Test, Analyze rust/js/py, Canvas visual diff, CodeQL) |

## 2. 컨트리뷰터 이력

@postmelee 누적: #887/#939/#947/#976/#979/#982/#1018/#1019(merged, #975 워터마크 톤)/#1051. 워터마크·복학원서·PageLayerTree replay 영역 핵심 기여자. 본 PR 은 #1019(#975) → #1016 → #1017 z-order 연속선.

## 3. 변경 내용

PageLayerTree paint op 를 `background → behindText → flow → inFrontOfText` replay plane 으로 분류하는 공통 helper 도입, replay 경로를 multi-pass 로 변경.

- `src/paint/replay_order.rs` (신규): `PaintReplayPlane` + `paint_op_replay_plane()`
- `src/paint/mod.rs`: 모듈 노출
- `src/renderer/skia/renderer.rs`: native Skia root replay 를 plane 별 multi-pass 로
- `rhwp-studio/src/view/canvaskit-renderer.ts` + `canvaskit/replay-plane.ts`: CanvasKit direct replay 동일 plane 순서
- `src/renderer/canvaskit_policy.rs`: replay plan 에 `replayPlane` 진단 노출
- `tests/issue_1017.rs`, `rhwp-studio/tests/render-backend.test.ts`: #1017 회귀 테스트
- 문서: plans/working/report (stage1~6) + orders

## 4. Root cause (PR 기재 + 검증)

#1016 에서 중앙 baked watermark payload 는 resolved 됐으나, `samples/복학원서.hwp` 에서 해당 image op 가 `wrap=behindText` 임에도 raw PageLayerTree 순서상 본문 textRun **뒤**에 위치. raw order 를 따르는 replay 는 behindText 워터마크를 본문 위 흰 사각처럼 합성. → plane 분류로 z-order 의미 복원.

## 5. 검증 결과

| 경로 | 결과 |
|------|------|
| **PNG (native Skia)** | ✅ 정상 — 작업지시자 시각 판정 통과. 워터마크가 본문 뒤로 |
| **웹캔버스 (CanvasKit)** | ✅ 정상 — 작업지시자 시각 판정 통과 |
| **SVG (`svg.rs`)** | ❌ **결함 잔존** — 아래 6절 |

## 6. ⚠️ SVG 경로 z-order 누락 (후속 처리 대상)

`svg.rs` 는 PaintOp replay 가 아니라 **RenderNode 트리 DFS 순회**로 렌더링하는 별도 아키텍처. PR 의 replay plane 은 PaintOp 경로(skia/canvaskit)에만 적용되어 SVG 는 영향 없음.

정량 검증 (`output/poc/pr1163/after/복학원서.svg`, PR 적용 후):
- 중앙 워터마크 `<image>` 가 줄 810 — 본문 `<text>` 범위(47~807) **뒤**
- SVG 는 후순위가 위 → 워터마크가 본문 텍스트를 덮음

메모리 룰 `feedback_image_renderer_paths_separate` (svg/web_canvas/skia 별도 사본, 정정 시 전 경로 점검) 해당. 근본 원인: RenderNode 트리 생성(layout)에서 BehindText 워터마크가 본문 노드 뒤에 배치.

**처리 (작업지시자 결정)**: PR #1163 은 PNG/CanvasKit 정정 완성도 있고 CI pass → **머지**. SVG z-order 누락은 **별도 후속 타스크**로 정정.

## 7. 머지 절차 비고

- CONFLICTING 원인: `mydocs/orders/20260529.md` 단일 파일 충돌 (devel 의 #1156 _v2 / PR #1162 / .mailmap 작업일지 추가 vs PR 의 #1017 섹션 추가). 코드는 auto-merge clean (`src/renderer/skia/renderer.rs` 포함 충돌마커 0).
- orders 는 union(양쪽 작업일지 보존)으로 해결. fork push 권한 없음 → 메인테이너 로컬 충돌 해결 후 devel 직접 머지.

## 8. 판단

PNG/CanvasKit z-order 정정 + 회귀 테스트 충실. **merge 권장** (orders union 해결). SVG 후속 별도 타스크 등록.

---

## 9. 처리 결과 (보고)

- **MERGED**: devel `832497a4` (close #1017)
- orders union 해결, skia 테스트 회귀 정정(_v2 기준 불투명), z-order 테스트 유지
- 검증: cargo test --tests 92 스위트 + native-skia skia 32 + issue_1017 2 통과, fmt
- PR 코멘트 등록: 머지·정정·SVG 한계 안내
- **SVG 후속 이슈 등록: #1167** (BehindText 워터마크 z-order, v1.0.0)
