# #8174 v2 S05 — 포장·소비 증적

제품 source `9c9c272fe1670baf305800dfc906349fa7933d4a`의 로컬 검증이다.
[수행 기록](../../../../working/task_m100_8174_v2_stage5.md)과 함께 읽는다.

- [검증 집계와 실제 명령](validation.json), [feature 소비 조합](feature-matrix.json).
- [14개 archive의 버전·크기·SHA-256·VCS 출처](metadata/final-packages.json),
  [다운로드 source와 checksum](metadata/final-selected-sources.json),
  [파일·권한 검사](metadata/archive-audit.json).
- [제품 source 동일성](metadata/product-source-cohorts.json),
  [Native producer](metadata/native-producer.json), [WASM producer](metadata/wasm-producer.json),
  [API 비활성화 앱](wasm-off-producer.json).
- [브라우저 원문 결과](browser.json), [CLI 계약](metadata/cli-contracts.json),
  [실패·복구](metadata/negative-cli-results.json), [Rust 실패 코드](metadata/negative-rust-results.json).
- [전쪽 시각 집계·쪽수·S04 대조](metadata/visual-summary.json),
  [직접 판독](direct-review.json), [기존 yank 경고 조사](metadata/existing-yank-review.json).
- `scores/`: Native/fresh WASM TSV 90개를 바이트 그대로 보존했다. 총 98페이지다.
- `images/`: 실제 패키지의 대표 compare·overlay·review PNG 36개다.
- `probes/`: 소비 앱 manifest·Rust 원문·기존 검사와 당시 진단 원문이다. 자동 배포·CI 도구가
  아니며 재실행할 때 source·경로·registry 원점·출력 초기 상태를 먼저 확인한다.

중간 로그·archive·변환 HWPX와 전체 실행 상태는 ignored
`output/pr-review/issue8174/v2-s05-20261025/`에 둔다. 모든 패키지 source를 최종 Git과
대조했으며, archive VCS는 13개 자식의 `202ef5416`과 README 수정 뒤 본체의 `9c9c272fe`를
구분한다. 동일 버전 archive를 이미 내려받은 registry에서 덮어쓰지 않았다.

Native는 `release-test`, WASM은 release·wasm-opt를 완료한 독립 앱이다. 앱이 추가한 세 export를
기존 Studio bundle의 크기 변화로 해석하지 않는다. S04의 전체 Native·Studio 검증과 이번 실제
포장 후 소비를 구분하며, `not_evaluated` TSV만으로 직접 판독을 대신하지 않았다.

로컬 registry 소비는 통과했다. Cargo 자체 package verify, 실제 crates.io 게시, 계정·토큰·
소유권·최종 제품 버전과 MD05의 남은 보안 감사를 완료했다고 표시하지 않는다.
