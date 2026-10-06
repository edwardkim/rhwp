# PR #2076 검토 — 도형 restrictInPage 하단 클램프 (lpaiu-cs)

- 이슈: #2075 / mergeable: MERGEABLE / CI: pass (WASM skip 은 정상) / 작성일: 2026-07-09

## 검토

- **머지된 #2033(그림 하단 클램프)의 도형 경로 짝 정정** — "layout_body_picture 와 동일
  로직" 주석과 달리 미갱신이던 누락을 해소. [[feedback_fix_scope_check_two_paths]]·
  [[feedback_image_renderer_paths_separate]] 축과 정합.
- 가드가 좁음(`!treat_as_char && flow_with_text && vert_rel_to==Para` + 하단 초과 시만
  활성 no-op 클램프). restrictInPage=off 대조 테스트 포함, 수정 전 실패 증명 제시
  (y=1332.3 > 페이지 1123 소실 → 쪽 안 배치).
- 접촉 파일 `shape_layout.rs` — 우리 리팩토링(R10~13)과 무접촉, 충돌 없음.

## 처리안

로컬 CI급 검증(신규 테스트 포함 + OVR) → **approve + merge** (시각 판정은 #2033 전례상
클램프 no-op 특성이라 표적 테스트로 갈음 — 선택 적용 거버넌스).
