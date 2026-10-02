---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-25
---

# PR #7399 검토 — PrEP 쪽수 pin의 기준 설명

## 최종 판정

**승인.** 원 head는 `issue_2006_1790387_prep_pagination_pin.rs`의 주석만 고치고 기존 140쪽 assertion은 유지한다. 통합 code head `459d5e581eac979e8a3c69a72e71fb7cc3c9ea88`의 집중 pin 검사와 전체 release-test가 통과했다. 기준 PDF의 Producer만으로 생성 주체를 단정하지 않으며, 정본의 절차적 변환 출처는 미검증으로 남긴다. 원격 CI·mergeability는 통합 PR 생성 후 확인한다.

## 접수와 적용

| 항목 | 값 |
| --- | --- |
| 원 PR | [#7399](https://github.com/edwardkim/rhwp/pull/7399), `planet6897`, `devel` 대상 |
| 원 head / `-x` 체리픽 | `a8499bbbe5734309fb0fbdc74d6e57c7b95ce95d` / `72bf668bf91cb799a4bba5a97825c6aa421b16b2` |
| 메인터너 보정 | 없음 |
| 검토 base / code head | `b3e3d4e2170a43ca449e3d832440a9274e4e8ee4` / `459d5e581eac979e8a3c69a72e71fb7cc3c9ea88` |
| 원격 참고값 (2026-09-25) | OPEN, MERGEABLE/CLEAN, Draft 아님; 1파일, +29/−4 |

## 변경 범위와 검증

검사 입력 `samples/issue2006/1790387_prep_final_report.hwpx` SHA-256은 `c68baed24096386f9041930d24d39409b61ac99463bf04dfd242440dfdeb739f`, 기준 `pdf/issue2006/1790387_prep_final_report-hwp2020-20260814.pdf` SHA-256은 `f4b50edce8b519d8e1230657fe3630fec43e44327e27501887facfe632dc0c6a`이다. 주석은 Cairo Producer 표기와 KoPub 글꼴 설명을 정리하며 출력 환경 차이를 밝힌다. 주석이 설명하는 독립 근거와 실제 정본 생성 이력을 구분한다.

| 항목 | 판정·근거 |
| --- | --- |
| 140쪽 assertion 유지 | **충족**. 원 head의 실행 코드·기대 쪽수 불변, 통합 head 집중 `prep_1790387_page_count_pin` 1/1 PASS |
| 정본 생성 경로 | **미검증**. PDF Producer만으로 Hancom/rhwp 생성 주체를 입증하지 않음 |
| 조판·Visual Sweep | **비해당**. 이 PR은 주석만 변경하며 renderer 결과를 바꾸지 않음 |

통합 head의 release-test는 **10,229/10,229 PASS·50 skip**, 필수 fmt·Native/WASM/workspace Clippy·workspace build·manifest base 비교가 PASS였다. 과거 16건 묶음의 PrEP 69·70쪽 시각 실패는 #7406 등 별도 렌더 변경을 포함한 결과이며, 그 변경은 현재 8건 통합 후보에 들어 있지 않다. 이 주석 PR의 승인이나 시각 완료로 읽지 않는다.

## Merge 후 contributor PR comment 계획

실제 통합 merge 뒤 merge SHA·CI URL, 주석 근거의 수정과 140쪽 pin 유지, 정본 생성 경로 미검증 범위를 한국어로 알린다. 현재 comment·approve·push·merge는 수행하지 않았다.
