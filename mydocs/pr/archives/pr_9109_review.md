# PR #9109 리뷰 — 저장 줄 없는 글상자의 자리차지 표 흐름

## 최종 판정

**머지 보류 — 원 PR의 오프셋·바깥 여백·세로 정렬 경계를 독립 Print로 검증 중이다.**

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
기존 px 검사의 적합성은 이 독립 Print의 시각 선행 조건을 확인한 뒤 판정한다.
실제 편집·반복 저장과 정상 대조군도 최종 검증에 포함한다.

## Merge 후 contributor PR comment 계획

통합 PR·merge SHA·CI와 실제 Print 제품을 한국어 존댓말로 알린다. 이 PR 주장인 저장 줄 없는
입력의 같은 쪽 raw·review·overlay를 본문에 표시하고 #9137의 이미지만으로 대체하지 않는다.
#9108은 두 PR의 실제 해결 범위를 합쳐 완전 해결 여부를 확인한 뒤 처리한다.


## 통합 후보 최종 소스의 중간 검증 결과

제품 소스 `167921b3c5`와 검증 head `21950b8ab8`에서 최소 글꼴 macOS 전체 회귀는
**13,696 PASS·0 FAIL·45 skip**이다. Native Skia·세 Clippy·workspace build·fmt·
suite 정책·unit tier·문서 테스트를 포함한 [Rust 13단계](../assets/all_open_prs_20261116/rust-gates-final.json)는 통과했다.
통합 검토의 공식 데스크톱 Print 66종·74쪽은
[Native](../assets/all_open_prs_20261116/native-claims-final.json)·
[fresh WASM](../assets/all_open_prs_20261116/wasm-claims-final.json) 각각 전쪽 90% 이상이다.
이 집계는 다른 PR의 입력을 이 PR만의 개선으로 계산한 값이 아니다. PR별 주장·반례는 위 접수 범위와
개별 보정 증거로 판정한다. 정본 보호 전쪽·실제 소비 경로와 최종 base/head 대조가 남아 있어
전체 PR 검토 완료나 머지 승인으로 기록하지 않는다.



## 원 PR 주장 페이지의 직접 판독

[입력·PDF 신원과 판독 기록](../assets/all_open_prs_20261116/claim-page-evidence/index.json)을 연결했다.

`textbox_tb_table_center_v2000_text-hwpx`, 1쪽.

- Native 100.00%: [raw](../assets/all_open_prs_20261116/claim-page-evidence/9109/textbox_tb_table_center_v2000_text-hwpx/native/raw-001.png) · [review](../assets/all_open_prs_20261116/claim-page-evidence/9109/textbox_tb_table_center_v2000_text-hwpx/native/review/review_001.png) · [overlay](../assets/all_open_prs_20261116/claim-page-evidence/9109/textbox_tb_table_center_v2000_text-hwpx/native/overlay/overlay_001.png).
- fresh WASM 100.00%: [raw](../assets/all_open_prs_20261116/claim-page-evidence/9109/textbox_tb_table_center_v2000_text-hwpx/wasm/raw-001.png) · [review](../assets/all_open_prs_20261116/claim-page-evidence/9109/textbox_tb_table_center_v2000_text-hwpx/wasm/review/review_001.png) · [overlay](../assets/all_open_prs_20261116/claim-page-evidence/9109/textbox_tb_table_center_v2000_text-hwpx/wasm/overlay/overlay_001.png).

`textbox_tb_table_top_v0_text-hwp`, 1쪽.

- Native 100.00%: [raw](../assets/all_open_prs_20261116/claim-page-evidence/9109/textbox_tb_table_top_v0_text-hwp/native/raw-001.png) · [review](../assets/all_open_prs_20261116/claim-page-evidence/9109/textbox_tb_table_top_v0_text-hwp/native/review/review_001.png) · [overlay](../assets/all_open_prs_20261116/claim-page-evidence/9109/textbox_tb_table_top_v0_text-hwp/native/overlay/overlay_001.png).
- fresh WASM 100.00%: [raw](../assets/all_open_prs_20261116/claim-page-evidence/9109/textbox_tb_table_top_v0_text-hwp/wasm/raw-001.png) · [review](../assets/all_open_prs_20261116/claim-page-evidence/9109/textbox_tb_table_top_v0_text-hwp/wasm/review/review_001.png) · [overlay](../assets/all_open_prs_20261116/claim-page-evidence/9109/textbox_tb_table_top_v0_text-hwp/wasm/overlay/overlay_001.png).
