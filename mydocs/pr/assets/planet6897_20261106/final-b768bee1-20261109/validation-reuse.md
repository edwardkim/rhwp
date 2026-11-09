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

## 제출 후 확인

## 통합 PR의 exact code head CI

- [통합 PR #8857](https://github.com/edwardkim/rhwp/pull/8857), code head `29c74b69851bc6578f10a2c78e23592a05b69a5d`.
- [CI 실행](https://github.com/edwardkim/rhwp/actions/runs/42621490405)과 같은 code head의 CodeQL·Render Diff·Adapter·Proptest·정책 검사를 실제 확인했습니다. 기록 확인 시각: `2026-11-09T22:58:24.689803+09:00`.
- 기본 기능 worker A–E의 실제 nextest는 **12,934 PASS / 0 FAIL / 45 skip**입니다. 기존 로컬 13,105건과 실행 범위·HEAD가 다르므로 합산하지 않습니다.
- source·test·workflow를 추가로 변경하지 않고 이 검토·오늘할일만 단일 부모 trailing commit으로 보완합니다. trailing head의 실제 CI/fast-pass와 mergeability는 push 뒤 별도로 확인하며, 문서만 바뀌었다는 이유로 통과를 미리 가정하지 않습니다.
- [잔여 이슈 #8856](https://github.com/edwardkim/rhwp/issues/8856)은 9문서군의 글리프·간격·도형 및 clip5587 Native 4쪽의 엄격 0.1%p 미충족을 계속 추적합니다. 부분 해결 #8635·#8636과 부모 #7445는 일괄 종료하지 않습니다.


### 실제 CI 실행 범위

기본 기능 A–E는 attempt 1의 실제 로그에서 **12,934 PASS / 0 FAIL / 45 skip**을 확인했습니다. Native Skia는 attempt 2에서 **239 PASS / 0 FAIL**입니다. 첫 Native 실행의 30분 제한 취소와 구분하며, 다른 프로필·로컬 전체 건수와 합산하지 않습니다. [원 worker·attempt·카운트·최종 checks](final-source/code-ci-29c74b69.json).
