# PR #9137 리뷰 — 저장 줄이 있는 글상자 표의 세로 정렬

## 최종 판정

**메인터너 보정 후 수용 가능.** 제한/비제한 머리 경계와 본문 커서의 공용 프레임 소비를 보정하고, 부적합한 고정 좌표 검사를 원본 저장값의 관계 검사로 교체한 통합 후보를 검증했다. 보정 SHA를 포함한 통합 PR의 최신 CI와 mergeability를 확인한 뒤 통합한다.

## 접수 정보와 범위

- [원 PR](https://github.com/edwardkim/rhwp/pull/9137), 작성자 `hyunmyungJaneLee`.
- 원 head `125f8c80c2eec043f604b1fc6d9815e3e3e7dcaf`, base `53774cc1508591b6ac794950f0063452fbaf6e87`.
- reviewer `jangster77`, milestone `v1.0.0`을 API로 확인했다.
- #9108의 저장 줄이 있는 경로를 다루는 #9109 후속 PR이며 공통 커밋 6개는 중복 적용하지 않았다.

## 검증과 조판 원칙

`samples/textbox_stored_table/`의 TopBottom·앞으로 배치 표, 오프셋 0/2000, 위/가운데/아래 정렬,
띠 아래로 밀린 저장 앵커를 독립 Print와 대조한다. stored 단위와 실제 개체 원점을 구분하고
측정 높이·정렬·paint가 같은 표의 소속과 바깥 상자를 소비하는지 확인한다.
저장 줄 없는 정상 대조군과 HWP/HWPX 양쪽 저장 경로도 포함한다.

원 PR의 **웹한글기안기 PDF는 공식 대조 자료가 아니다.** 같은 입력의 한컴 2020 데스크톱
Print를 확보했다. `textbox_nostored_header_front_tall.hwpx`는 이 독립 Print에서도 base raw
33.04%이므로 정상 시각 대조군으로 취급하지 않으며, 후보·주장과 기존 결함의 경계를 별도로 확인한다.
`_pushed`의 원 PR 잔여 간격도 공식 Print로 다시 확인한다. 기존 점수를 복사해 수용하지 않는다.

## Merge 후 contributor PR comment 계획

통합 PR·merge SHA·CI와 실제 Print 제품을 한국어 존댓말로 알린다. 저장 줄이 있는 이 PR의
각 주장에 대응하는 같은 쪽 raw·review·overlay를 본문에 표시한다. #9109 공유 범위와
메인터너 보정 범위가 있으면 구분하며 #9108의 완전 해결 여부를 확인한 뒤 종료한다.


## 제한 표 대조군의 보정과 집중 검증

독립 Print에서 확인한 제한 표·자유 표의 차이를 공통 프레임 계산과 실제 본문 커서에
함께 반영했다. 제품 source `167921b3c5`의 원본·세 단일 속성 대조군·표 없는 대조군은
Native/fresh WASM 모두 100.00%다. 기존 저장 줄 검사 3건과 새 관계형 2건·
완료 쪽 Print 위치 고정형 1건, 총 6건은 최소 글꼴 환경에서 통과했다.
[원인과 최종 집중 결과](../assets/all_open_prs_20261116/header-front-cause.md),
[Native raw](../assets/all_open_prs_20261116/frame-boundary-validation/restricted-header/native/rhwp_001.png)·
[review](../assets/all_open_prs_20261116/frame-boundary-validation/restricted-header/native/review_001.png)·
[overlay](../assets/all_open_prs_20261116/frame-boundary-validation/restricted-header/native/overlay_001.png),
[WASM review](../assets/all_open_prs_20261116/frame-boundary-validation/restricted-header/wasm/review_001.png)·
[overlay](../assets/all_open_prs_20261116/frame-boundary-validation/restricted-header/wasm/overlay_001.png)를 보존했다.
전체 원 PR 입력·정본 보호·전체 회귀도 아래 최종 검증으로 완료했다.


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

`textbox_stored_center_tb_v2000-hwpx`, 1쪽.

- Native 100.00%: [raw](../assets/all_open_prs_20261116/claim-page-evidence/9137/textbox_stored_center_tb_v2000-hwpx/native/raw-001.png) · [review](../assets/all_open_prs_20261116/claim-page-evidence/9137/textbox_stored_center_tb_v2000-hwpx/native/review/review_001.png) · [overlay](../assets/all_open_prs_20261116/claim-page-evidence/9137/textbox_stored_center_tb_v2000-hwpx/native/overlay/overlay_001.png).
- fresh WASM 100.00%: [raw](../assets/all_open_prs_20261116/claim-page-evidence/9137/textbox_stored_center_tb_v2000-hwpx/wasm/raw-001.png) · [review](../assets/all_open_prs_20261116/claim-page-evidence/9137/textbox_stored_center_tb_v2000-hwpx/wasm/review/review_001.png) · [overlay](../assets/all_open_prs_20261116/claim-page-evidence/9137/textbox_stored_center_tb_v2000-hwpx/wasm/overlay/overlay_001.png).

`textbox_stored_top_tb_v0_pushed-hwp`, 1쪽.

- Native 97.48%: [raw](../assets/all_open_prs_20261116/claim-page-evidence/9137/textbox_stored_top_tb_v0_pushed-hwp/native/raw-001.png) · [review](../assets/all_open_prs_20261116/claim-page-evidence/9137/textbox_stored_top_tb_v0_pushed-hwp/native/review/review_001.png) · [overlay](../assets/all_open_prs_20261116/claim-page-evidence/9137/textbox_stored_top_tb_v0_pushed-hwp/native/overlay/overlay_001.png).
- fresh WASM 97.37%: [raw](../assets/all_open_prs_20261116/claim-page-evidence/9137/textbox_stored_top_tb_v0_pushed-hwp/wasm/raw-001.png) · [review](../assets/all_open_prs_20261116/claim-page-evidence/9137/textbox_stored_top_tb_v0_pushed-hwp/wasm/review/review_001.png) · [overlay](../assets/all_open_prs_20261116/claim-page-evidence/9137/textbox_stored_top_tb_v0_pushed-hwp/wasm/overlay/overlay_001.png).

`_pushed`의 A1·A2는 한 줄 피치 낮은 차이가 남는다. BOX·BODY의 공용 점유·정렬 개선과
이 공개 잔여를 구분하며 모든 위치가 일치했다고 기록하지 않는다.

## 98% 미만 대조군의 회귀 판정 보정

밀린 앵커 원본의 Native/fresh WASM 최저는 오프셋 0에서 97.48/97.37%, 2000HU에서
97.40/97.30%다. 기존 검사가 고정한 77.8/104.3px 거리를 그대로 유지하지 않고 같은 함수 안에서
원본의 이전 저장 줄 끝 + 표 세로 오프셋 + 바깥 위 여백을 실제 표의 내용 프레임 상대 원점과
비교하도록 교체했다. T1→T2→표/BOX→A1→A2의 순서·포함도 함께 확인한다.
이는 측정 좌표 핀이 아니라 원본 단위의 소비 관계다. 제품 코드·PDF·시각 기준은 바꾸지 않았다.
이 검사 변경의 집중 실행과 최종 최소 글꼴 전체 회귀·lint 결과를 확인한 뒤 최종 판정을 갱신한다.

보정 SHA는 `e4ca059525`(제한 프레임), `167921b3c5`(본문 커서 공유), `af0a5b9519`(기존 함수의 관계 검사)다.
97%대 `_pushed`는 고정 px 핀 대신 저장 줄 끝·원본 오프셋·표 바깥 여백과 실제 포함·순서를 보호한다.

## 통합 PR #9143의 code head CI

[통합 PR #9143](https://github.com/edwardkim/rhwp/pull/9143)의 code head
`8fdcff552506c7d4a51edf6fcbebe2d63a1a729a`에서 [CI](https://github.com/edwardkim/rhwp/actions/runs/43948529913),
CodeQL·Render Diff·Proptest·Adapter·CI 정책 판정이 모두 완료·성공했다.
원격 기본 회귀 다섯 archive는 **13,511 PASS·0 FAIL·45 skip**이며 로컬 최소 글꼴 macOS의
13,696 PASS와 실행 환경·선택 범위를 구분한다. Native Skia와 frontend package gate도 성공했다.
[정확한 workflow·job·head와 실제 summary](../assets/all_open_prs_20261116/code-ci-final.json)를 보존했다.
문서 전용 trailing의 최신 required check·mergeability를 확인한 뒤 통합하며, 아직 merge나 원 PR 종료가
완료됐다고 기록하지 않는다. contributor 원 head·fork branch는 보존한다.
