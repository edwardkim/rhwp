# PR #8442 리뷰 — fix(edit): 서식 JSON의 문자열 밖 공백을 걷어내 읽고, 읽은 키가 없는 --props는 거부한다 (#8439)

## 최종 판정

**메인터너 보정 후 수용 가능**
메인터너가 통합 후보의 실제 전쪽 증적과 회귀 근거를 보강했습니다. 원 PR에 대응하는 입력·쪽과 반례는 아래에 기록했습니다.
원 head와 리베이스·메인터너 보정을 적용한 통합 후보를 구분합니다. 실제 적용 커밋은 아래 표에 기록했습니다.
제출 전 필수 검증의 완료 근거를 아래에 기록했습니다. 작업지시자는 준비 완료 후 PR 생성·CI 모니터링을 승인했습니다.
아래 값은 명시한 코드 SHA의 실측 결과이며 원 PR의 승인이나 닫기를 뜻하지 않습니다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR | [#8442](https://github.com/edwardkim/rhwp/pull/8442) · `planet6897` → `devel` |
| 원 head | `9b7af377c4e293c49c4cbbe776fa094563ffe6b9` |
| 현재 통합 적용 커밋 | `cb842a649278b233ec6fc986fe2698bccb265bae` |
| 이전 검토 기준 devel | `22947f6a90c998e95fa51238d5b907a0ebd37162` |
| 이전 필수 검증 제품 코드 | `a7e5ce7112ea8b227c317a9cbfb9d6fed0b00973` |
| 초기 대표 PNG producer / 당시 base | `4b149b6cdfc514b9858ed644be2cf8120c731cd4` / `7fdd5441ff1e77c083818e920a48089f294de35e` |
| reviewer | `jangster77` 요청. `2026-11-05 03:16:02 UTC` 원격 재조회에서 원 head 불변·Open·non-Draft 확인 |
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
148개 실제 검증 파일과 현재 Git 파일의 바이트가 일치함을 다시 확인했습니다.
위 개별 입력은 저장소의 동일 파일을 재사용합니다.

## 원 PR 주장 쪽의 시각 증적

JSON 해석과 명령의 적용·거부 계약이 대상입니다. 해당 명령을 직접 실행하는
정식 회귀의 결과와 최종 전체 회귀 로그를 검증 근거로 연결합니다.

## Merge 후 원 PR 후속 처리

이 문서 고정 시점에는 통합 PR 생성·병합과 원 PR 댓글·닫기를 수행하지 않았습니다.
실제 병합 뒤 원 PR의 주장 문서·페이지에 대응하는 증거와 merge SHA를 연결합니다.

<!-- planet6897-20261103-generated-review -->

## 최신 통합 검증 상태

현재 base는 `ed057276c1e0e07e743218d95641e015b46b6715`입니다. 이 갱신은 후속 문서 14개뿐이고,
183개 패치가 충돌 0건·range-diff 모두 `=`로 리베이스됐습니다. 제품·테스트·fixture·검증 도구의 내용은
리베이스 전과 같으므로 완료 증거를 재사용합니다. 실제 측정 source는
`ba2ebfd34a4ebf402bce8e002254b672a09ef764`이며 문서 commit을 새 렌더 제품으로 표시하지 않습니다.
[현재 적용 SHA 39개](assets/planet6897_20261103/integration-commits.md)를 확인합니다.

- [최소 글꼴 환경의 필수 실행 41개](assets/planet6897_20261103/rebase8609/rust-regression-validation.md)는 모두 exit 0입니다.
  전체 nextest 12,583 PASS·45 SKIP, Native Skia lib 4,040 PASS·12 IGNORE,
  CI Native 모듈 21개와 doctest 8 PASS·3 IGNORE를 포함합니다.
- [최신 Native 영향 10입력·43쪽](assets/planet6897_20261103/rebase8609/native-own-validation.md)과
  [fresh WASM 같은 43쪽](assets/planet6897_20261103/rebase8609/wasm-own-validation.md)은 전쪽 90% 이상,
  직전 후보 대비 정밀 raw 0.1%p 초과 하락·누락·쪽수 불일치 0건입니다.
- [최신 필수 편람 Native 383쪽](assets/planet6897_20261103/rebase8609/handbook-native-validation.md)은
  최저 표시 raw 99.96%, 기준 표시값 하락·fallback 0건·exit 0입니다.
- [최신 실제 API·브라우저](assets/planet6897_20261103/rebase8609/runtime-validation.md)는 실제
  271 assertion·17 Worker·12쪽 렌더/저장과 가져오기·편집 산출물 대조를 완료했습니다.
- 기존 c183의 [Native 80입력·406쪽](assets/planet6897_20261103/current-c183a1ac61/native-validation.md)과
  [fresh WASM 전406쪽](assets/planet6897_20261103/current-c183a1ac61/wasm-validation.md)은 양쪽 90% 이상입니다.
  충돌 없는 리베이스 뒤 전쪽 측정을 반복하지 않는 지시에 따라 원래 producer를 명시해 재사용합니다.
- 보호 PR #8389·#8397의 [c183 Native](assets/planet6897_20261103/current-c183a1ac61/pr8389_8397/native-validation.md)와
  [c183 fresh WASM](assets/planet6897_20261103/current-c183a1ac61/pr8389_8397/wasm-validation.md)은
  각 17문서·1,770쪽·원 TSV 2,153행에서 표시값 하락·누락 0행입니다.

원 PR별 주장에 대응하는 입력·쪽의 대표 review/overlay는 아래 원래 출처를 유지합니다.
최신 영향 입력의 그림은 최신 source로 별도 표시하며, 다른 문서의 대표 그림으로 대체하지 않습니다.
[직접 판독 원장](assets/planet6897_20261103/direct-reading-current.md)과
[입력/PDF·compat·글꼴·명령](assets/planet6897_20261103/visual-reproduction.md)을 연결합니다.
회귀 측정 주석 갱신은 실제 assertion·helper 본문과 구분했고 주석만 바뀐 전체 검사는 반복하지 않았습니다.

[새 base Native 대조](assets/planet6897_20261103/rebase8609/native-base-preservation.md)와
[fresh WASM 대조](assets/planet6897_20261103/rebase8609/wasm-base-preservation.md)는 각각 14입력·41쪽을 완료했습니다.
양쪽 모두 raw 하락 자체가 0쪽이며 90% 미만·누락·쪽수 차이·제품 해시 오류 0건입니다.
문서-only 리베이스의 [내용 동일성](assets/planet6897_20261103/latest-rebase.md)을 확인해 완료 결과를 재사용했습니다.
추가 전체 회귀는 필요하지 않습니다. 제출 전 필수 검증은 완료했고 통합 PR 생성 후 정확한 head의 CI를 모니터링합니다.
원 PR 39개는 원격 재조회에서 head 불변·Open·non-Draft·reviewer 요청을 확인했습니다.
본인 계정의 우선 처리할 열린 PR은 0건입니다. 이 문서 고정 시점에는 원격 push·PR 생성·review·comment·close·merge를 수행하지 않았습니다.
작업지시자는 준비 완료 후 PR 생성·CI 모니터링을 승인했습니다. 최종 본문·링크·증적 감사를 고정한 뒤 제출합니다.
