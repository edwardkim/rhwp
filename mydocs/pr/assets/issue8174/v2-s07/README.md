# #8174 S07 실행 증거

검증 source는 `0ca4dac5000e7fb888879fcac168376d0a9d4a2f`, 고정 devel은
`9418ed0c7cf289c5c4ac2b3bb8cdc18a1b637a9a`, 보정 전은
`4e2149dc65bdbc15413bf09d710c85882260e2ae`다. 자세한 명령·판정과 남은 조건은
[S07 보고](../../../../working/task_m100_8174_v2_stage7.md)를 따른다.

자원 계약 통과와 시각 완료를 구분한다. 전쪽 수치가 90% 이상이어도 CFF 문자 폭과 원본
12쪽 PDF의 11쪽 줄바꿈 차이는 남아 있어 **시각 완료·PR 제출을 보류**한다.
성능도 무영향으로 판정하지 않는다.

- [구현 경계·Git blob](source-boundaries.json), [실제 검증 명령](gates.json),
  [최종 기능 대조 명령](feature-runtime-gates.json).
- [Native producer](candidate-native-producer.json), [최적화 WASM producer](candidate-wasm-producer.json).
- [보정 전·고정 devel producer](baseline-producers.json), [자원·시각 분리 판정](verdict.json).
- [수정 전 6 FAIL](before-font-contracts.json), [Native 11 PASS](candidate-font-contracts.json),
  [WASM 11 PASS](wasm-font-contracts.json), [font 단위 검사의 문맥·해시](font-unit-harness.json).
- [실제 CLI 거절](cli-font-boundaries.json), [WASM 거절](browser-tiny-fonts.json),
  [Worker 복구](browser-worker-recovery.json), [Studio 활성 상태 보존](studio-fonts.json).
- [전체 시각 점수](visual-summary.json), [직접 판독과 보류 이유](visual-inspection.json),
  [RGBA 비교](pixel-preservation.json), [HWPX member 비교](content-preservation.json).
- [성능 요약](performance-summary.json), [Native 원문](performance/native-import.json),
  [제품 WASM 원문](performance/wasm-import.json), [Native memo](performance/native-memo.json),
  [WASM memo](memo-performance-wasm.json).
- [독립 입력·글꼴 해시](font-visual-inputs.json), [font 공급 진단](font-supply.json),
  [진단 재실행](cff-font-supply-runs.json), [원본 PDF](../../../../../pdf/issue8174/fonts/cff-cid.pdf).

`tsv/{base,before,candidate}/{native,wasm}/<문서>/silhouette.tsv`는 실행 결과를 바이트 그대로
복사한 전쪽 TSV다. base/before는 폰트 2문서·13쪽, candidate는 43문서·54쪽씩이다.
폰트 원본 PDF 직접 가져오기는 compat 2022, 기존 기능 HWPX 대조는 S05와 같은 compat 2024다.
`images/`는 대표 review와 standalone overlay다. `manifest.json`은 파일별 해시를 고정한다.
`probes/`는 생성·진단·측정 source이며 회귀 suite나 제품 API의 추가 대상이 아니다.
실행 로그·중간 tree/JSON·바이너리·의존성은 ignored output에 보존했다.
