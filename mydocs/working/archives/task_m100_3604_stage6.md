---
kind: working
status: completed
issue: 3604
stage: 6
last_verified: 2026-08-01
---

# #3604 Stage 6: CLI·Studio 출력 암호화 경로

## 목표

- `rhwp convert`의 HWP5 출력과 `rhwp export-hwpx`의 HWPX 출력에 암호를 설정한다.
- 입력 암호와 출력 암호를 분리해 재암호화와 형식 변환을 모두 지원한다.
- 저장 뒤의 `--verify`와 `--verify-pages`가 출력 암호를 사용해 실제 재열기를 수행하게 한다.

## 구현 계획

1. 전역 입력 암호 옵션과 별도로 `--output-password`, `--output-password-stdin`을 추가한다.
2. 두 stdin 옵션을 같이 쓸 때 입력 암호를 첫 줄, 출력 암호를 둘째 줄로 읽도록 고정하고 도움말에 기록한다.
3. `DocumentCore`와 WASM facade에 HWP5/HWPX 암호 저장 entrypoint를 추가한다. HWP 출력은 기존 HWPX-to-HWP adapter를 반드시 거친다.
4. `convert`와 `export-hwpx`가 출력 암호가 있으면 새 entrypoint를 사용하고, 검증 재열기도 해당 암호로 연다.
5. `rhwp-studio` 저장 흐름에 암호 설정과 새 암호/확인 dialog를 추가한다. browser session에는 암호 값 대신 보호 저장 여부만 유지하고 다음 저장 시 재입력받는다.
6. 실제 HWP5/HWPX fixture에서 평문 열기 거부, 올바른 암호 재열기, 잘못된 암호 거부를 CLI와 Rust/Studio 테스트로 확인한다.

## 범위와 보안

- 이 Stage는 rhwp CLI, public Rust facade, rhwp-studio 저장 UI를 변경한다. MCP 전달과 server 후처리는 다음 hwp-convert Stage에서 처리한다.
- 암호는 stdout JSON, 오류 문자열, 파일명, 환경 변수에 기록하지 않는다.
- PDF 출력 암호화는 범위 밖이다. HWP5/HWPX 파일 형식 암호화만 지원한다.

## 시작 상태

- Stage 5에서 세 파일 형식의 알고리즘을 `src/password_crypto.rs` 하나로 통합했고, HWP5/HWPX serializer의 password entrypoint를 만들었다.
- CLI는 `--password`와 `--password-stdin`으로 입력 암호 문서를 열 수 있지만, 출력은 항상 평문이었다.

## 테스트 결과

| 검증 | 결과 | 근거 |
| --- | --- | --- |
| `cargo check` | 성공 | 새 DocumentCore 암호 저장 entrypoint와 CLI가 컴파일됨 |
| `cargo test --bin rhwp --no-fail-fast` | 성공 | 전역 output password 제거·중복 거부 unit test를 포함한 CLI unit test 통과 |
| `cargo test --lib password_crypto::tests --no-fail-fast` | 성공 | HWP5 AES bit-CFB 외부 vector 및 partial block round-trip 통과 |
| HWP5 평문 → `convert --output-password-stdin --verify-pages` | 성공 | 보호 HWP5 생성, 평문 열기 거부, 올바른 암호 `info` 재열기 확인 |
| HWPX 평문 → `export-hwpx --output-password-stdin --verify-pages` | 성공 | 보호 HWPX 생성, 평문 열기 거부, 올바른 암호 재열기 확인 |
| 암호 HWPX → 두 stdin 옵션 `export-hwpx --verify-pages` | 성공 | stdin 첫 줄 입력 암호, 둘째 줄 출력 암호로 재암호화; 이전 암호 거부와 새 암호 재열기 확인 |
| `wasm-pack build --target web --out-dir pkg` | 성공 | 브라우저용 두 password export binding 생성 |
| Studio 암호 저장 test 10건 | 성공 | dialog 확인·DOM 초기화, HML 거부, 저장 상태, public binding 계약 통과 |
| `node --test scripts/frontend-wasm-bindings.test.mjs` | 성공 | 모든 명시 WASM export가 생성 type declaration에 존재 |
| `rhwp-studio: npm run build` | 성공 | TypeScript와 Vite production bundle 통과 |
| `cargo fmt --check`, `git diff --check` | 성공 | 형식과 whitespace 검사 통과 |

실제 CLI 재열기 검증에는 정상 HWP와 HWPX fixture를 각각 사용했다. 보호 결과는 평문
`info`가 실패하고 암호 stdin `info`가 성공해야 통과로 판정했다. 최소 HWPX fixture 하나는
암호화 전 평문 serializer가 `styleIDRef: [0]` 미등록 참조로 거부하므로, 암호화 결과의
회귀 판정에는 사용하지 않고 정상 HWPX fixture로 교체했다.

## 구현 결과

- `DocumentCore::export_hwp_with_adapter_with_password()`는 HWPX source adapter를 적용한 뒤
  HWP5 EncryptVersion 4 serializer를 호출한다.
- `DocumentCore::export_hwpx_native_with_password()`는 기존 HWP5-origin HWPX 보조 entry와
  line segment materialization 규칙을 유지한 채 ODF password serializer를 호출한다.
- CLI는 output password 존재 여부만 JSON envelope의 `passwordProtected`로 표시하며 암호 값은
  출력하지 않는다.
- manual은 입력·출력 암호 stdin 순서와 `convert`/`export-hwpx` 범위를 현행화했다.
- Studio는 HWP/HWPX 저장 대화상자의 암호 설정 선택에서 새 암호·확인 dialog를 열고 해당
  serializer만 호출한다. HML에는 암호 설정 control을 제공하지 않는다.
- Studio는 암호 문자열을 브라우저 저장소, 파일명, 로그, 다음 저장 상태에 보관하지 않는다.
  보호 저장 여부 boolean만 유지하고, 다음 Ctrl+S에서 새 암호와 확인을 다시 입력받는다.
- public WASM JavaScript와 type declaration에도 두 password export binding을 추가했다.

## 다음 Stage

`hwp-convert`의 별도 local_012 Stage 1에서 MCP server, stdio bridge, direct CLI가
`output_password`를 전달하고 Hancom 결과를 `rhwp --output-password-stdin`으로 후처리하도록
구현·mock 검증을 완료했다. 다음 운영 Stage에서는 최신 rhwp binary를 포함한 MCP 배포본으로
실제 Hancom 변환 결과의 password-protected output을 재열기까지 확인한다.
