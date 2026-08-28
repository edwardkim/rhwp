# Task M100-4121 Stage 4 중간 체크포인트 — 통합 검증 안전 중단

## 중단 상태

2026-08-28 이동 요청에 따라 실행 중이던 전체 nextest와 `127.0.0.1:7700` Studio 개발 서버를
정상 종료했다. 테스트 실패로 중단한 것이 아니며, 아래 완료 증적 다음부터 재개한다.

## 완료한 검증

- 최신 branch 코어로 최적화 WASM 빌드 완료
  - `CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh --target web --out-dir pkg`
- 실제 Google Chrome에서 #4121 Stage 2~4 E2E 50/50 통과
  - 기존 mouse drag, Shift 선택, scroll-in 반복 페이지 투영
  - 선택 delete/typing/IME/paste/copy/cut/부분 서식과 Undo/Redo
  - 4페이지 Both 머리말의 전 페이지 투영
  - Odd/Even 꼬리말의 같은 정의 페이지만 투영 및 다른 정의 클릭 전환
  - 다문단 copy/cut/paste/부분 서식
- `cargo fmt --all`, `cargo fmt --all -- --check` 통과
- Rust unit tier 검사 4,221 tests / 299 modules 통과
- Rust integration suite manifest 1,015 sources / 최소 6,559 cases 통과
- E2E manifest tracked 121 / rows 121 통과
- `cargo clippy --locked --all-targets -- -D warnings` 통과
- `git diff --check` 통과

로컬 E2E 증적은 다음 ignored 산출물에 있다.

- `rhwp-studio/e2e/screenshots/issue4121-stage4-both-header-multiline-selection.png`
- `rhwp-studio/e2e/screenshots/issue4121-stage4-odd-even-footer-switch.png`
- `output/e2e/header-footer-selection-issue4121-report.html`

## 중단한 검증

다음 전체 회귀는 release-test 바이너리 빌드 중 사용자의 이동 요청으로 `Ctrl+C` 종료했다.
실패 판정이 아니며 재개 시 같은 target을 사용한다.

```bash
cargo nextest run --locked \
  --cargo-profile release-test --target-dir target/pr-review \
  --tests --no-fail-fast
```

## 재개 순서

1. 위 전체 nextest를 완료한다.
2. `npm --prefix rhwp-studio test`와 `npm --prefix rhwp-studio run build`를 실행한다.
3. 필요하면 아래 서버를 다시 실행해 사용자 수동 확인을 받는다.

   ```bash
   cd rhwp-studio
   npm run dev -- --host 127.0.0.1 --port 7700
   ```

4. 수동 확인 결과와 전체 게이트를 `mydocs/report/task_m100_4121_report.md`에 기록한다.
5. 오늘할일을 갱신하고 Stage 4 최종 커밋을 만든다.

원격 push, PR 생성 및 #4121 close는 아직 수행하지 않는다.
