# PR #9093의 제출 중 devel 전진 반영

원격 #9093은 2026-11-15 13:04:01 UTC에 병합됐고 수용 PR #9096은 13:04:21 UTC에 생성됐다.
앞선 push 전 조회 후 발생한 변경이므로 로컬에서 `1805d8f08c`를 `b037f305c5`에 실제 병합하고 검증했다.

[실제 명령·초기화 WASM 응답·12개 결과 대조](validation.json).
최신 devel과 후보의 Studio 소스는 바이트가 같고, 동일 Chromium 1660786(실제 UA Chrome 152)에서
웹 등록·로컬 공급, 별칭 미공급·TTF/intrinsic 공급 12개 결과가 모두 같다. 양쪽 모두 실제 원시
Canvas의 폭·전체 픽셀과 일치하며 공급된 별칭의 다른 프로그램을 구분한다.
후보 Studio 2,798 PASS·2 기존 SKIP, TypeScript·production build·E2E manifest·기존 브라우저 26건도 통과했다.
초기화 응답 WASM의 SHA를 직접 검증했다. Rust 입력·최적화 WASM·독립 Visual Sweep exporter는 바뀌지 않아
앞선 전체 Rust·Native·전쪽 검증은 해당 경로의 증거로 유지한다. 새 Studio 경로를 과거 캡처로 검증했다고 주장하지 않는다.
