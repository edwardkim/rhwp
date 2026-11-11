# planet6897 원 PR 39개 실제 체리픽 검토

> 최신 검토 범위와 결과(2026-11-11): [원 PR 주장·보호 문서로 제한한 범위](maintainer-repair/final-scope.md)를 따른다.
> 제품 코드 `ab9116f48f`의 [Rust 필수 게이트 12개](maintainer-repair/final-rust-gates.json)는 모두 통과했으며,
> [전체 회귀 13,307 PASS / 0 FAIL](maintainer-repair/final-full-nextest.json),
> [Native 보호 문서 18조건·1,771쪽 하락 0쪽](maintainer-repair/final-native-protected.json)을 확인했다.
> WASM 보호 문서와 원 PR 주장 페이지의 최종 검사는 진행 중이다. 아래 본문은 `e6261fd4ec` 초기 검토 기록이며 최신 판정과 구분한다.


**검토 결과: 통합 후보 머지 보류.** 실제 적용·전체 회귀·전쪽 비교와 개별 PR 증적을 남겼으며, 아래 실패를 통과로 보고하지 않습니다. GitHub 댓글·승인·close·새 통합 PR 제출은 하지 않았습니다.

- 브랜치: `review/integration-20261111-planet6897-open39`.
- 고정 base: `e118de8abadd2dffb2798036b9ffd381b96ed7d5`.
- 검토 코드 head: `e6261fd4ecd4ed2c5b0039d7526915c8c7837408`. 이후 검토 문서 커밋과 구분합니다.
- 원 PR 39개, 적층 중복 제외 102개 커밋을 `cherry-pick -x`로 실제 적용했습니다. 충돌/빈 커밋은 0개입니다. 원 contributor의 head를 변경하지 않았습니다. 새 #8932는 동결 범위 밖입니다.

## 완료한 검증

- Mac 최소 글꼴 전체 nextest: **13,302 PASS / 0 FAIL / 45 skip**, 395.627초. 신규 42개 검사 모듈의 77개 함수도 실제 실행 결과와 매핑했습니다.
- fmt·Native/WASM/all-target Clippy·workspace build·Skia lib 4,034 PASS / 12 ignore·PNG 2 PASS·PDF 35 PASS·manifest/unit tiers 완료.
- 후보 Native/fresh WASM 각각 **48조건·1,665쪽** 산출, 독립 PDF 쪽수 일치. 지원 대조군과 기존 미달은 별도 표기했습니다.
- 원 PR 주장 페이지 88쪽 × 두 backend의 raw/PDF/review/overlay를 보존했습니다. 39개 대표 페이지는 두 backend 모두 review와 overlay를 직접 판독했습니다.
- 반복 저장 4입력 × 8단계: IR diff 0, 쪽수 보존, 삽입 문자열 1회 확인.
- fresh WASM은 Mac locked wrapper `--no-opt` 로컬 대체 빌드이며 Docker 최적화 결과가 아닙니다. root pkg·Studio public·실제 초기화 WASM hash를 확인했습니다.

## 수용 보류 근거

| 구분 | 확인한 결과 |
| --- | --- |
| #8803 관련 대조군 | HPV 159쪽 Native −9.77412%p, WASM −9.79218%p. 그림/원고의 쪽 소속이 독립 Print와 다릅니다. |
| #8746 | 36483048 5쪽 Native 79.65043% / WASM 78.51980%, 36453591 WASM 5쪽 88.81582%. |
| #8788·#8903·#8928 | 각각 WASM 3쪽 77.73117%, 6쪽 88.56802%, 3쪽 89.31429%. |
| #8736·#8792 | 주장 페이지 개선과 별개로 해당 원본의 다른 페이지에 90% 미달이 남습니다. |
| #8930 원 CI | Linux Canvas visual diff 실패. Mac legacy/layer 대조군 PASS로 대체하지 않았습니다. |
| 정본 보호 문서 | Native 532쪽·WASM 967쪽이 공식 표시값보다 낮음. 동일 현재 조건 base Native 11종 대조 완료. Native 1,761쪽 중 1,749쪽 PNG 동일; 추가 표시값 하락은 issue7905 2쪽 −0.05%p 한 곳. 조건 재현 실패와 제품 추가 하락을 구분하며 기준을 면제하지 않습니다. |
| 실제 브라우저 | 15개 캡처, HFT 표본 CK 두 경로의 glyphMapping 계약 실패 및 비교 가능 8쌍의 기본 시각 예산 초과. |

정본 보호 문서 18종·1,771쪽의 Native/fresh WASM 전쪽 TSV를 완료했습니다. 공식 두 자리 기준보다 낮은 쪽은 Native 532쪽·fresh WASM 967쪽입니다. 이 수치는 **모두 새 코드 회귀라는 뜻이 아닙니다**. 같은 현재 조건의 고정 base Native 11종·1,761쪽 대조도 완료했습니다. 1,749쪽은 base/후보 PNG hash가 같고, 532쪽의 공식 기준 미달은 base에도 있습니다. 후보가 base보다 추가로 낮춘 공식 표시값은 `issue7905` 2쪽 93.77→93.72%(raw −0.04790%p) 한 곳입니다. 기존 보호 점수를 재현하지 못한 조건과 통합 코드가 추가로 낮춘 쪽을 [보호 비교 정본](protected-final.json)에 구분했습니다. WASM 보호 검사의 동일 조건 base 비교는 수행하지 않았으므로 해당 하락의 제품 원인을 확정하지 않습니다.

필수 편람 HWPX Native 383쪽은 258쪽 100.00→99.99%, 272쪽 100.00→99.94%, 368쪽 100.00→99.97%로 `re_review_required`입니다. 최소 글꼴 전체 회귀와 기존 원본 글꼴 보호 조건의 재현 성공을 혼동하지 않으며 보호 문턱을 면제하지 않습니다.

실제 브라우저는 같은 WASM 응답 hash를 확인한 뒤 5개 입력 × Canvas2D/CanvasKit compat/default = 15개를 캡처했습니다. [소비 경로 결과](validation/actual-browser.json)에서 HFT 표본의 CanvasKit 두 경로가 `textRun:glyphMapping` 계약을 실패했습니다. 비교 가능한 Canvas2D↔CanvasKit 8쌍도 기본 시각 차이 예산을 초과했습니다. 캡처 완료를 소비 경로 PASS로 쓰지 않습니다. 같은 조건의 고정 base에서도 동일한 glyphMapping 계약 실패 두 건과 시각 예산 초과 8쌍이 재현됐습니다. 따라서 이 실패를 이번 원 PR의 새 결함으로 단정하지 않습니다. [base/후보 브라우저 대조](validation/actual-browser-base-comparison.json)를 보존하며 이 조건의 소비 경로 수용도 보류합니다.

[원 PR CI 대조](validation/source-ci.json)는 고정한 39개 head에 대해 조회했습니다. #8930의 Linux `Canvas visual diff` 실패가 남아 있습니다. 다른 38개는 조회한 check run에 실패·실행 중 항목이 없었습니다. 원 PR CI 통과가 누적 통합 후보 검증을 대신하지 않습니다.

## 개별 리뷰

| 원 PR | 검토 문서 | 주요 판정 |
| --- | --- | --- |
| #8689 | [리뷰](../../archives/pr_8689_review.md) | 통합 후보의 공통 보호 TSV 비교를 충족하지 못해 이 head의 수용을 보류합니다. |
| #8708 | [리뷰](../../archives/pr_8708_review.md) | 통합 후보의 공통 보호 TSV 비교를 충족하지 못해 이 head의 수용을 보류합니다. |
| #8710 | [리뷰](../../archives/pr_8710_review.md) | 통합 후보의 공통 보호 TSV 비교를 충족하지 못해 이 head의 수용을 보류합니다. |
| #8717 | [리뷰](../../archives/pr_8717_review.md) | 통합 후보의 공통 보호 TSV 비교를 충족하지 못해 이 head의 수용을 보류합니다. |
| #8719 | [리뷰](../../archives/pr_8719_review.md) | 통합 후보의 공통 보호 TSV 비교를 충족하지 못해 이 head의 수용을 보류합니다. |
| #8729 | [리뷰](../../archives/pr_8729_review.md) | 보호 TSV 미충족. HFT 표본 CanvasKit glyphMapping 실패는 동일 조건 base에도 재현. |
| #8736 | [리뷰](../../archives/pr_8736_review.md) | 주장 16·17쪽의 개선과 별개로 전 138쪽에서 Native 8쪽·fresh WASM 10쪽이 90% 미만입니다. |
| #8740 | [리뷰](../../archives/pr_8740_review.md) | 통합 후보의 공통 보호 TSV 비교를 충족하지 못해 이 head의 수용을 보류합니다. |
| #8744 | [리뷰](../../archives/pr_8744_review.md) | 통합 후보의 공통 보호 TSV 비교를 충족하지 못해 이 head의 수용을 보류합니다. |
| #8746 | [리뷰](../../archives/pr_8746_review.md) | 결재문서 36483048의 5쪽이 Native 79.65043%·fresh WASM 78.51980%이며, 36453591의 fresh WASM 5쪽도 88.81582%입니다. |
| #8751 | [리뷰](../../archives/pr_8751_review.md) | 통합 후보의 공통 보호 TSV 비교를 충족하지 못해 이 head의 수용을 보류합니다. |
| #8758 | [리뷰](../../archives/pr_8758_review.md) | 통합 후보의 공통 보호 TSV 비교를 충족하지 못해 이 head의 수용을 보류합니다. |
| #8772 | [리뷰](../../archives/pr_8772_review.md) | 통합 후보의 공통 보호 TSV 비교를 충족하지 못해 이 head의 수용을 보류합니다. |
| #8775 | [리뷰](../../archives/pr_8775_review.md) | 통합 후보의 공통 보호 TSV 비교를 충족하지 못해 이 head의 수용을 보류합니다. |
| #8785 | [리뷰](../../archives/pr_8785_review.md) | 통합 후보의 공통 보호 TSV 비교를 충족하지 못해 이 head의 수용을 보류합니다. |
| #8788 | [리뷰](../../archives/pr_8788_review.md) | 저장 되감김 문서의 fresh WASM 3쪽이 77.73117%로 미달입니다. |
| #8792 | [리뷰](../../archives/pr_8792_review.md) | 주장 페이지의 그림 배치는 개선됐지만 전 313쪽에서 Native·fresh WASM 각각 10쪽이 90% 미만입니다. |
| #8794 | [리뷰](../../archives/pr_8794_review.md) | 통합 후보의 공통 보호 TSV 비교를 충족하지 못해 이 head의 수용을 보류합니다. |
| #8803 | [리뷰](../../archives/pr_8803_review.md) | HPV 보고서 159쪽이 같은 base 대비 Native 9.77412%p · fresh WASM 9.79218%p 하락했습니다. |
| #8806 | [리뷰](../../archives/pr_8806_review.md) | 통합 후보의 공통 보호 TSV 비교를 충족하지 못해 이 head의 수용을 보류합니다. |
| #8830 | [리뷰](../../archives/pr_8830_review.md) | 통합 후보의 공통 보호 TSV 비교를 충족하지 못해 이 head의 수용을 보류합니다. |
| #8833 | [리뷰](../../archives/pr_8833_review.md) | 통합 후보의 공통 보호 TSV 비교를 충족하지 못해 이 head의 수용을 보류합니다. |
| #8835 | [리뷰](../../archives/pr_8835_review.md) | 통합 후보의 공통 보호 TSV 비교를 충족하지 못해 이 head의 수용을 보류합니다. |
| #8838 | [리뷰](../../archives/pr_8838_review.md) | 통합 후보의 공통 보호 TSV 비교를 충족하지 못해 이 head의 수용을 보류합니다. |
| #8846 | [리뷰](../../archives/pr_8846_review.md) | 통합 후보의 공통 보호 TSV 비교를 충족하지 못해 이 head의 수용을 보류합니다. |
| #8848 | [리뷰](../../archives/pr_8848_review.md) | 통합 후보의 공통 보호 TSV 비교를 충족하지 못해 이 head의 수용을 보류합니다. |
| #8849 | [리뷰](../../archives/pr_8849_review.md) | 통합 후보의 공통 보호 TSV 비교를 충족하지 못해 이 head의 수용을 보류합니다. |
| #8861 | [리뷰](../../archives/pr_8861_review.md) | 통합 후보의 공통 보호 TSV 비교를 충족하지 못해 이 head의 수용을 보류합니다. |
| #8862 | [리뷰](../../archives/pr_8862_review.md) | 통합 후보의 공통 보호 TSV 비교를 충족하지 못해 이 head의 수용을 보류합니다. |
| #8876 | [리뷰](../../archives/pr_8876_review.md) | 통합 후보의 공통 보호 TSV 비교를 충족하지 못해 이 head의 수용을 보류합니다. |
| #8879 | [리뷰](../../archives/pr_8879_review.md) | 통합 후보의 공통 보호 TSV 비교를 충족하지 못해 이 head의 수용을 보류합니다. |
| #8881 | [리뷰](../../archives/pr_8881_review.md) | 통합 후보의 공통 보호 TSV 비교를 충족하지 못해 이 head의 수용을 보류합니다. |
| #8901 | [리뷰](../../archives/pr_8901_review.md) | 통합 후보의 공통 보호 TSV 비교를 충족하지 못해 이 head의 수용을 보류합니다. |
| #8903 | [리뷰](../../archives/pr_8903_review.md) | 같은 문단 어울림/TAC 대조군의 fresh WASM 6쪽이 88.56802%로 미달입니다. |
| #8909 | [리뷰](../../archives/pr_8909_review.md) | 통합 후보의 공통 보호 TSV 비교를 충족하지 못해 이 head의 수용을 보류합니다. |
| #8921 | [리뷰](../../archives/pr_8921_review.md) | 통합 후보의 공통 보호 TSV 비교를 충족하지 못해 이 head의 수용을 보류합니다. |
| #8928 | [리뷰](../../archives/pr_8928_review.md) | 명시적 쪽 나눔 뒤 간격 문서의 fresh WASM 3쪽이 89.31429%로 미달입니다. |
| #8929 | [리뷰](../../archives/pr_8929_review.md) | 통합 후보의 공통 보호 TSV 비교를 충족하지 못해 이 head의 수용을 보류합니다. |
| #8930 | [리뷰](../../archives/pr_8930_review.md) | 원 PR의 Linux Canvas render diff CI 실패가 남아 있습니다. Mac의 같은 기본 문턱 대조군 PASS를 Linux 통과로 대신하지 않습니다. |

## 재현·증적 정본

- [실제 체리픽 SHA 대응](cherry-pick.json) · [원 head 대조](source-head-check.json) · [전체 회귀](validation/full-nextest.json) · [함수별 검사](validation/individual-tests.json).
- [전체 원값 비교](visual-final.json) · [보호 문서 비교](protected-final.json) · [입력 96개 Git/LFS 감사](inputs-committed.json).
- [Native producer](producers/native.json) · [fresh WASM producer](producers/wasm.json) · [고정 base producer](producers/fixed-base.json).
- [직접 판독](direct-reading.json) · [추가 미달 판독](additional-direct-reading.json) · [브라우저 실제 소비](validation/actual-browser.json).
- [추가 하락 페이지의 review](protection-diagnostics/issue7905/candidate/review_002.png) · [overlay](protection-diagnostics/issue7905/candidate/overlay_002.png) · [base review](protection-diagnostics/issue7905/base/review_002.png).
- [산출물 SHA-256 목록](artifact-index.json). 비공개 글꼴 바이너리·실행 파일·로그는 asset에 포함하지 않습니다.

기준 PDF를 잘못 연결했던 개인정보 HWPX 초기 실행과 Chrome 중첩 sandbox 초기 실패는 무효 실행으로 구분했습니다. 올바른 HWPX Print PDF와 실제 producer로 다시 산출한 결과만 정본에 포함합니다. PNG를 재사용한 경우 해시와 생산 근거를 [정정 기록](reference-correction.json)에 남겼습니다.
