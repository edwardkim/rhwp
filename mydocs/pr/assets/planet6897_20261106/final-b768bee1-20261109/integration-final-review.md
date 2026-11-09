# planet6897 원 PR 27건 통합 검토

## 최종 판정

**메인터너 보정 후 수용 가능**

원 기여와 메인터너 보정을 포함한 아래 통합 code head의 로컬 검증·시각 대조를 완료했습니다. 원 contributor head를 그대로 병합하는 승인은 아닙니다. 통합 PR의 최신 head CI·충돌 확인은 게시 후 별도 단계로 남습니다.

- 검증 code head: `eb876d1416fb558c874f9efec65c5722a6769751`
- 검증 base: `b768bee1c4c0023087687f79a0a95f12108743e9`
- 브랜치: `review/integration-20261106-planet6897-open`
- [원 head 실시간 조회](original-pr-heads.json) · [56개 원 커밋→적용 커밋](../contributor-commit-map-final.tsv)

[최종 source·리베이스·증거 재사용·잔여 판정](validation-reuse.md)

## 통합 PR의 exact code head CI

- [통합 PR #8857](https://github.com/edwardkim/rhwp/pull/8857), code head `29c74b69851bc6578f10a2c78e23592a05b69a5d`.
- [CI 실행](https://github.com/edwardkim/rhwp/actions/runs/42621490405)과 같은 code head의 CodeQL·Render Diff·Adapter·Proptest·정책 검사를 실제 확인했습니다. 기록 확인 시각: `2026-11-09T22:58:24.689803+09:00`.
- 기본 기능 worker A–E의 실제 nextest는 **12,934 PASS / 0 FAIL / 45 skip**입니다. 기존 로컬 13,105건과 실행 범위·HEAD가 다르므로 합산하지 않습니다.
- source·test·workflow를 추가로 변경하지 않고 이 검토·오늘할일만 단일 부모 trailing commit으로 보완합니다. trailing head의 실제 CI/fast-pass와 mergeability는 push 뒤 별도로 확인하며, 문서만 바뀌었다는 이유로 통과를 미리 가정하지 않습니다.
- [잔여 이슈 #8856](https://github.com/edwardkim/rhwp/issues/8856)은 9문서군의 글리프·간격·도형 및 clip5587 Native 4쪽의 엄격 0.1%p 미충족을 계속 추적합니다. 부분 해결 #8635·#8636과 부모 #7445는 일괄 종료하지 않습니다.

## 기존 전쪽 검증 — 생산자 SHA를 유지해 재사용

- 최소 글꼴 전체 nextest: 13,105 PASS / 0 FAIL / 45 skip. Native 전용 11모듈 73 PASS / 0 FAIL.
- fmt·Native/WASM/all-target Clippy, workspace/release build, manifest/unit-tier, release/Skia lib·PNG·PDF·doctest를 통과했습니다.
- Studio 2,784 PASS / 0 FAIL / 2 skip 및 build 통과. 실제 Canvas exporter 17건, projection 16건, Python 76건, Web Canvas 12건을 통과했습니다.
- 실제 CanvasKit 준비·스타일·실패 진단 계약 13개 assertion을 확인했습니다. 의도적 준비 실패의 Canvas2D fallback을 CanvasKit 렌더 성공으로 주장하지 않습니다.
- 공개 글꼴 이름 API 15호출 및 저장 대상 7언어 fontRef·오류 불변성을 실제 패키지에서 확인했습니다.
- Native/fresh WASM 각각 94조건·2,995쪽, 최저 92.24% / 92.29%. 90% 미만·누락·쪽수 불일치 0, 보호 1,771쪽/경로의 공식 표시값 하락 0.
- 완료 쪽의98% 미만 후퇴·일반2%p 초과 하락은 없습니다. 엄격0.1%p 비교에는 Native 뒤배경 표4쪽0.14171%p 잔여가 있으며 사용자 요청에 따라 새 이슈로 분리합니다. 특별 보호 TSV는 공식 표시값 하락0을 유지합니다.
- 공개 웹폰트 최소 환경의 Native HWPX 편람 gate 383쪽 통과: 공식 두 자리 값 130쪽 99.96%, 나머지 382쪽 100.00%. RGB 픽셀 동일성은 아닙니다.
- Mac WASM은 locked wrapper `--target web --out-dir pkg --no-opt` 로컬 대체 빌드입니다. root pkg·Studio public·실제 초기화 응답의 hash를 확인했습니다. Docker 최적화 결과로 주장하지 않습니다.

[전체 회귀](full-nextest-result.json) · [Native 전용](native-feature-summary.json) · [Rust 게이트](rust-gates-state.json) · [실제 소비](actual-consumer-test-summary.json) · [최소 글꼴 판정](minimum-font-qualification.json)

## 전쪽·직접 판독 증적

[Native TSV](native-final.tsv) · [fresh WASM TSV](wasm-final.tsv) · [최신 base Native 대조](comparisons/latest-base-native-comparison.tsv) · [최신 base WASM 대조](comparisons/latest-base-wasm-comparison.tsv)

대표 168쌍을 현재 산출물로 검증했습니다. 164쌍은 같은 backend에서 직접 판독한 이전 producer의 raw·PDF raw·review·overlay와 모두 바이트가 같음을 확인했고, 변경된 보고서 95·96쪽의 Native/WASM 4쌍은 새로 직접 판독했습니다. 다른 backend의 그림으로 판독을 대체하지 않았습니다.

[직접 판독·동일성 영수증](claim-direct-reviews.json) · [입력/독립 PDF 감사](committed-input-audit.json) · [실제 CanvasKit](canvaskit-actual-initialization.json)

## base 자체의 기존 실패와 잔여 범위

base의 보류/실행 문제는 비교 JSON의 reference_execution_problems에 따로 기록했습니다. report-838은 base 110쪽/독립 PDF 109쪽이며 후보는 109쪽입니다. 공통 109쪽만 실제 PNG로 재측정하고 대응 없는 110쪽을 보존했으며 base를 통과로 바꾸지 않았습니다.

[base 쪽수 불일치·복구 영수증](comparisons/base-extra-page-recovery-state.json) · [잔여 8문서군 측정 계획](residual/plan.json)

사용자 지시에 따라 확인한 잔여 글리프·간격·도형·선택지 차이는 새 이슈로 분리합니다. 이 기록의 코드/시각 검증 통과가 모든 잔여 결함의 해결을 의미하지 않습니다. 새 이슈 게시 뒤 통합 PR 본문과 후속 기록에 실제 링크를 연결합니다.

## 개별 검토

| 원 PR | review | 구현·보정 |
| --- | --- | --- |
| #8550 | [review](../../../archives/pr_8550_review.md) | [구현·보정](../../../archives/pr_8550_review_impl.md) |
| #8552 | [review](../../../archives/pr_8552_review.md) | [구현·보정](../../../archives/pr_8552_review_impl.md) |
| #8559 | [review](../../../archives/pr_8559_review.md) | [구현·보정](../../../archives/pr_8559_review_impl.md) |
| #8563 | [review](../../../archives/pr_8563_review.md) | [구현·보정](../../../archives/pr_8563_review_impl.md) |
| #8581 | [review](../../../archives/pr_8581_review.md) | [구현·보정](../../../archives/pr_8581_review_impl.md) |
| #8589 | [review](../../../archives/pr_8589_review.md) | [구현·보정](../../../archives/pr_8589_review_impl.md) |
| #8600 | [review](../../../archives/pr_8600_review.md) | [구현·보정](../../../archives/pr_8600_review_impl.md) |
| #8604 | [review](../../../archives/pr_8604_review.md) | [구현·보정](../../../archives/pr_8604_review_impl.md) |
| #8605 | [review](../../../archives/pr_8605_review.md) | [구현·보정](../../../archives/pr_8605_review_impl.md) |
| #8607 | [review](../../../archives/pr_8607_review.md) | [구현·보정](../../../archives/pr_8607_review_impl.md) |
| #8611 | [review](../../../archives/pr_8611_review.md) | [구현·보정](../../../archives/pr_8611_review_impl.md) |
| #8615 | [review](../../../archives/pr_8615_review.md) | [구현·보정](../../../archives/pr_8615_review_impl.md) |
| #8617 | [review](../../../archives/pr_8617_review.md) | [구현·보정](../../../archives/pr_8617_review_impl.md) |
| #8621 | [review](../../../archives/pr_8621_review.md) | [구현·보정](../../../archives/pr_8621_review_impl.md) |
| #8623 | [review](../../../archives/pr_8623_review.md) | [구현·보정](../../../archives/pr_8623_review_impl.md) |
| #8625 | [review](../../../archives/pr_8625_review.md) | [구현·보정](../../../archives/pr_8625_review_impl.md) |
| #8638 | [review](../../../archives/pr_8638_review.md) | [구현·보정](../../../archives/pr_8638_review_impl.md) |
| #8642 | [review](../../../archives/pr_8642_review.md) | [구현·보정](../../../archives/pr_8642_review_impl.md) |
| #8644 | [review](../../../archives/pr_8644_review.md) | [구현·보정](../../../archives/pr_8644_review_impl.md) |
| #8645 | [review](../../../archives/pr_8645_review.md) | [구현·보정](../../../archives/pr_8645_review_impl.md) |
| #8649 | [review](../../../archives/pr_8649_review.md) | [구현·보정](../../../archives/pr_8649_review_impl.md) |
| #8650 | [review](../../../archives/pr_8650_review.md) | [구현·보정](../../../archives/pr_8650_review_impl.md) |
| #8654 | [review](../../../archives/pr_8654_review.md) | [구현·보정](../../../archives/pr_8654_review_impl.md) |
| #8655 | [review](../../../archives/pr_8655_review.md) | [구현·보정](../../../archives/pr_8655_review_impl.md) |
| #8662 | [review](../../../archives/pr_8662_review.md) | [구현·보정](../../../archives/pr_8662_review_impl.md) |
| #8663 | [review](../../../archives/pr_8663_review.md) | [구현·보정](../../../archives/pr_8663_review_impl.md) |
| #8681 | [review](../../../archives/pr_8681_review.md) | [구현·보정](../../../archives/pr_8681_review_impl.md) |

## 원격 후속 순서

개별 검토·증적 커밋 → 고정 SHA의 실제 공개 바이트 확인 → 잔여 이슈 게시 → 통합 Open PR → 정확한 code head CI → 기존 27개 review 보완·오늘할일 기록 → 최신 trailing head 검증 → 병합·원 PR/실제 해결 이슈 후속 처리 순서로 진행합니다.

원 PR/이슈를 번호가 언급됐다는 이유만으로 모두 닫지 않습니다. 원 기여 성과·메인터너 보정·미해결 범위를 구분해 해당 주장 페이지의 실제 raw·review·overlay로 설명합니다.
