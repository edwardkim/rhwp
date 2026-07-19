# #2318 최종 결과보고 — 바탕쪽 개체가 본문 텍스트를 가림 (plane 분류 정정)

- 이슈: [#2318](https://github.com/edwardkim/rhwp/issues/2318)
- 마일스톤: M100 (v1.0.0) / 브랜치: `local/task2318`
- 계획서: `mydocs/plans/task_m100_2318.md` (버그 정정형, 구현계획서 생략)
- 단계 보고: `mydocs/working/task_m100_2318_stage{1..4}.md`

## 결론

**시각 판정 통과** (작업지시자, 2026-07-17 — skia PNG + wasm/studio 모두).
shortcut.hwp 의 바탕쪽 쪽번호 "1" 이 본문 "Ctrl+Y"/"Alt+Y" **뒤**에 깔려
한컴 실기와 일치한다.

## 근인과 정정

- **근인**: 바탕쪽 글상자의 저장 wrap = InFrontOfText(글 앞으로). paint replay
  plane 분류기가 이를 본문 기준으로 해석해 front plane 으로 승격 →
  plane 재생 backend 전체(studio 다층 canvas 합성 + skia per-plane 재생)에서
  본문 텍스트를 가림. 착수 정찰의 "skia 정상" 은 오판이었고 skia 도 결함
  범위였다 (2단계 실측). SVG 만 `node_z_plane` 의 MasterPage plane 1 직접
  배치(#1167)로 원래 정상.
- **한컴 의미론**: 바탕쪽 개체의 wrap 은 바탕쪽 **내부** 순서에만 적용되고,
  바탕쪽 전체는 항상 본문 뒤.
- **정정**: `RenderLayerInfo.master_page` provenance 필드 + 분류기 단일 지점
  cap(BehindText 상한, Background 제외). rust 소비자(web_canvas/skia/
  canvaskit_policy/plane 집계)는 layer 상속 경유 자동 반영. studio TS 는
  공유 분류기(`replay-plane.ts`)에 동일 cap + `page-renderer.ts` 로컬 중복
  제거로 단일 진실 원천화. 바탕쪽 내부 정렬(원본 wrap)과 기존 전지 배경
  강등 가드는 불변.

## 산출물

| 구분 | 내용 |
|------|------|
| rust | `render_tree.rs`(필드) / `replay_order.rs`(cap) / `layout.rs`(provenance 부여 2지점) / `json.rs`(masterPage 방출) |
| studio | `types.ts` / `canvaskit/replay-plane.ts`(cap) / `page-renderer.ts`(중복 제거) |
| 테스트 | `tests/issue_2318_master_page_plane.rs` 2건 (red→green + wrap 보존 가드), studio cap 단위 6단언, `e2e/issue-2318-master-page-zorder.test.mjs` 4단언 |

## 게이트

- `cargo test --tests` release-test 전 통과 / `--lib` 2280 / fmt --all / clippy 0
- studio 307/307 + tsc / WASM 빌드 / CDP e2e 4/4
- SVG 계약 불변 (issue_1167/1197, svg_snapshot 등 green)
- 머리말/꼬리말 유사 증상: 380개 샘플 전수 스윕 노출 **0건** — 대비 샘플
  부재로 추정 정정 금지 원칙에 따라 범위 제외 (샘플 확보 시 별도 이슈)

## 남은 항목

없음. devel merge 후 이슈 close (승인 필요).
