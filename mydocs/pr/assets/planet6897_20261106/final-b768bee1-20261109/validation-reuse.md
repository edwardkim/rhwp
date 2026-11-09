# 최종 제출과 기존 증거 재사용

- 제출 code head: `2e262d0d4b2885b130e2db917b86e894710425e4`, base `e00f9b7e26e581896fa17eaff3c8eb3f4889f9d4`.
- 마지막 제품 보정의 실측 HEAD: `e1fbd75cfcbefc0f633bcbd2ccef256614267ce8`. 이후 리베이스는 문서 6개만 바꾸었으며 제품·테스트 트리와 203개 적용 패치는 동일합니다. [전후 대조](final-source/rebase/result.json).
- 전체 회귀 13,105 PASS / 0 FAIL / 45 skip과 Native/fresh WASM 전쪽 94조건·2,995쪽은 `eb876d1416fb558c874f9efec65c5722a6769751`의 실제 실행입니다. 이번 HEAD에서 전체를 다시 실행했다는 뜻이 아닙니다. 사용자의 충돌 없는 리베이스 후 기존 자료 재사용 지시에 따라 실행 당시 producer·입력·PDF·글꼴 조건을 보존합니다.
- 앞선 `83c4c8c403ab07250c67d235ee13630a2aaf7ed0`의 HWP3 영향 3조건·10쪽과 Native 집중 72 PASS 및 Skia lib 검사를 재사용합니다. 마지막 보정은 명시적 자연 전진 세션 경계만 제한하고 나머지 source 메트릭 계약을 이전 전쪽 producer와 동일하게 유지합니다.
- 마지막 제품 보정에서는 Native/fresh WASM 2조건·8쪽, 최소 글꼴 Native 집중 6 PASS, fmt·Native/WASM/all-target Clippy·workspace build·manifest·unit-tier 검사를 실제 완료했습니다. [실행 영수증](final-source/scoped-validation-state.json).
- 새 WASM은 locked wrapper `--target web --out-dir pkg --no-opt`의 Mac 로컬 대체 빌드입니다. root pkg·Studio public·실제 Vite JS 응답과 instantiateStreaming WASM hash를 확인했습니다. [실제 초기화](final-source/canvaskit-actual-initialization.json). CanvasKit 13개 assertion과 실제 그룹 도형 화면도 확인했으며 의도적 Canvas2D fallback을 CanvasKit 성공에 합산하지 않았습니다.
- 통합 TSV는 페이지마다 `measurement_head`·`reuse_reason`을 포함합니다. Native/fresh WASM 각각 최저 **93.24%**, 미달·누락·쪽수 불일치 0입니다. 보호 TSV 1,771쪽/경로의 공식 두 자리 값 하락은 0입니다. 편람 HWP 59쪽은 마지막 source에서 두 경로 모두 100.00%로 복구했습니다.
- 고정 실측 base는 `b768bee1`입니다. 제출 base `e00f9b7e`의 HWP3 기능은 별도 영향 검증으로 연결하고, 문서 전진을 새 base 실측으로 표시하지 않습니다. latest-base 비교의 공통 2,991쪽과 base 자체의 누락 4쪽을 구분합니다.
- **엄격한 0.1%p 기준은 Native 뒤배경 표 4쪽 한 건이 미충족**입니다. 99.94357% → 99.80186%, 하락 0.14171%p이며 일반 2%p·90%와 특별 보호 TSV는 충족합니다. 이 결과를 무회귀로 기재하지 않고 사용자의 빠른 PR·잔여 새 이슈 지시에 따라 분리합니다. 글리프·간격 잔여 8문서군과 함께 새 이슈에서 처리합니다.

전체 전쪽·최소 글꼴 gate의 기존 조건과 최신 집중 검사의 조건을 바꾸거나 PDF·점수를 갱신해 통과시키지 않았습니다. 기존 per-case TSV와 168쌍의 판독 그림은 각각 표시된 과거 producer의 자료이며, 현재 수정된 clip5587·편람 집중 판독은 [최종 source 증적](final-source/current-direct-review-ledger.json)을 따릅니다.
