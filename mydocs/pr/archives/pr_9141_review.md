# PR #9141 리뷰 — 누름틀 끝 뒤 컨트롤의 저장 줄 소속

## 최종 판정

**승인.** 한컴 저장 줄에서 FIELD_END 보조 슬롯 뒤 개체의 줄 소속과 변경 없는 두 형식 반복 저장을 확인했다. 값을 편집한 뒤 저장 줄 없는 TAC 표 재조판까지 해결한 판정은 아니다. 통합 PR의 최신 CI와 mergeability 확인은 병합 전 조건이다.

## 접수 정보와 범위

- [원 PR](https://github.com/edwardkim/rhwp/pull/9141), 작성자 `johndoekim`.
- 원 head `aea7b6a1fadac99257e7b302bf15b5afb3c38225`, base `53774cc1508591b6ac794950f0063452fbaf6e87`.
- reviewer `jangster77`, milestone `v1.0.0`을 API로 확인했다.
- #9058의 FIELD_END 보조 8유닛이 후행 컨트롤 앞에 속하는 경우를 공용 슬롯 계산으로 정리한다.
  관련 번호의 모든 양식을 이 세 원본만으로 완전 해결했다고 추정하지 않는다.

## 검증과 조판 원칙

기존 `mydocs/pr/assets/johndoekim-20261116/issue9058/`의 cursor/text 원본과
`samples/issue9082/textbox_filled_field_tac_tables.hwp`를 검토한다. 같은 입력에 대응하는
기존 한컴 2022 `Microsoft Print to PDF` 세 건은 새 사용자 지시에 따라 공식 기준에서 제외하고
참고 자료로 보존한다. 같은 원본을 현행 MCP의 한컴 2020 `Hancom PDF` Print로 새로 확보했다.
새 Print 세 건은 모두 MCP 수신을 완료했고 `Hancom PDF 1.3.0.550`·1쪽을 확인했다.
[66건 Print 영수증](../assets/all_open_prs_20261116/mcp-print-references.json)에 원본/PDF SHA와
실제 제품·job·출력 경로를 기록했다. 기존 Microsoft 기준의 측정값은 공식 결과로 재사용하지 않고
새 Print로 재측정했다.
저장 제품·직접 Print 제품·MCP engine을 혼동하지 않는다. 웹한글기안기 PDF도 공식 기준이 아니다.

한컴에서 저장한 빈 필드·4자 값·1자 값 원본에서 FIELD_END 앞뒤·후행 컨트롤·글자 없는 문단의
소속을 확인했다. HWP/HWPX 저장→재열기→재저장→재열기의 12회에서도 두 표의 위아래 순서와
글상자 안 포함, 필드 값과 1쪽을 보존했다. 인접 #9074·#9021·#8199 계약은 전체 회귀에 포함된다.
소스 UTF-16 위치와 저장 스트림 슬롯을 혼용하지 않고 같은 보조 슬롯 계산을 실제 composer와
fallback 배치가 소비하는지 대조한다. Native/fresh WASM 전쪽 TSV와 직접 판독도 완료했다.

## Merge 후 contributor PR comment 계획

최종 CI·통합 merge SHA·실제 편집/저장 결과와 세 원본의 같은 페이지 raw·review·overlay를
한국어 존댓말 코멘트에 표시한다. #9058의 원 이슈와 다른 하위 범위를 확인해 완전 해결된
범위만 종료하고, 이 PR을 통합한 사실과 남은 범위가 있으면 구분한다.


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

`field_end_slot_textbox_with_cursor-hwp`, 1쪽.

- Native 100.00%: [raw](../assets/all_open_prs_20261116/claim-page-evidence/9141/field_end_slot_textbox_with_cursor-hwp/native/raw-001.png) · [review](../assets/all_open_prs_20261116/claim-page-evidence/9141/field_end_slot_textbox_with_cursor-hwp/native/review/review_001.png) · [overlay](../assets/all_open_prs_20261116/claim-page-evidence/9141/field_end_slot_textbox_with_cursor-hwp/native/overlay/overlay_001.png).
- fresh WASM 100.00%: [raw](../assets/all_open_prs_20261116/claim-page-evidence/9141/field_end_slot_textbox_with_cursor-hwp/wasm/raw-001.png) · [review](../assets/all_open_prs_20261116/claim-page-evidence/9141/field_end_slot_textbox_with_cursor-hwp/wasm/review/review_001.png) · [overlay](../assets/all_open_prs_20261116/claim-page-evidence/9141/field_end_slot_textbox_with_cursor-hwp/wasm/overlay/overlay_001.png).

`field_end_slot_textbox_with_text-hwp`, 1쪽.

- Native 100.00%: [raw](../assets/all_open_prs_20261116/claim-page-evidence/9141/field_end_slot_textbox_with_text-hwp/native/raw-001.png) · [review](../assets/all_open_prs_20261116/claim-page-evidence/9141/field_end_slot_textbox_with_text-hwp/native/review/review_001.png) · [overlay](../assets/all_open_prs_20261116/claim-page-evidence/9141/field_end_slot_textbox_with_text-hwp/native/overlay/overlay_001.png).
- fresh WASM 100.00%: [raw](../assets/all_open_prs_20261116/claim-page-evidence/9141/field_end_slot_textbox_with_text-hwp/wasm/raw-001.png) · [review](../assets/all_open_prs_20261116/claim-page-evidence/9141/field_end_slot_textbox_with_text-hwp/wasm/review/review_001.png) · [overlay](../assets/all_open_prs_20261116/claim-page-evidence/9141/field_end_slot_textbox_with_text-hwp/wasm/overlay/overlay_001.png).

`textbox_filled_field_tac_tables-hwp`, 1쪽.

- Native 100.00%: [raw](../assets/all_open_prs_20261116/claim-page-evidence/9141/textbox_filled_field_tac_tables-hwp/native/raw-001.png) · [review](../assets/all_open_prs_20261116/claim-page-evidence/9141/textbox_filled_field_tac_tables-hwp/native/review/review_001.png) · [overlay](../assets/all_open_prs_20261116/claim-page-evidence/9141/textbox_filled_field_tac_tables-hwp/native/overlay/overlay_001.png).
- fresh WASM 100.00%: [raw](../assets/all_open_prs_20261116/claim-page-evidence/9141/textbox_filled_field_tac_tables-hwp/wasm/raw-001.png) · [review](../assets/all_open_prs_20261116/claim-page-evidence/9141/textbox_filled_field_tac_tables-hwp/wasm/review/review_001.png) · [overlay](../assets/all_open_prs_20261116/claim-page-evidence/9141/textbox_filled_field_tac_tables-hwp/wasm/overlay/overlay_001.png).

## 공개 저장 경로와 원 PR의 비범위 대조

제품 소스 `167921b3c5`의 fresh WASM 공개 API를 Node에서 직접 호출했다. 세 원본 × HWP/HWPX ×
재열기 2회 = **12회**의 변경 없는 저장은 필드 값·쪽수·두 표의 위치를 보존했다.
[base/후보 기록](../assets/all_open_prs_20261116/public-api-save-validation/field-save-scope-candidate.json)을
보존했다. 이는 Node의 구조 검사이며 Studio 화면·PDF 일치율 검증과 구분한다.

추가로 `setFieldValue`로 값을 바꿨을 때는 세 원본 모두 두 표가 한 줄로 붙었다.
[고정 base](../assets/all_open_prs_20261116/public-api-save-validation/field-save-scope-base.json)와
[후보](../assets/all_open_prs_20261116/public-api-save-validation/field-save-scope-candidate.json)에서
같이 재현했다. 원 PR의 「남은 범위」에도 값 설정 후 저장 줄 없는 글상자의 TAC 표 폭 줄바꿈은
별개 미해결 범위로 명시돼 있다. 이 결과를 편집 통과로 기록하지 않는다. 현재 검토의 수용 범위는
한컴이 저장한 줄에서 FIELD_END 보조 슬롯을 센 개체 소속 복구이며, 필드 편집 후 재조판까지
해결했다고 주장하거나 #9058의 다른 미해결 범위를 자동 종료하지 않는다.

## 통합 PR #9143의 code head CI

[통합 PR #9143](https://github.com/edwardkim/rhwp/pull/9143)의 code head
`8fdcff552506c7d4a51edf6fcbebe2d63a1a729a`에서 [CI](https://github.com/edwardkim/rhwp/actions/runs/43948529913),
CodeQL·Render Diff·Proptest·Adapter·CI 정책 판정이 모두 완료·성공했다.
원격 기본 회귀 다섯 archive는 **13,511 PASS·0 FAIL·45 skip**이며 로컬 최소 글꼴 macOS의
13,696 PASS와 실행 환경·선택 범위를 구분한다. Native Skia와 frontend package gate도 성공했다.
[정확한 workflow·job·head와 실제 summary](../assets/all_open_prs_20261116/code-ci-final.json)를 보존했다.
문서 전용 trailing의 최신 required check·mergeability를 확인한 뒤 통합하며, 아직 merge나 원 PR 종료가
완료됐다고 기록하지 않는다. contributor 원 head·fork branch는 보존한다.
