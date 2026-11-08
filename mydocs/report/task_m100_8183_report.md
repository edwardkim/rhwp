# #8183 결과 보고 — PDF 이미지 코덱 개별 opt-in

Issue: [#8183](https://github.com/edwardkim/rhwp/issues/8183)

JBIG2와 JPEG 2000을 `pdf-jbig2`, `pdf-jpeg2000`으로 개별 선택할 수 있게 했다.
기본값은 비활성이며 각 선택은 PDF 가져오기를 포함한다. Native CLI·Rust library·WASM이
같은 backend 기능을 소비하고, 실제 합산된 Cargo feature를 지원 정보에 반영한다.

## 구현과 계약

- 실제 decoder의 예약·반복을 요청별 자원 예산에 연결했다. 한도·거부 조건을 완화하거나
  무제한 fallback으로 우회하지 않는다. 전체 parser의 할당 증명은 `false`를 유지한다.
- 공유 사전·palette·PDF Indexed·alpha·mask의 source sample과 저장 후 리소스를 보존한다.
- CLI `capabilities`·가져오기 metadata·WASM 사전 조회가 같은 지원 정보를 제공한다.
- 네 codec/resource vendor package의 source·license·lock과 변경 근거를 보존했다.
- [설치·빌드·사용·오류 가이드](../manual/pdf_image_codecs.md)를 보강했다.

## 검증과 성능

base는 `d9fd85ed5`, 제품 source는 `cfc5be8fe`, 정식 테스트 source는 `2a58c5f37`다.
이후 변경은 문서·증적 표시·메인테이너 결정 기록이다.

- Native·최적화한 fresh WASM 네 구성 각각 19건, 실제 Studio·Worker의 실패·복구를 확인했다.
- 원본 PDF 기준 17쪽의 Native/fresh WASM raw 최저 100%, 저장 후 표시도 100%다.
  같은 HWPX의 base 대조에서 0.1pp 이상 하락은 없다.
- 전체 `release-test` 11,366 PASS / 0 FAIL / 45 skip, 선택 코덱 정식 회귀 7 PASS,
  요청 자원 helper scoped 검사 4 PASS, 필수 lint·Native Skia·doc tests·설치 검증을 완료했다.
- Studio TypeScript와 2361 PASS / 0 FAIL / 기존 2 skip을 확인했다.
- 같은 JPEG의 11회 교대 측정에서 일관된 큰 지연 증가는 관측되지 않았다. 양쪽 코덱 WASM의
  200 DPI JPEG 중앙값은 base 대비 +1.91%, 산출물은 default 대비 +553,400 bytes(+2.544%)다.
  측정 범위·min–max·RSS와 선형 heap의 차이를 구분해 [원문](../pr/assets/issue_8183/performance/README.md)을 보존했다.

## 반투명 이미지의 메인테이너 결정

2026-10-24 메인테이너가 [비교판](../working/task_m100_8183_alpha_visual_review.md)을 확인해
두 4×2 alpha 사례의 RHWP 표현을 정답으로 수용하고 PR 진행을 결정했다.
한컴 2024 Print의 1비트 mask 출력에 RHWP를 맞추지 않는다. 원래 Print 대조의
27.39104%·52.99065%와 `re_review_required`를 보존하며, 해당 두 Print를 alpha의 정답지로
사용하지 않는다. 다른 문서의 시각 gate를 면제하는 결정이 아니다.
[결정 원문과 범위](../working/task_m100_8183_alpha_visual_review.md#메인테이너-결정--2026-10-24)를 따른다.

## 제출 범위와 이후 조건

- 이번 승인은 devel 대상 Open PR 제출이다. 최신 head CI·정식 review·merge는 이후 절차다.
- #8174의 MSRV·패키지 배포 결정, crates.io 게시·0.9.0 릴리스는 이번 PR에서 수행하지 않는다.
- source preflight는 PASS이나 `publish_ready=false`다. 기존 lockfile의 yank 경고도
  [3단계 기록](../working/task_m100_8183_stage3.md)에 남겼으며 registry 배포 안전 판정을 대신하지 않는다.

[수행 계획](../plans/task_m100_8183.md) · [구현 계획](../plans/task_m100_8183_impl.md) ·
[2단계](../working/task_m100_8183_stage2.md) · [3단계 상세](../working/task_m100_8183_stage3.md) ·
[전체 TSV·PNG](../pr/assets/issue_8183/README.md)
