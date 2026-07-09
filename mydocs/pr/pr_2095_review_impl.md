# PR #2095 검토 처리 계획

- 대상 PR: https://github.com/edwardkim/rhwp/pull/2095
- 검토 기준 SHA: `86b0be18b64cfc8cc7b9b6f86f20d6be2ffc6eb5`
- 상태: 수정 요청 대기
- 작성일: 2026-07-09

## Stage 1 — 리뷰 확인

- reviewer assign 완료.
- PR diff 및 설명 확인 완료.
- 렌더/PDF visual sweep 대상은 아님. 다만 WASM/Studio 편집 저장 결과에 직접 영향을 주므로 브라우저 런타임 검증 대상으로 분류.

## Stage 2 — 로컬/CI 검증

- GitHub Actions 통과 확인.
- Rust focused test, lib test, fmt, clippy 완료.
- `wasm-pack build --target web --out-dir pkg` 완료.
- 기존 `localhost:7700` Vite dev server 재사용.
- Puppeteer headless Chrome 으로 실제 Studio 앱의 `window.__wasm` 경로 검증 완료.
- 지정 샘플 `samples/issue1937_rowbreak_footnote_overpagination.hwp` 4쪽 표 row insert 추가 검증 완료.

## Stage 3 — 결론

- blocking finding 1건으로 수정 요청.
- GitHub review/comment 는 작업지시자 승인 전 제출하지 않는다.
- 작성자 수정 후 재검증 항목:
  - `wasm-pack build --target web --out-dir pkg`
  - `samples/issue1937_rowbreak_footnote_overpagination.hwp` 7번 문단 뒤 새 문단 삽입: 단일 글자모양 `87` 보존 확인
  - `samples/issue1937_rowbreak_footnote_overpagination.hwp` 4쪽 표 row insert: 기존 body row 글자모양 보존 확인
  - `samples/issue1937_rowbreak_footnote_overpagination.hwp` 179번 문단 offset 10 표 삽입: cursor 위치 글자모양 `37` 이 새 셀 문단에도 보존되는지 확인
  - 관련 Rust 회귀 테스트
  - GitHub Actions 최신 head 통과

## 후속 조건

- 수정 반영 전 merge 보류.
- 수정 반영 후 증적 HWPX 를 다시 갱신하거나, 더 적합한 자동 회귀 테스트가 추가되면 현재 증적은 실패 재현 자료로 유지한다.
