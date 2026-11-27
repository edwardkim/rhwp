# PR #9281 구역 설정 재검토

메인테이너 보완 후 글자가 낀 첫 구역과 단 정의가 없는 문서 모두 A4·시작 번호 1로 인쇄됐다. 다음 구역까지 포함한 32쪽의 Native·fresh WASM 최저는 100.00%다. 2026-11-24 작업지시자가 기존 devel과 보완본의 PNG가 같은 WASM 보호 문서 14쪽에 한해 통합을 승인했다. [최종 출력과 저장 검증](../assets/semanticist21-20261124-rework/final/README.md)

## 기여자 재제출 시점의 검토

2026-11-24 메인테이너 판정은 **머지 보류**다. `semanticist21`이 보완한 원본에서는 첫 구역의 중복 바탕쪽이 사라졌다. 구역 정의와 단 정의 사이에 글자가 있는 문서에서는 용지와 시작 쪽 번호가 여전히 한컴 인쇄와 다르다. [보완 요청](https://github.com/edwardkim/rhwp/pull/9281#issuecomment-7089119750) · [기여자 답변](https://github.com/edwardkim/rhwp/pull/9281#issuecomment-7093796899)

| 비교 입력 | Windows 한컴 Print | 후보 Native · fresh WASM |
| --- | --- | --- |
| 두 구역 HWPX 원본, 짝수 시작 HWP, 한컴에서 다시 저장한 HWP·HWPX | 문서별 4쪽 | 16쪽 모두 양쪽 raw 100.00% |
| B5 첫 구역에서 구역 정의 바로 뒤에 단 정의 | 첫 두 쪽 B5 | 4쪽 모두 양쪽 raw 100.00% |
| 구역 정의와 단 정의 사이에 쪽 번호 위치 컨트롤 | 첫 두 쪽 A4 | 4쪽 모두 양쪽 raw 100.00% |
| 바로 붙여 둔 정의 사이에 `<hp:t>A</hp:t>` 하나 추가 | 첫 두 쪽 A4, 1·2쪽 번호 | B5, 5·6쪽 번호. 1쪽 54.00% · 54.31%, 2쪽 50.88% · 51.06% |

Windows 한컴 2020에서 마지막 두 문서를 각각 인쇄했다. 문자 대조군의 첫 두 쪽은 96 DPI에서 한컴 794 × 1122px, 후보 687 × 971px이다. 둘째 구역 두 쪽은 양쪽 raw 100.00%다. 해당 대조군 네 쪽의 최신 `devel`과 후보 PNG는 backend별로 같다. 이번 수정에서 새로 생긴 회귀로 집계하지 않았다. [원본과 인쇄 영수증](../assets/semanticist21-20261124-rework/inputs.tsv) · [용지·쪽 번호 비교](../assets/semanticist21-20261124-rework/section-text-pages.png) · [겹침 비교](../assets/semanticist21-20261124-rework/section-text-overlay.png)

`drop_first_section_def_like_hancom`은 첫 문단의 `controls`에서 두 컨트롤의 인덱스를 비교한다. 본문 글자는 이 배열에 들어가지 않아 첫 구역의 B5와 시작 번호 5가 그대로 적용된다. 새 head에서 XML의 실제 글자·컨트롤 순서를 구분하고, 인접·글자 개재·컨트롤 개재 및 둘째 구역의 결과를 위 원본으로 다시 확인하면 보류를 해제할 수 있다. #8482는 기여자가 별도 미해결 범위로 참조했다. [구역 판정 코드](../../../src/parser/hwpx/mod.rs) · [쪽별 결과](../assets/semanticist21-20261124-rework/pages.tsv)

원 head는 `43f85a99b5`에서 `f528069f3b`로 바뀌었다. 리뷰어는 `edwardkim`이며 원 head의 CI와 최신 `devel` 개별 병합 시뮬레이션이 통과했다. 실제 로컬 검증은 세 기여를 함께 적용한 `327be655a1`에서 수행했다. [공통 검증 기록](../assets/semanticist21-20261124-rework/README.md)

## 통합 PR #9376

원 PR의 head `f528069f3b38100e5928c1fb63bcfe1fba089d95`를 `980b84b40e`의 부모로 보존했다. 메인테이너 보완 커밋은 f997ce9409 · af7f716c37이다. 첫 구역을 A4·1쪽으로 시작할 조건, 다음 구역의 번호와 바탕쪽 보존을 독립 한컴 Print와 실제 저장·편집 경로에서 검증했다. Native와 fresh WASM의 해당 32쪽은 모두 100.00%였다. [통합 셀프 리뷰](pr_9376_review.md) · [원격 제품 CI](https://github.com/edwardkim/rhwp/actions/runs/45279878825)
