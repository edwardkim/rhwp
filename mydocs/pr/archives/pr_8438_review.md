# PR #8438 리뷰 — 굵은 글자 폭과 HWP3·글상자 조판 통합

## 최종 판정

**통합 머지 완료 — 누락된 원 PR별 검토 보완 중.** [통합 PR #8438](https://github.com/edwardkim/rhwp/pull/8438)은 `b8338ba38be288b6e9385f5372d184649323aa9f`로 머지됐다. 기존 기록은 네 원 PR만 범위에 적었으나 아래 19개 원 PR도 실제 체리픽 이력에 포함됐다. 전체 회귀 통과와 개별 PR의 기능·시각 검토는 구분한다.

## 접수와 변경 범위

- 통합 PR [#8438](https://github.com/edwardkim/rhwp/pull/8438), 작성자 `jangster77`, 대상 `devel`.
- 검증 코드 head: `ee741d74476423d4cd4f995c49af27ff0cb853fd`; 조판 제품 코드 검증: `d536455b4e`. 정책 비교 base: `1a7ae8f5bd17ebc22b281fd1915973e18e4e641f`.
- 원 PR별 적용 SHA·주장·검증은 [#8161](pr_8161_review.md), [#8346](pr_8346_review.md), [#8356](pr_8356_review.md), [#8405](pr_8405_review.md)에 각각 기록했다. 관련 이슈 #8079·#8304·#8306·#8399는 Refs다.
- 굵은 글자의 보통 면 전진폭, 암호 HWP3 문서의 찾아보기·배경·간격, HWPX 정규화 검증, 글상자 외곽의 줄 배제를 다룬다. 메인터너는 표 뒤 문단 원점과 글꼴 메트릭 경로를 보정하고 진단 로그에서 문서 본문을 제거했다.

## 누락된 원 PR 범위 정정

[#8297](pr_8297_review.md) · [#8316](pr_8316_review.md) · [#8321](pr_8321_review.md) · [#8333](pr_8333_review.md) · [#8336](pr_8336_review.md) · [#8338](pr_8338_review.md) · [#8341](pr_8341_review.md) · [#8354](pr_8354_review.md) · [#8357](pr_8357_review.md) · [#8364](pr_8364_review.md) · [#8365](pr_8365_review.md) · [#8375](pr_8375_review.md) · [#8376](pr_8376_review.md) · [#8378](pr_8378_review.md) · [#8380](pr_8380_review.md) · [#8386](pr_8386_review.md) · [#8396](pr_8396_review.md) · [#8402](pr_8402_review.md) · [#8404](pr_8404_review.md).

기존 본문에서 이 원 PR들을 누락했다. 각 원 PR의 head는 기존 리뷰의 조회값과 같으며 커밋은 리베이스된 SHA로 통합 이력에 있다. 개별 리뷰에서 실제 대응 커밋과 독립 증거를 검토하고, 확정되지 않은 항목을 전체 통과로 표시하지 않는다.

## 검증 결과

- 최소 글꼴 전체 Rust 회귀 12,368건 통과, 45건 skipped. Native Skia 라이브러리 3,858건 통과·12건 ignored, 그림 placeholder 2건과 직접 PDF 검사 22건 통과. 이후 이동한 테스트와 진단 로그 변경 관련 검사 10건을 다시 통과했다.
- `cargo fmt --all -- --check`, Native/WASM Clippy, workspace build·all-target Clippy, Rust suite manifest·unit tier, Studio Node 2,715건과 TypeScript 검사를 통과했다.
- 코드 head의 [Full CI](https://github.com/edwardkim/rhwp/actions/runs/41322587728), [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/41322587755), [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/41322587627), [Adapter](https://github.com/edwardkim/rhwp/actions/runs/41322587676), [Proptest](https://github.com/edwardkim/rhwp/actions/runs/41322587718)가 완료됐다. CodeQL 종합 검사의 경고는 사용자 확인에 따라 GitHub에서 분류된 후 통과했다. 남은 다른 보안 경고는 이 PR에서 변경하지 않았다.
- #8389·#8397의 기존 표시값 기준 Native·fresh WASM 대조 2,153행, 중복을 제외한 1,770쪽에서 표시값 하락이 없었다. [Native 대조표](../assets/planet6897_integration_final/pr8389_8397/native-display-comparison.tsv) · [WASM 대조표](../assets/planet6897_integration_final/pr8389_8397/wasm-display-comparison.tsv) · [입력과 비교 조건](../assets/planet6897_integration_final/pr8389_8397/README.md).

## 시각 증적

각 입력과 독립 한컴 Print PDF의 경로·SHA-256은 [시각 증적 설명](../assets/planet6897_integration_final/README.md)에 있다. 기존에 산출한 Native/fresh WASM TSV와 같은 페이지의 review·overlay를 재사용했다.

| 대상 | 전쪽 최저 Native / WASM | 검토 자료 |
| --- | --- | --- |
| #8161, 113424, 46쪽 | 91.51% / 91.60% | [Native TSV](../assets/planet6897_integration_final/pr8161/native-113424.tsv) · [WASM TSV](../assets/planet6897_integration_final/pr8161/wasm-113424.tsv) · [5쪽 review](../assets/planet6897_integration_final/pr8161/native-113424-p005-review.png) · [overlay](../assets/planet6897_integration_final/pr8161/native-113424-p005-overlay.png) |
| #8346, 암호 HWP3, 24쪽 | 98.27% / 98.28% | [Native TSV](../assets/planet6897_integration_final/pr8346/native-hwp3-original.tsv) · [WASM TSV](../assets/planet6897_integration_final/pr8346/wasm-hwp3-original.tsv) · [17쪽 review](../assets/planet6897_integration_final/pr8346/native-hwp3-p017-review.png) · [overlay](../assets/planet6897_integration_final/pr8346/native-hwp3-p017-overlay.png) |
| #8405, A·D·E·P HWP/HWPX | 99.98% 이상 / 99.98% 이상 | [원 PR 리뷰와 입력별 TSV](pr_8405_review.md) · [D HWPX review](../assets/planet6897_integration_final/pr8405/native-D-hwpx-review.png) · [overlay](../assets/planet6897_integration_final/pr8405/native-D-hwpx-overlay.png) |

#8356은 저장 정규화와 실제 손실 검출에 관한 CLI 변경이며 조판 출력 변경이 아니다. 각 변경의 확대 판독과 남은 글자 가장자리 차이는 개별 리뷰에 기록했다.

## 머지 후 처리

정확한 trailing head CI 확인과 사용자 승인에 따라 머지한 뒤 원 PR별로 통합 결과와 해당 원본의 증거를 연결한다. 최종 merge SHA로 고정한 링크를 사용하고 게시 결과를 재조회한다.
