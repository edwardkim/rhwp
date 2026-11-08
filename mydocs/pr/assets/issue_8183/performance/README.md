# #8183 반복 성능 측정 원문

제품 source `cfc5be8fe`, base `d9fd85ed5bc3c4d9ce6b2981422e48bf9bb90867`다.
`native_samples.tsv`, `wasm_samples.tsv`는 각각 154개(각 조합 11회)의 최종 측정 원문이다.
Native는 `release-test`+`native-skia`, WASM은 release·codegen 16·wasm-opt 완료 패키지다.

Native의 wall time은 새 CLI process의 가져오기·저장·JSON 전체, RSS는 `/usr/bin/time`의 peak다.
WASM은 모듈 재사용의 동기 가져오기만 측정하며 linear capacity는 RSS가 아니다.
2회/3회 준비 실행 뒤 구성 순서를 교대했다. 측정 중 작업 소유 Cargo/회귀는 실행하지 않았다.
원문 중 최대값을 제거하지 않았고 서로 다른 codec의 입력을 동등 작업으로 간주하지 않는다.

환경·방법·해석·중앙값과 범위는 [3단계 기록](../../../../working/task_m100_8183_stage3.md#성능-관측)에 있다.
원본 입력은 [pdf/issue8183](../../../../../pdf/issue8183/README.md)에 있다.
