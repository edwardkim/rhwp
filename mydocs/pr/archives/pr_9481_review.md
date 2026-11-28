# PR #9481 검토

검증 기준 `devel`은 `ea9f0bb7a044a9cdb537d3c3a13502bece2d0e2c`, 측정 코드는 `be48607ad9444d8232df4f699c73832d9a34885d`, 제출 커밋은 `9d88bcfe3aedf57197b36d971ea16b18ef50dace`입니다. #7445의 하위 이슈 #7789·#7790·#7791·#7792·#7806·#7807·#7808·#7811·#7812를 검토했습니다. 별도로 #7803의 해결 커밋이 `devel`에 반영된 것을 확인했습니다. [PR #9481](https://github.com/edwardkim/rhwp/pull/9481) · [#7803 종료 근거](https://github.com/edwardkim/rhwp/issues/7803#issuecomment-7217589296)

## 조판과 출력

- 저장된 줄과 셀의 소속을 기준으로 표 조각의 높이와 쪽 경계를 정합니다. `prepare_table_fragment_budget`에서 계산한 예산을 행 스캔과 `emit_table_fragment`에 전달하는 경로를 확인했습니다. #7792에서 첫 조각의 열린 경계와 이어받은 행의 내용 순서를 확인하고, 2쪽 하단선과 3쪽 상단선을 한컴 Print에서 직접 대조했습니다. #7811과 #7812에서는 마지막 조각의 행 점유와 뒤 본문 순서를 함께 확인했습니다. [표 이어받기 경로](../../../src/renderer/typeset/table/continuation/fragment.rs) · [#7792 3쪽 비교](../assets/issues7789_7812/table-text-anchor-wrap/native/review_003.png)
- HWP3 원본 글꼴 종류와 구두점의 글자 슬롯을 유지하는 경로를 확인했습니다. SO-SUEOP의 HWP3와 HWPX 각각 26쪽에서 명조 한점 리더의 전진폭이 인접 한글 한 글자와 같은 것을 한컴 Print와 대조했습니다. [HWP3 글꼴 처리](../../../src/parser/hwp3/font_program.rs) · [구두점 슬롯](../../../src/renderer/composer/punctuation_slots.rs) · [SO-SUEOP 26쪽 비교](../assets/issues7789_7812/so-sueop-hwp3/native/review_026.png)
- AIFT는 독립 한컴 단면 Print의 74쪽을 기준으로 표 조각과 뒤 제목의 소속을 확인했습니다. 33쪽의 표가 본문 오른쪽 기준을 약 5px 넘는 원본 배치와 쪽수, 뒤 제목의 순서를 보호합니다. [AIFT 전쪽 TSV](../assets/issues7789_7812/aift/native/silhouette.tsv)

## 로컬 검증

- Native Skia와 fresh WASM에서 문서 11개, 각각 336쪽을 96 DPI의 2px raw 실루엣으로 독립 한컴 Print와 대조했습니다. 전쪽 PNG는 리베이스 전 검증본과 일치했고, 최저값은 Native 98.11%, fresh WASM 98.04%였습니다. [전쪽 증적 영수증](../assets/issues7789_7812/receipt.json)
- 편람 HWPX 383쪽을 포함한 Native 보호 문서 413쪽과 #8506 표지의 Native·fresh WASM 4쪽이 새 `devel`과 PNG SHA-256까지 일치했습니다. 공통 보호 문서 4종의 두 출력과, #9480에서 반영된 표 배치·캐럿·머리말·캡션 회귀 5건을 검증했습니다. 반복 암호 저장한 HWP5 파일의 SHA-256도 `devel`과 후보에서 같았습니다. [편람 Native 전쪽 TSV](../assets/issues7789_7812/native-protected/handbook_383.tsv) · [PR #9480](https://github.com/edwardkim/rhwp/pull/9480)
- Rust 회귀 14,291건, Native Skia 라이브러리 4,033건과 Native Skia 통합 모듈 21개의 194건이 통과했습니다. 설정으로 Rust 회귀 45건과 라이브러리 12건을 건너뛰었습니다. 서식, Native·WASM Clippy, 워크스페이스 빌드와 검사, 테스트 목록 검증을 마쳤습니다.

## GitHub Actions와 병합 후 처리

제출 커밋 `9d88bcfe3aedf57197b36d971ea16b18ef50dace`의 Full CI와 최종 CI 정책 판정이 통과했습니다. [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/46100637326/job/140850609096) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/46100637265) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/46100637240) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/46100637277) · [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/46100637339) · [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/46103521554)

이 기록을 포함한 커밋의 검사와 병합 가능 상태를 확인합니다. 병합 뒤 대상 이슈 9건에 전쪽 측정표와 대표 비교 PNG를 본문에 넣고 종료한 뒤 #7445의 해당 행에 취소선을 표시합니다.
