# PR #8442 리뷰 — fix(edit): 서식 JSON의 문자열 밖 공백을 걷어내 읽고, 읽은 키가 없는 --props는 거부한다 (#8439)

## 최종 판정

**메인터너 보정 후 수용 가능**
메인터너가 통합 후보의 실제 전쪽 증적과 회귀 근거를 보강했습니다. 원 PR에 대응하는 입력·쪽과 반례는 아래에 기록했습니다.
원 head와 리베이스·메인터너 보정을 적용한 통합 후보를 구분합니다. 실제 적용 커밋은 아래 표에 기록했습니다.
완료된 검증과 남은 공통 검증을 아래에서 구분합니다. 최종 수용 판정과 PR 생성은 아직 대기입니다.
아래 값은 명시한 코드 SHA의 실측 결과이며 원 PR의 승인이나 닫기를 뜻하지 않습니다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR | [#8442](https://github.com/edwardkim/rhwp/pull/8442) · `planet6897` → `devel` |
| 원 head | `9b7af377c4e293c49c4cbbe776fa094563ffe6b9` |
| 통합 적용 커밋 | `93810dbeb1967e3f35fe4d50ed8085c942fbaf6b` |
| 이전 검토 기준 devel | `22947f6a90c998e95fa51238d5b907a0ebd37162` |
| 이전 필수 검증 제품 코드 | `a7e5ce7112ea8b227c317a9cbfb9d6fed0b00973` |
| 전쪽 시각 측정 producer / 당시 base | `4b149b6cdfc514b9858ed644be2cf8120c731cd4` / `7fdd5441ff1e77c083818e920a48089f294de35e` |
| reviewer | `jangster77` 요청. 2026-11-04T17:44:08.556Z 원 head 불변·Open·non-Draft·요청 상태 재확인 |
| 기본 경로 | `maintainer_general` |
| 보조 경로 | `intake_and_review`, `local_validation`, `multi_pr_update_branch`, `post_merge` |
| 관련 이슈 | [#8439](https://github.com/edwardkim/rhwp/issues/8439) |

## 변경과 검토 범위

원 PR의 변경 주장은 **fix(edit): 서식 JSON의 문자열 밖 공백을 걷어내 읽고, 읽은 키가 없는 --props는 거부한다 (#8439)**입니다. 해당 주장에 연결된 입력과 쪽을 아래에 분리해 적었습니다.
적층 PR에 중복 포함된 선행 커밋과 이 PR의 실제 신규 커밋은 최종 통합 기록에서 구분합니다.

## 검증 입력과 전쪽 결과

이 PR은 위 원본 대조 표의 직접 대상이 아닙니다. 해당 변경의 독립 계약 검사와 입력 생성 결과는 최종 검토에 연결합니다.

## 조판 원칙 준수 검토

조판 구현 변경은 비해당입니다. 서식 JSON의 해석·명령 적용·거부 계약을 바꾸며 줄 구성·배치 알고리즘은 변경하지 않습니다.

## 구현 주장과 검증 증거의 대조

서식 JSON의 문자열 밖 공백을 처리하면서 문자열 내부 공백·escape를 보존하고, 인식한 서식 키가 없는 --props는 오류로 거부한다.

document_core/helpers.rs의 공통 JSON 읽기 경로와 CLI의 본문·셀·머리말/꼬리말·각주 서식 명령이 수정 결과와 빈 변경 여부를 함께 확인한다.

공백이 있는 값·배열·중첩 객체와 빈 객체·알 수 없는 키를 CLI 및 DocumentCore 검사에서 대조한다. 조판 알고리즘 변경은 없으므로 이 PR의 입증 대상은 명령의 적용·거부 계약이다.

관련 정식 회귀 파일: [issue_8439_props_json_whitespace.rs](../../tests/cases/issue_8439_props_json_whitespace.rs)

## 검증 입력 커밋 확인

이번 통합의 [입력·PDF SHA-256 대조표](assets/planet6897_20261103/input-integrity.tsv)에 있는
148개 파일은 증적 고정 커밋 `fd97ae040f`의 파일 내용과 일치합니다.
위 개별 입력은 저장소의 동일 파일을 재사용합니다.

## 원 PR 주장 쪽의 시각 증적

JSON 해석과 명령의 적용·거부 계약이 대상입니다. 해당 명령을 직접 실행하는
정식 회귀의 결과와 최종 전체 회귀 로그를 검증 근거로 연결합니다.

## Merge 후 원 PR 후속 처리

통합 PR 생성·병합과 원 PR 댓글·닫기는 아직 수행하지 않았습니다.
실제 병합 뒤 원 PR의 주장 문서·페이지에 대응하는 증거와 merge SHA를 연결합니다.

<!-- planet6897-20261103-generated-review -->

## 최신 리베이스 이후 통합 검증 상태

최신 base는 `f529dc92d4d172bf32390d7749743a45d9e41bfc`, 검증 시작 source는
`ba2ebfd34a4ebf402bce8e002254b672a09ef764`다. 원 PR 39개와 메인터너 보정을 담은
168개 커밋을 충돌 0건으로 리베이스했고 range-diff 전 행이 `=`로 대응했다.
[원 head와 현재 적용 SHA](assets/planet6897_20261103/integration-commits.md)를 구분한다.

최신 source의 최소 글꼴 Rust·Native Skia·lint·최적화 WASM 및 영향 경계 검증은 진행 중이다.
완료 판정이나 최종 승인으로 표시하지 않는다. 검증 중 소스 해시를 고정하고 문서 갱신과 구분한다.
충돌 없는 리베이스 뒤 기존 전쪽 비교를 반복하지 않는 작업지시자의 범위를 따른다.
새 base 기능은 같은 시스템 글꼴 환경의 실제 base/candidate 출력과 영향 경계로 대조한다.
새 base 대비 허용 하락은 0.1%p이며, 보호 PR #8389·#8397은 원 TSV의 소수 둘째 자리
표시값 하락을 허용하지 않는다. 기존 TSV·PNG의 producer는 원래 출처대로 유지한다.

직전 c183 제품에서는 [최소 글꼴 Rust 12,563 PASS와 필수 40개 실행](assets/planet6897_20261103/rust-validation-c183a1ac61.md),
[Native 80입력·406쪽](assets/planet6897_20261103/current-c183a1ac61/native-validation.md),
[fresh WASM 80입력·406쪽](assets/planet6897_20261103/current-c183a1ac61/wasm-validation.md),
[실제 브라우저/API 실행](assets/planet6897_20261103/runtime-validation-c183a1ac61.md)을 완료했다.
양쪽 전쪽 최저는 각각 92.45232%·92.87288%다. 이 결과를 최신 리베이스 제품의 재출력으로
취급하지 않는다. 보호 범위 Native는 17문서·1,770쪽·원 TSV 2,153행의 하락·누락 0행이며,
같은 c183 WASM 보호 범위의 남은 집계를 진행 중이다. [공식 증거 목록](assets/planet6897_20261103/README.md)과
[재실행 명령·입력·Print PDF·조판 세대·글꼴 계약](assets/planet6897_20261103/visual-reproduction.md)을 연결한다.

원 PR별 주장 쪽의 직접 판독·회귀 주석·최종 본문 감사를 마친 뒤 수용 판정을 확정한다.
통합 PR 생성·push·원격 review·comment·close는 아직 수행하지 않았다. PR 생성 전 승인을 기다린다.
