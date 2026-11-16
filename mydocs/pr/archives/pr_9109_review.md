# PR #9109 리뷰 — 저장 줄 없는 글상자의 자리차지 표 흐름

## 최종 판정

**승인.** 저장 줄 없는 글상자의 자리차지 표 흐름이라는 원 PR 범위를 같은 입력의 Native/fresh WASM 전쪽과 실제 소비 경로에서 확인했다. 통합 PR의 최신 CI와 mergeability 확인은 병합 전 조건이다.

## 접수 정보와 범위

- [원 PR](https://github.com/edwardkim/rhwp/pull/9109), 작성자 `hyunmyungJaneLee`.
- 원 head `6c73bc8c5b400429aee7f7a6328a3ae5c36d8da1`, base `53774cc1508591b6ac794950f0063452fbaf6e87`.
- reviewer `jangster77`, milestone `v1.0.0`을 API로 확인했다.
- #9108의 저장 줄 없는 글상자 안 문단 기준 TopBottom 표를 흐름과 정렬 높이에 반영한다.
- HWP/HWPX 8쌍은 같은 파일로 가정하지 않고 각 커밋 바이트와 각각의 Print를 확인한다.

## 검증과 조판 원칙

`samples/textbox_tb_table/`의 표·앞뒤 문단 소유, 세로 오프셋과 바깥 위 여백, 가운데/아래 정렬을
같은 입력의 base·후보 출력으로 대조한다. 같은 #9108의 저장 줄이 있는 후속 범위는 #9137과
구분하며 공유 커밋을 중복 적용하지 않았다.
측정 높이와 실제 paint의 원점이 같은 배치 계약을 소비하는지 확인한다.

원 PR의 **웹한글기안기 PDF는 공식 대조 자료로 사용하지 않는다.** 동일 입력을 실제
한컴 2020 Print로 출력하고 Native/fresh WASM 전쪽 TSV·내용·쪽수·직접 판독을 확인한다.
기존 px 검사의 적합성도 독립 Print의 시각 선행 조건과 대조했다.
정상 대조군·반복 저장과 실제 backend 소비도 해당 범위에서 확인했다.

## Merge 후 contributor PR comment 계획

통합 PR·merge SHA·CI와 실제 Print 제품을 한국어 존댓말로 알린다. 이 PR 주장인 저장 줄 없는
입력의 같은 쪽 raw·review·overlay를 본문에 표시하고 #9137의 이미지만으로 대체하지 않는다.
#9108은 두 PR의 실제 해결 범위를 합쳐 완전 해결 여부를 확인한 뒤 처리한다.


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

`textbox_tb_table_center_v2000_text-hwpx`, 1쪽.

- Native 100.00%: [raw](../assets/all_open_prs_20261116/claim-page-evidence/9109/textbox_tb_table_center_v2000_text-hwpx/native/raw-001.png) · [review](../assets/all_open_prs_20261116/claim-page-evidence/9109/textbox_tb_table_center_v2000_text-hwpx/native/review/review_001.png) · [overlay](../assets/all_open_prs_20261116/claim-page-evidence/9109/textbox_tb_table_center_v2000_text-hwpx/native/overlay/overlay_001.png).
- fresh WASM 100.00%: [raw](../assets/all_open_prs_20261116/claim-page-evidence/9109/textbox_tb_table_center_v2000_text-hwpx/wasm/raw-001.png) · [review](../assets/all_open_prs_20261116/claim-page-evidence/9109/textbox_tb_table_center_v2000_text-hwpx/wasm/review/review_001.png) · [overlay](../assets/all_open_prs_20261116/claim-page-evidence/9109/textbox_tb_table_center_v2000_text-hwpx/wasm/overlay/overlay_001.png).

`textbox_tb_table_top_v0_text-hwp`, 1쪽.

- Native 100.00%: [raw](../assets/all_open_prs_20261116/claim-page-evidence/9109/textbox_tb_table_top_v0_text-hwp/native/raw-001.png) · [review](../assets/all_open_prs_20261116/claim-page-evidence/9109/textbox_tb_table_top_v0_text-hwp/native/review/review_001.png) · [overlay](../assets/all_open_prs_20261116/claim-page-evidence/9109/textbox_tb_table_top_v0_text-hwp/native/overlay/overlay_001.png).
- fresh WASM 100.00%: [raw](../assets/all_open_prs_20261116/claim-page-evidence/9109/textbox_tb_table_top_v0_text-hwp/wasm/raw-001.png) · [review](../assets/all_open_prs_20261116/claim-page-evidence/9109/textbox_tb_table_top_v0_text-hwp/wasm/review/review_001.png) · [overlay](../assets/all_open_prs_20261116/claim-page-evidence/9109/textbox_tb_table_top_v0_text-hwp/wasm/overlay/overlay_001.png).

## 통합 PR #9143의 code head CI

[통합 PR #9143](https://github.com/edwardkim/rhwp/pull/9143)의 code head
`8fdcff552506c7d4a51edf6fcbebe2d63a1a729a`에서 [CI](https://github.com/edwardkim/rhwp/actions/runs/43948529913),
CodeQL·Render Diff·Proptest·Adapter·CI 정책 판정이 모두 완료·성공했다.
원격 기본 회귀 다섯 archive는 **13,511 PASS·0 FAIL·45 skip**이며 로컬 최소 글꼴 macOS의
13,696 PASS와 실행 환경·선택 범위를 구분한다. Native Skia와 frontend package gate도 성공했다.
[정확한 workflow·job·head와 실제 summary](../assets/all_open_prs_20261116/code-ci-final.json)를 보존했다.
문서 전용 trailing의 최신 required check·mergeability를 확인한 뒤 통합하며, 아직 merge나 원 PR 종료가
완료됐다고 기록하지 않는다. contributor 원 head·fork branch는 보존한다.
