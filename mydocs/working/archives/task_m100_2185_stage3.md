# Task M100 #2185 Stage 3 완료보고서 — 포맷·WASM·Studio 인접 검증

## 목표

Stage 1의 공통 한글 줄 나눔 의미 정정이 HWP/HWPX 저장 계약, PUA와 기존 roundtrip을
훼손하지 않았는지 확인한다. 새 WASM을 빌드한 뒤 Studio의 단위·프로덕션 빌드와 실제
브라우저 편집까지 검증한다.

## 포맷·native 회귀 검증

- HWPX bit 7 파서
  - `cargo test --profile release-test --lib parser::hwpx::header::tests::test_parse_hwpx_para_shape_break_non_latin_word_bit -- --exact --nocapture`
  - 통과, 1 passed
  - `KEEP_WORD→bit7=1`, `BREAK_WORD→bit7=0` 저장 매핑 유지
- HWPX bit 7 직렬화
  - `cargo test --profile release-test --lib serializer::hwpx::header::tests::write_para_pr_emits_align_and_break_from_preserved_bits -- --exact --nocapture`
  - 통과, 1 passed
  - `bit7=0→BREAK_WORD` 역매핑 확인
- `cargo test --profile release-test --test issue_937 -- --nocapture`
  - 통과, 6 passed
  - PUA 원문·표시·SVG·filler 회귀 없음
- `cargo test --profile release-test --test hwp5_roundtrip_baseline -- --nocapture`
  - 통과, 3 passed, 약 14.92초
  - 기존 malformed FAT 경고 6회는 lenient CFB 폴백 후 통과
- `cargo test --profile release-test --test hwpx_roundtrip_baseline -- --nocapture`
  - 통과, 4 passed, 약 1.78초
  - 기존 invalid UTF-8 `Preview/PrvText.txt` 경고 3회는 lossy 폴백 후 통과

## WASM·Studio 빌드

- 표준 WASM 빌드
  - 현재 환경은 `docker compose` 하위 명령을 제공하지 않아 설치된 독립
    `docker-compose --env-file .env.docker run --rm wasm`을 사용했다.
  - 중지돼 있던 Colima를 시작해 빌드한 뒤 다시 중지해 원래 상태로 복구했다.
  - 결과: 통과, `wasm-pack 0.15.0`, 약 2분 08초
  - 산출물: `pkg/rhwp.js`, `pkg/rhwp_bg.wasm`, `pkg/rhwp.d.ts`
- Docker 데몬 부재를 확인하는 과정에서 로컬 `wasm-pack build --target web`도 시도했으나
  wasm-bindgen 설치 권한 단계에서 실패했다. 최종 권위 게이트는 위 Docker 빌드이며
  정상 통과했다.
- `cd rhwp-studio && npm test`
  - 통과, 185 passed, 실패·skip·todo 0
- 새 WASM 생성 후 `cd rhwp-studio && npm run build`
  - 통과, TypeScript와 Vite/PWA 빌드 성공, 135 modules
  - 기존 CanvasKit `fs`/`path` externalize와 500kB 초과 chunk 경고만 출력
- 모든 생성물은 기존 ignore 경로이며 추적 파일 변화 없음

## Studio 브라우저 검증

- 로컬 서버: `npm run dev -- --host 127.0.0.1 --port 7700`
- 자동 로드 URL로 `issue1949_giant_cell_nested_tables_perf.hwp`를 열었다.
- 초기 상태
  - 115쪽, 로드 약 1021.6ms
  - 첫 문단 1.1.1은 네 줄이며 마지막 줄이 `하여 적용한다.`로 끝남
- 네 번째 줄 끝에 `1`을 한 글자 입력했다.
- 편집 후
  - 마지막 줄만 `하여 적용한다.1`로 변경
  - 앞 세 줄의 시작 위치와 네 번째 줄 시작 위치가 육안상 불변
  - 다음 1.1.2 문단의 수직 위치 불변
  - 상태 바 `1 / 115 쪽` 유지
  - console warning/error 0건
  - `pagination` 로그 0건, `Violation` 로그 0건

115쪽 문서는 Studio의 자동 지연 pagination 한도 30쪽을 넘으므로 단일 셀 입력은 현재
페이지 로컬 갱신으로 남는다. 이번 검증에서 전체 pagination fallback이나 셀 overflow
fallback 신호는 발견되지 않았다.

## 판정

- renderer 의미 정정은 HWP/HWPX raw bit 저장 계약을 바꾸지 않았다.
- `0da18bbc`의 PUA 및 다른 roundtrip 변경을 훼손하지 않았다.
- native 통합 테스트와 실제 WASM/Studio가 동일하게 문단 경계를 보존한다.
- 입력 지연은 한글 줄 나눔 계산이 아니라 별도 전체 pagination·Canvas 갱신 비용이라는
  Stage 2 분리 결론과 모순되는 결과가 없다.

## 상태

Stage 3 검증 완료. Studio 개발 서버는 작업지시자의 추가 확인을 위해 7700 포트에서 계속
실행 중이다. 작업지시자 승인 전에는 Stage 4 광역 게이트를 진행하지 않는다.
