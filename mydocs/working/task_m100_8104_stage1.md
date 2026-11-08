# #8104 Studio 오류 계약 구현·검증

Issue: #8104. 기준 `upstream/devel`: `6097253468`.

## 구현 근거와 변경

#3350 원 PR #3656(`9daeb0bb8`), 통합 #3661(`e5d3049199`),
후속 #3662의 source guard 강화(`2deb282081`)를 대조했다.
기존 before 복원·snapshot ID 해제·원 오류 전파와 이중 실패 AggregateError를 유지한다.
새 `failureRecovery`는 해제까지 성공한 뒤에만 restored다. 기존 source guard도
restore/discard/rethrow를 그대로 검사하며 그 사이의 outcome 기록을 추가로 확인한다.

공통 InputHandler 경계에서 snapshot outcome과 커서·선택 checkpoint, 양 history 스택,
dirty 상태를 결합한다. 실제 visible page render를 기다리고 커서 질의를 엄격하게 실행한다.
일반 cursor 조회의 fallback은 그대로 두되 복구 인증에는 사용하지 않는다. IME·지연 pagination,
부분 변경, undo/redo 실패와 trap은 검증된 성공으로 간주하지 않는다.

버전 계약·수신 검증기·직접 EventTarget/MessagePort·typed failure를 구현했다.
`eventId`로 재전파와 이벤트/응답 중복을 식별하고 문서 generation으로 오래된 완료를 분리한다.
오류 처리 중인 같은 세션의 추가 공통 명령은 거부한다. 호스트 raw 편집·저장 경계 보호와
실제 앱 번들 반영·배포는 #594에서 별도로 수행한다.

## 검증

- Node 24.15.0, Rust 1.93.1, macOS arm64, 로컬 Google Chrome headless.
- `scripts/wasm-pack-locked.sh --target web --out-dir pkg --dev`: PASS.
  초기 로컬 pkg 타입이 현재 devel보다 오래돼 `getTableCellBboxesAtPage`가 없었고 fresh 빌드로 해소했다.
- TypeScript와 production build: PASS. bundle 크기 안내 경고는 별도 기록.
- 전체 npm test: 2,440 PASS / 0 FAIL / 0 skip.
- 브라우저: 본문 clean/dirty rollback, mutation 전 거부, rollback 실패, 부분 변경, trap,
  render 실패, cursor 조회 실패, 실제 문서 교체 중 늦은 완료, 실제 global error 라우팅,
  MessagePort 호스트와 중복·미지원·잘못된 payload를 검증했다.
- 정상 대조군 #8102 셀 생성·클릭·키보드·undo 진입·legacy flat·다중 문단 선택 삭제 회귀: PASS.
- 생성된 `rhwp-studio/public/rhwp.js`는 검증용 fresh WASM glue이며 소스 변경에 커밋하지 않는다.

중간 로그·JSON·브라우저 HTML은 ignored `output/pr-review/issue-8104/` 및 `output/e2e/`에 둔다.
최종 code SHA와 최종 판정은 결과보고에 기록한다. Rust 조판·모델·serializer·정상 출력 규칙을
변경하지 않았으므로 Native/fresh WASM의 독립 PDF Visual Sweep은 이번 오류 계약의 검증 게이트가 아니다.
새 렌더링 fixture나 픽셀 기대값을 추가하지 않았다. 기존 정상 셀 회귀는 실제 Chrome으로 대조했다.
