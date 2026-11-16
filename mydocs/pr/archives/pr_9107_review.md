# PR #9107 리뷰 — HWPX 일반 줄 간격의 판본별 단위

## 최종 판정

**승인.** 판본별 물리 줄간격이라는 원 PR 범위를 독립 데스크톱 Print와 실제 편집·두 형식 반복 저장으로 확인했다. 검증한 통합 후보의 최신 GitHub CI와 mergeability 확인은 병합 전 조건이다.

## 접수 정보와 범위

- [원 PR](https://github.com/edwardkim/rhwp/pull/9107), 작성자 `hyunmyungJaneLee`.
- 원 head `ac181689d3799dcbe549d1a943613724be126fe2`, base `53774cc1508591b6ac794950f0063452fbaf6e87`.
- reviewer `jangster77`, milestone `v1.0.0`을 API로 확인했다.
- #8676의 `hp:switch`가 없는 일반 `lineSpacing` 중 xmlVersion 1.4 이상 비율 외 단위의
  읽기·쓰기 변환을 검토한다. 1.2와 1.4의 FIXED·BETWEEN_LINES 대조군은 각각 독립 Print를 확보했다.

## 검증과 조판 원칙

입력 `samples/issue8676/plain_{fixed,between}_v{12,14}.hwpx`의 저장 제품은 한컴 2020 계열이다.
MCP engine 2020 Print를 사용하며, xmlVersion 1.4라는 이유로 2024 엔진으로 바꾸지 않는다.
base의 1.4 입력 미달을 확인했으며 후보 출력은 별도 실행으로 판정했다.
일반 노드와 switch/default의 실제 선택 경로, 백분율·판본 경계와 serializer 역변환을 대조한다.
기존 #4898 기대값 변경도 독립 단위 근거와 반복 저장 결과로 검토한다.

원 PR의 **웹한글기안기 PDF는 공식 대조 자료로 수용하지 않는다.** 새 데스크톱 Print의
입력/PDF SHA·쪽수·제품·영수증과 페이지별 TSV·raw·review·overlay를 정식 자료로 남긴다.
정본 보호 문서의 공식 두 자리 값도 별도로 확인한다.

## Merge 후 contributor PR comment 계획

최종 CI와 통합 merge SHA를 명시하고 원 PR 주장에 대응하는 1.2/1.4의 같은 페이지 이미지를
본문에 표시한다. 웹한글기안기 자료를 제외하고 실제 한컴 2020 Print로 검증했음을 알린다.
#8676의 해결 범위를 확인한 뒤 원 PR과 완전 해결된 이슈를 종료한다.


## 통합 후보의 최종 로컬 검증 결과

제품 소스 `167921b3c5`와 검증 head `3e9ce11f9c`에서 최소 글꼴 macOS 전체 회귀는
**13,696 PASS·0 FAIL·45 skip**이다. Native Skia·세 Clippy·workspace build·fmt·
suite 정책·unit tier·문서 테스트를 포함한 [Rust 13단계](../assets/all_open_prs_20261116/rust-gates-final.json)는 통과했다.
통합 검토의 공식 데스크톱 Print 66종·74쪽은
[Native](../assets/all_open_prs_20261116/native-claims-final.json)·
[fresh WASM](../assets/all_open_prs_20261116/wasm-claims-final.json) 각각 전쪽 90% 이상이다.
이 집계는 다른 PR의 입력을 이 PR만의 개선으로 계산한 값이 아니다. PR별 주장·반례는 위 접수 범위와
개별 보정 증거로 판정한다. 관계형 검사 보정 뒤 9단계는 이 head에서 다시 실행했고, 제품 및
해당 검사 소스가 바뀌지 않은 Native Skia·직접 PDF·placeholder·doc 4단계는 원 실행 신원을
보존해 재사용했다. [최신 회귀 판정](../assets/all_open_prs_20261116/relation-test-final.json)을 따른다.

[Native/base 비교](../assets/all_open_prs_20261116/native-base-head-comparison.json)와
[WASM/base 비교](../assets/all_open_prs_20261116/wasm-base-head-comparison.json)는 66종·74쪽에서
각각 상승 59쪽·하락 0쪽이다. [공통 보호 8조건·명시적 편람 383쪽](../assets/all_open_prs_20261116/public-gates-final.json)도 통과했다.
[실제 소비](../assets/all_open_prs_20261116/actual-consumers-final.json)는 Native PNG 16개·브라우저 48개를
실행했고 runtime fallback/replay hard 위반은 없다. report-only 최소 잉크량 32 FAIL과 자동 선택
readiness 미실행은 그대로 공개하며 통과로 바꾸지 않았다. [정본 전쪽 보호](../assets/all_open_prs_20261116/protected-final.json)는
Native/fresh WASM 18문서씩 총 3,542행에서 하락·누락 0쪽으로 완료했다. 상승도 0쪽이므로
정본 하한을 변경하지 않았다. [sample10 원본 공급 재현](../assets/all_open_prs_20261116/sample10-original-supply-validation.json)의
base/후보 Native/WASM 네 실행은 각각 전 763쪽 최저 99.57%·보호 하락 0쪽이다.
로컬 필수 검증은 완료했으며 통합 PR 게시 뒤 최신 head CI·병합·후속 처리는 별도 확인한다.



## 원 PR 주장 페이지의 직접 판독

[입력·PDF 신원과 판독 기록](../assets/all_open_prs_20261116/claim-page-evidence/index.json)을 연결했다.

`plain_fixed_v14-hwpx`, 1쪽.

- Native 100.00%: [raw](../assets/all_open_prs_20261116/claim-page-evidence/9107/plain_fixed_v14-hwpx/native/raw-001.png) · [review](../assets/all_open_prs_20261116/claim-page-evidence/9107/plain_fixed_v14-hwpx/native/review/review_001.png) · [overlay](../assets/all_open_prs_20261116/claim-page-evidence/9107/plain_fixed_v14-hwpx/native/overlay/overlay_001.png).
- fresh WASM 100.00%: [raw](../assets/all_open_prs_20261116/claim-page-evidence/9107/plain_fixed_v14-hwpx/wasm/raw-001.png) · [review](../assets/all_open_prs_20261116/claim-page-evidence/9107/plain_fixed_v14-hwpx/wasm/review/review_001.png) · [overlay](../assets/all_open_prs_20261116/claim-page-evidence/9107/plain_fixed_v14-hwpx/wasm/overlay/overlay_001.png).

`plain_between_v14-hwpx`, 1쪽.

- Native 100.00%: [raw](../assets/all_open_prs_20261116/claim-page-evidence/9107/plain_between_v14-hwpx/native/raw-001.png) · [review](../assets/all_open_prs_20261116/claim-page-evidence/9107/plain_between_v14-hwpx/native/review/review_001.png) · [overlay](../assets/all_open_prs_20261116/claim-page-evidence/9107/plain_between_v14-hwpx/native/overlay/overlay_001.png).
- fresh WASM 100.00%: [raw](../assets/all_open_prs_20261116/claim-page-evidence/9107/plain_between_v14-hwpx/wasm/raw-001.png) · [review](../assets/all_open_prs_20261116/claim-page-evidence/9107/plain_between_v14-hwpx/wasm/review/review_001.png) · [overlay](../assets/all_open_prs_20261116/claim-page-evidence/9107/plain_between_v14-hwpx/wasm/overlay/overlay_001.png).

## 공개 API 편집·반복 저장 검증

제품 소스 `167921b3c5`의 fresh WASM 공개 API로 xmlVersion 1.2/1.4 × FIXED/BETWEEN 4개
원본의 TOP 문단에 `X`를 입력하고 HWP/HWPX 두 형식으로 저장→재열기→재저장→재열기를 실행했다.
**8조건·16회 재열기** 모두 편집한 본문, AONE/ATWO의 줄간격 종류·물리값·줄 피치와 1쪽을 보존했다.
[실행 기록](../assets/all_open_prs_20261116/public-api-save-validation/linespacing-repeat-save-result.json)에
입력·WASM·저장 바이트 SHA와 각 단계의 속성·피치를 기록했다. Node의 공개 API 구조 검증이며
브라우저 Canvas 또는 픽셀 일치율 검증으로 표현하지 않는다. 독립 데스크톱 Print와 실제 화면은
별도 전쪽 TSV·주장 페이지 증적을 따른다.

공식 FIXED/BETWEEN_LINES 1.2/1.4 네 입력은 Native/fresh WASM 모두 100.00%이며,
[공개 편집·반복 저장](../assets/all_open_prs_20261116/public-api-save-validation/linespacing-repeat-save-result.json)에서
8개 형식 조건의 16회 재열기를 통과했다. AT_LEAST는 같은 비율 외 단위 분기를 사용하지만
AT_LEAST의 별도 독립 Print 측정까지 실행한 것으로 확대하지 않는다.

## 통합 PR #9143의 code head CI

[통합 PR #9143](https://github.com/edwardkim/rhwp/pull/9143)의 code head
`8fdcff552506c7d4a51edf6fcbebe2d63a1a729a`에서 [CI](https://github.com/edwardkim/rhwp/actions/runs/43948529913),
CodeQL·Render Diff·Proptest·Adapter·CI 정책 판정이 모두 완료·성공했다.
원격 기본 회귀 다섯 archive는 **13,511 PASS·0 FAIL·45 skip**이며 로컬 최소 글꼴 macOS의
13,696 PASS와 실행 환경·선택 범위를 구분한다. Native Skia와 frontend package gate도 성공했다.
[정확한 workflow·job·head와 실제 summary](../assets/all_open_prs_20261116/code-ci-final.json)를 보존했다.
문서 전용 trailing의 최신 required check·mergeability를 확인한 뒤 통합하며, 아직 merge나 원 PR 종료가
완료됐다고 기록하지 않는다. contributor 원 head·fork branch는 보존한다.
