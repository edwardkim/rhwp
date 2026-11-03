# [PR #8365](https://github.com/edwardkim/rhwp/pull/8365) 리뷰 — fix(pdf-import): 편집 우선 가져오기의 고정 줄간격 문단이 한/글처럼 마지막 줄까지 고정 피치를 차지한다 ([이슈 #8355](https://github.com/edwardkim/rhwp/issues/8355) 일부)

## 최종 판정

**메인터너 보정 후 수용 가능** — 원 PR의 적용 내용은
[통합 PR #8438](https://github.com/edwardkim/rhwp/pull/8438)의 머지 SHA
`b8338ba38be288b6e9385f5372d184649323aa9f`에 반영됐다. 원 head를 중복 병합하지
않는다. 후속 제품 보정 `5aa6ee08dd`와 회귀 보강 `c8e4f0ffac` 기준으로 이 PR의
입력과 독립 한컴 Print PDF를 대조한 전쪽 Native/fresh WASM 검증에서 90% 미만과
고정 devel 대비 2%p 초과 하락은 없다. [원 PR 입력별 전쪽 TSV](../assets/review_8438_followup/final-original-inputs/README.md)와
아래의 주장 페이지 review·overlay를 함께 판독했다. 최소 글꼴 전체 Rust 회귀
12,387건 통과·45건 skipped, 선행 [PR #8389](https://github.com/edwardkim/rhwp/pull/8389)·
[PR #8397](https://github.com/edwardkim/rhwp/pull/8397) 2,153쪽 Native/fresh WASM
표시값 하락 0쪽이다. 후속 보정 [PR #8517](https://github.com/edwardkim/rhwp/pull/8517)의 최신 head CI와
병합을 확인했다. 원 PR은 증적 댓글 게시 뒤 중복 반영 사유로 종료한다.

## 2026-11-02 통합 머지 사실과 적용 SHA 재확인

[통합 PR #8438](https://github.com/edwardkim/rhwp/pull/8438)은 `b8338ba38be288b6e9385f5372d184649323aa9f`로 머지됐다. 아래 체리픽 기록의 종전 통합 SHA는 리베이스 전 이력이다. 머지된 이력에서 원본과 커밋 제목이 일치하는 대응은 다음과 같다. 이는 변경 내용의 완전한 동등성이나 개별 시각 판정을 대신하지 않는다.

- 원본 → 머지 이력: `674a06d9cd` → `16bc9071e2`, `0b24ab5ee3` → `bcfcaa0a2a`, `3978c9c8b1` → `69997e5d33`, `5fe53b6a66` → `2dc2a8ad72`, `e735b252e1` → `0451ecf545`, `e1feb8bf7d` → `7890fe3800`, `99dac8a02b` → `bcfb2b6dbf`, `1d9456e754` → `7963cd3355`.
- 제품 코드 `d536455b4e`의 최소 글꼴 전체 회귀는 12,368건 통과·45건 skipped였다. 이 로그에서 확인한 해당 모듈: `issue_8355_pdf_editable_fixed_trailing` 4건. 이 수치는 개별 PR의 독립 Print 검증과는 구분한다.
- 뒤의 중간 후보·검증 대기 문장은 당시 상태이며 최종 통합 결과가 아니다. 원 PR별 최종 시각·기능 검토와 최신 head 일치 확인은 계속한다.

2026-11-03 00시경 KST에도 원 PR head와 열린 상태를 재확인했다. [일괄 조회 결과](../assets/review_8438_followup/original-pr-heads-20261103.md)는 기능·시각 최종 판정을 대신하지 않는다.

## 접수와 적용

- 원 PR: [PR #8365](https://github.com/edwardkim/rhwp/pull/8365), 작성자 `planet6897`, reviewer `jangster77`.
- 원 PR head 조회값: `1d9456e7541afb619d7efd605bdf0a27a18ba93c`. 조회 시 원 PR head와 마지막 체리픽 원본이 일치한다.
- 마지막 적용 원본: `1d9456e7541afb619d7efd605bdf0a27a18ba93c`. 검증 시작 당시 코드 후보: `abea5b028f8f135bdf000ca700e1993ebf0b1973`, 기준 `upstream/devel`: `bc9dfdd057258de59afebeb5121c4bee7398e07f`.
- 관련 이슈: [이슈 #8355](https://github.com/edwardkim/rhwp/issues/8355).
- 선행 PR: 별도 누적 선행 PR이 명시되지 않았다.

후속 `e1f03ffbb3`은 Python 테스트 준비, `af175315e7`과 `e920cba6b0`은 회귀 검사 주석, `a63bb719de`는 검증 경과를 기록했다. `5d1294811b`은 전체 그림의 잘림 오판을 보정했다. `e1b8ab0f54`, `d6fe30c3ad`, `0897582f6c`, `8c4eca265e`에 Native 진단, [PR #8410](https://github.com/edwardkim/rhwp/pull/8410) 집중 6건 통과와 글꼴 대조를 기록했으며 새 코드의 전쪽 출력 검증이 필요하다.

## 변경 주장과 검토 관점

PDF 편집 우선 가져오기의 고정 줄간격 문단에서 마지막 줄도 고정 피치를 점유하게 한다.
마지막 줄과 뒤 본문의 배치·분리 및 연속 문단 대조군을 확인한다.

## 체리픽과 보정

- 원본 `674a06d9cdfe403a53bdb41905a54b41bae1b6e7` → 통합 `5cd7ea000bb3907f09182e62ea7605efd81779a7`.
- 원본 `0b24ab5ee382c3f2b48bfdff4a8d29c7ab6bf38b` → 통합 `037c3eb95a4f35106e1fe72133b2d5ed94146d4f`.
- 원본 `3978c9c8b1f8762b72d7130157e966f9c11bbc5f` → 통합 `aa97e460e6a0675394f500a828be20706f4f313f`.
- 원본 `5fe53b6a662862d329ac84820823861da4e79e1a` → 통합 `9942b47fb05d32e351e1eb2d5bc33d8bf58e925a`.
- 원본 `e735b252e1e67a45906c81e2ae859fe2b12badeb` → 통합 `3236a008c00445dffa3a3519523b2d394775f39b`.
- 원본 `e1feb8bf7dd90fe4778c179cc23a3cc3987a207c` → 통합 `19230187889eeaf653e2630b851fd024fbf27a5e`.
- 원본 `99dac8a02b491c70bece5b44a33893fe8d3b4bfa` → 통합 `b53d15fd6597d343b72fc3c8365187b42643c211`.
- 원본 `1d9456e7541afb619d7efd605bdf0a27a18ba93c` → 통합 `65689e6610901f35dab71b9fc51c0e14f07fbdb4`.

이 PR의 원본 커밋 적용 중 별도로 기록된 충돌은 없다.

## 검증과 증거

이전 통합 후보의 최소 글꼴 전체 회귀 로그에서 이 PR 관련 다음 모듈의 실행을 확인했다. 새 base 후보의 재실행 결과는 이 값으로 대신하지 않는다.
- [`issue_8355_pdf_editable_fixed_trailing.rs`](../../../tests/cases/issue_8355_pdf_editable_fixed_trailing.rs): 이전 실행 4건 통과, 0건 실패, 0건 건너뜀.

검토 후보에 포함된 입력 바이트를 Git blob과 대조했다. 아래 자료는 원 PR의 제출 증거이며 새 후보의 실행 결과를 뜻하지 않는다.
- [`editable-import-after-2024.pdf`](../../../pdf/issue8355/editable-import-after-2024.pdf) — SHA-256 `318476b97bdc7f779571c7887b8fa08ec1085129e8e0ac4a71529507040a0725`.
- [`editable-import-before-2024.pdf`](../../../pdf/issue8355/editable-import-before-2024.pdf) — SHA-256 `5955905454d2d3c789c2f4221441fe8bda90c4d616d8a871694b756ffeb4e112`.
- [`21_언어_기출_편집가능본-2010.pdf`](../../../samples/21_언어_기출_편집가능본-2010.pdf) — SHA-256 `609d56421009ccab5a50cb2a8f4e78af4d0961de2e7ae85d811905e8434ac251`.

기존 생성 목록으로 최소 글꼴 회귀 12,320건 통과·45건 건너뜀을 확인했다. 생성 목록에서 누락된 [PR #8410](https://github.com/edwardkim/rhwp/pull/8410) 두 모듈의 6건을 목록 재작성 후 같은 환경에서 별도로 실행해 모두 통과했다. 두 실행을 구분하며 한 번의 전체 실행으로 기록하지 않는다.

2026-11-01 고정 후보의 [PR #8389](https://github.com/edwardkim/rhwp/pull/8389)·[PR #8397](https://github.com/edwardkim/rhwp/pull/8397) 전쪽 정밀 대조에서 2,153행 중 Native 12행, WASM 8행이 공개 기준보다 낮았다. 이 수치는 이후 GIF·Canvas 보정 전 결과이며 최종 후보의 재검증 결과가 아니다. [PR #7905](https://github.com/edwardkim/rhwp/pull/7905) 3쪽은 GIF 오른쪽 경계 누락, sample16 35쪽 WASM은 메타파일 내부의 미대응 3픽셀이 확인됐다. [PR #8040](https://github.com/edwardkim/rhwp/pull/8040) 두 원본은 Native·WASM 전쪽이 90% 이상이나 Native 정밀 기준 하락이 남는다. 진행과 근거는 [작업 기록](../../working/task_m100_planet6897_system_fonts_20261031.md)을 따른다. 하락 보정과 전쪽 대조, 각 PR의 독립 Print·Native/fresh WASM 및 직접 판독을 완료해야 한다. [PR #8410](https://github.com/edwardkim/rhwp/pull/8410)·[PR #8412](https://github.com/edwardkim/rhwp/pull/8412)의 새 base 기능도 별도로 보존 검증한다.

원 PR의 최신 head·required check와 통합 후보의 새 검증 결과를 확인하기 전에는 승인이나 merge를 진행하지 않는다.

## 2026-11-02 일괄 대조 추가 증거

현재 통합 코드로 `samples/21_언어_기출_편집가능본-2010.pdf`를 편집 우선
가져오기 해 저장한 [HWP](../assets/issue_8381/followup8438/editable-saved.hwp)와
그 파일의 [한컴 2024 Print PDF](../../../pdf/issue8361/batch-editable-saved-2024.pdf)를
96dpi에서 같은 16쪽으로 대조했다. MCP 작업과 파일 해시는 [기준 기록](../assets/issue_8373/followup8438/print-sources.md)에 있다.
Native 최저 98.79%, fresh WASM 최저 98.72%이며 양쪽 모두 90% 미만은 없다.
[Native 전체 TSV](../assets/issue_8381/followup8438/native-all.tsv) ·
[WASM 전체 TSV](../assets/issue_8381/followup8438/wasm-all.tsv).
12쪽의 글상자 끝과 뒤 문항을 직접 대조했고 고정 줄간격 문단과 다음 내용 관계는 기존 회귀에서도 검사했다.
[12쪽 Native review](../assets/issue_8381/followup8438/native-review-012.png) · [12쪽 Native overlay](../assets/issue_8381/followup8438/native-overlay-012.png) ·
[12쪽 fresh WASM review](../assets/issue_8381/followup8438/wasm-review-012.png) · [12쪽 fresh WASM overlay](../assets/issue_8381/followup8438/wasm-overlay-012.png).

### 새 base의 저장본 확인

검증 기준 base
`d3f7209bbb0fdb1711c5f40a70ad22f0ca0be66d`에서 `target/pr-review` Native
실행 파일(SHA-256 `34dfddd062947e91c33cb3c2b2c16a10f9903d2d1184fcdb646683e6a0dc2f1f`)로
같은 PDF를 다시 편집 우선 가져오기 했다. 결과 HWP는 16쪽이며 SHA-256
`edaf8172427572450503f3265fca95b1c1d2c9673c1e5b25528f5b55294a043f`로 위 기존
Print 입력과 바이트 단위로 일치한다. 새 base와 이전 검증 코드 사이의 렌더러·
파서·HWPX 저장기는 변경되지 않았고 후속 브랜치에도 제품 코드 차이가 없어
위 기존 TSV와 Print 이미지를 재사용한다.

### 현재 저장 후보의 공통 입력 재확인

제품 코드 `dd284b5cda`로 위 PDF를 편집 우선 가져오기 해 HWP로 저장했다. 16쪽이며 결과 SHA-256 `edaf8172427572450503f3265fca95b1c1d2c9673c1e5b25528f5b55294a043f`는 위 한컴 Print에 사용한 저장 HWP와 일치한다. [PR #8365](https://github.com/edwardkim/rhwp/pull/8365)·[PR #8375](https://github.com/edwardkim/rhwp/pull/8375)·[PR #8380](https://github.com/edwardkim/rhwp/pull/8380)이 같은 입력을 사용해 한 번 실행한 결과다. 이는 입력 바이트의 재확인이며 현재 시험 중인 WMFC 변경 이후의 화면 검증은 아니다.

### 2026-11-02 새 base 최소 글꼴 집중 회귀

검증 기준 base
`d3f7209bbb0fdb1711c5f40a70ad22f0ca0be66d`에서 `issue_8355_pdf_editable_fixed_trailing` 4건을 실행해 모두 통과했다.
전체 선택 범위는 28건 통과이며 전체 회귀 실행 결과가 아니다. 시스템 글꼴을 격리하고
저장소의 재배포 가능한 최소 글꼴만 노출했다. 기존 Print 시각 증거와는 별도로 판정한다.

### 2026-11-02 의존성 리베이스 후 전체 Rust 회귀

최소 글꼴 환경에서 전체 선택 12,369건을 두 실행으로 완료해 모두 통과했다.
첫 실행의 중단과 나머지 307건의 재실행, 정확한 목록 대조는
[공통 실행 근거](../assets/review_8438_followup/rebase-de2295-rust.md)에 기록했다.
이는 이 PR의 시각 검증이나 최종 판정을 대신하지 않는다.

### 2026-11-03 최종 코드의 원 PR 입력 전쪽 재검증

제품 코드 `d56930dda7`의 Native와 같은 코드로 새로 빌드한 fresh WASM으로
이 PR과 관련된 공통 편집 저장본 16쪽을 각 독립 한컴 Print PDF와 다시 대조했다.
전쪽 최저는 Native 98.79%, fresh WASM 98.72%이며, 모든 입력에서 쪽수 일치·누락 0쪽·90% 미만 0쪽이다.
[입력별 SHA-256·전체 Native/fresh WASM TSV](../assets/review_8438_followup/final-original-inputs/README.md)에
정확한 원본·PDF·페이지 수와 전쪽 결과를 연결했다. 이 검사는 기존 본문의
주장 영역 review/overlay 직접 판독과 함께 판단한다.

2026-11-03 현재 코드의 주장 페이지 그림 재산출: 제품 코드 `d56930dda7`의 Native와 fresh WASM으로 같은 입력·Print PDF의 12쪽을 다시 그렸다. 위 review/overlay 이미지가 최신 산출물이며, 전체 16쪽 TSV는 기존 정식 TSV와 SHA-256이 각각 `26c24d0363511691cb07e63ef0c353c8517eefc0013140bc508509d61cd7a0c3`·`85fe94720b9471cdb0340b31bb754499e3de298672ef820241bebb6f75841cb0`으로 바이트 단위 일치했다.

## 2026-11-03 후속 보정 PR의 원격 완료 확인

후속 보정 [PR #8517](https://github.com/edwardkim/rhwp/pull/8517)은 code head
`ba0557118ad896f1c410e80da5100fedf10139d8`의
[Build & Test](https://github.com/edwardkim/rhwp/actions/runs/41592450512),
[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/41592450498),
[Render Diff](https://github.com/edwardkim/rhwp/actions/runs/41592450511),
Adapter·Proptest와 CI Impact Policy 통과 뒤
`c97709b59742372a817b3886f442144728fca646`로 `devel`에 병합됐다.
이 원격 완료는 위의 개별 원본·Print PDF·전체 페이지 TSV·대표 비교 판정과 함께
확인한다. 사용자 지정 옵션 2로 이 리뷰와 오늘할일을 문서 전용 후속 PR에 반영하고,
원 PR에는 해당 증적을 게시한 뒤 중복 병합 없이 종료한다.

## Merge 후 contributor PR comment 계획

- 원 PR의 편집 입력 고정 줄간격 기여가 [PR #8438](https://github.com/edwardkim/rhwp/pull/8438)에
  통합된 사실과 [PR #8517](https://github.com/edwardkim/rhwp/pull/8517)의
  메인터너 보정·최종 병합 SHA를 구분해 한국어로 안내한다.
- 같은 편집 가능 입력의 한컴 Print 12쪽과 비교한 raw 일치율은 Native
  **99.70%**, fresh WASM **99.69%**다.
  [review](../assets/issue_8381/followup8438/native-review-012.png)·[overlay](../assets/issue_8381/followup8438/native-overlay-012.png) 및
  [Native 전쪽 TSV](../assets/review_8438_followup/final-original-inputs/pr8365-8375-8380-editable-native.tsv)·[fresh WASM 전쪽 TSV](../assets/review_8438_followup/final-original-inputs/pr8365-8375-8380-editable-wasm.tsv)를
  댓글에 연결한다. 대표 이미지는 병합 SHA에 고정한 raw URL로 직접 표시한다.
- 옵션 2 문서 PR에 이 기록이 반영된 뒤 원 PR head·상태를 다시 확인하고,
  기여자에게 증적 댓글을 게시한 다음 중복 병합 없이 종료한다.

## 2026-11-03 원 PR 댓글·종료 확인

원 PR [#8365](https://github.com/edwardkim/rhwp/pull/8365)에 주장 입력·페이지와 일치하는
[검토 댓글](https://github.com/edwardkim/rhwp/pull/8365#issuecomment-6598199158)을 게시하고
중복 병합 없이 닫았습니다. 게시 본문은 GitHub API로 다시 읽어 확인했습니다.
관련 [이슈 #8355](https://github.com/edwardkim/rhwp/issues/8355): 열림 — 편집 후 18쪽 입력.
코드 [PR #8517](https://github.com/edwardkim/rhwp/pull/8517)의 병합 SHA는
`c97709b59742372a817b3886f442144728fca646`, 검토 문서
[PR #8523](https://github.com/edwardkim/rhwp/pull/8523)의 병합 SHA는
`014be1516bfd4e4c91b0b5f50678fecaa6a1de1f`입니다.
