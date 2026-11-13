# #8946 최종 제출 전 증적

최종 코드 `6b7ceb9cd5`와 최신 base `4c876654d8`의 결과다.
[수용 조건·원인·검증 기록](../../../../working/task_m100_8946_stage5.md)을 함께 읽는다.

- `visual-results.json`, `hwpx-*-full.tsv`, `hwp3-*-full.tsv`: Native/fresh WASM 각 763쪽,
  96 DPI, 색상 경계 보정 전 2px raw. 같은 원본·글꼴·외부 그림의 독립 한컴 Print와 대조했다.
- `host-visual-results.json`, `host-*-full.tsv`: 실제 프로덕션 Canvas2D의 각 763쪽. 그림을 제외하지 않는다.
- `producer-check.json`, `environment.json`, `host-*-runtime-assets.json`: 실제 최적화 산출물과
  새 프로필의 첫 JS/WASM 응답. `visual-results.json`에는 보존한 Native 실행 파일의 해시도 있다.
- `default-mode-parity.json`: 원본 HWPX·그림 포함 HWPX·HWP3 각각의 none/omitted/false,
  각 763쪽 base/후보 tree와 실제 PNG의 동일성.
- `final-rebase-parity.json`, `host-hwpx-rebase-parity.json`: 앞 단계의 직접 확대 판독을
  재사용하는 근거. Native/fresh WASM 네 묶음과 실제 HWPX 호스트의 전쪽 출력이 동일하다.
- `rowbreak-*.tsv`, `rowbreak-final-parity.json`: 최신 devel로 반영한 #8945의 실제 5개 입력·8쪽.
- `handbook-gate.json`, `handbook-native-full.tsv`: 별도 필수 편람 383쪽 gate passed, 기준 하락 0.
- `edit-*.json`, `edit-*-after.png`: 실제 원본의 삽입·삭제, 빈 문단을 포함한 소유·점유·다음 내용,
  저장 후 새 bridge에서 재열기 관측. 삭제를 undo 또는 저장 LineSeg 복원과 동일시하지 않는다.
- `host-*-first-paint.json`, `host-*-result.json`: 딸기 원본 파일의 indexed ID, 실제 readFace,
  Rust 환경·nativeOtf 측정·준비 완료 뒤 첫 paint 연결. trace의 unsupported paint를 증거로 쓰지 않는다.
- `e2e-layout-contracts.json`: 두 backend #7403 E2E 중 Canvas2D의 97개 조판·수명·저장 계약.
- `minimal-font-evidence.json`: 회귀 실행에 실제 노출된 OS·공개 fixture와 개인 글꼴 차단 증거.

사설 글꼴 프로그램, 사용자 절대 경로, 브라우저 프로필, 원시 테스트 로그는 포함하지 않았다.
독립 PDF 출처·원본 그림 3개의 동등성은 [단계 4](../../../../working/task_m100_8946_stage4.md)를 따른다.
큰 PDF의 저장소 제외 계약을 바꾸지 않으며 기존 PDF와 SHA-256을 재사용한다.

핵심 재현은 라이선스가 확인된 동일 글꼴을 `FONT_DIR`에 준비한 뒤 아래 명령을 실행한다.
T7 및 공용 Cargo target 설정은 로컬 저장 정책을 먼저 따른다.

```sh
node scripts/rust-test-suite-manifest.mjs --prepare
cargo nextest run --locked --cargo-profile release-test --tests --test-threads 8 --no-fail-fast
cargo test --locked --profile release-test --features native-skia --lib
node scripts/run-rust-test.mjs issue_2225_missing_picture_placeholder -- --cargo-profile release-test --features native-skia
node scripts/run-rust-test.mjs render_p37_direct_pdf_export -- --cargo-profile release-test --features native-skia
```

전체 회귀는 개인 글꼴을 차단한 최소 글꼴 환경에서 실행한다. 기존 SVG 테스트가 스스로 생성하는
임시 fixture만 허용한다. Native/fresh WASM 시각 검증은 `scripts/visual_sweep.py --silhouette-only`,
HWPX `--compat 2024`, HWP3 `--compat 2022`, 같은 `--font-path`를 사용한다.
WASM에는 fresh `--wasm-pkg pkg`와 세 `--linked-image samples/<name>`을 공급한다.
편람은 같은 시각 격리 환경에서 CLI 웹폰트 캐시도 T7의 허용 경로로 설정한 뒤 `--handbook-gate`를
실행한다. 공개 웹폰트의 실제 선택 환경은 `d68b9023…`다.
