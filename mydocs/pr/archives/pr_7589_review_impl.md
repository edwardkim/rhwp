# PR #7589 구현·검증 기록

[개별 리뷰](pr_7589_review.md)의 실제 소비 경로와 원 커밋 대응을 따른다.
원 contributor head는 `300d1af385064eb32bb85c94f3a40ab7213d2d2a`, 검증 코드·테스트 source는
`04c92da78165ccbf63b467855591df183655a98d`, 고정 devel은 `faa885e8b54ffc7cdf4196cd43a3103f362499a2`다.

## 보정과 검증

원 PR은 같은 `FIRST\nSECOND`에서 MultiLine=1/0의 SVG가 달라야 한다고 주장한다.
파서·직렬화가 개행과 속성을 보존하는지 검사한 점은 유지한다. 그러나 그 뒤의
`assert_ne!`는 이 입력의 한컴 인쇄 동작을 입증하지 않는다.

사용자가 두 HWPX를 수정 없이 한컴의 파일 → 인쇄 → PDF로 출력했고,
win10-ted의 `C:\temp`에서 원본 바이트를 가져왔다. 두 PDF는 각각 A4 한 쪽이며
실제 Creator는 `Hwp 2022 12.0.0.4605`다. 파일명의 2020과 실제 제품을 구분한다.
두 Print의 래스터는 동일하고 입력 상자에는 `FIRST`만 표시된다.

공통 `FormObjectNode`의 표시 문자열에서 첫 저장 줄을 선택하되 모델 원문과
MultiLine 속성은 보존하도록 보정했다. SVG·Canvas·Native가 같은 표시 결과를
소비하며 특정 문서나 좌표에 따른 예외는 없다. 모든 입력 상자의 편집 중 화면까지
MultiLine이 무의미하다고 일반화하지 않는다.

공통 연산의 정상 대조군과 실패 경계를 개별 리뷰의 실제 검사에 연결한다. 이미 정확한
독립 기대값은 유지하며, 잘못된 고정 위치·과거 쪽수 핀은 원본·독립 Print에 따라 재검토한다.
실행 결과와 명령, 현재 입력의 전체 Native/fresh WASM TSV·직접 판독은
[최종 제출 증거](../assets/s21_20261027/README.md)의 해당 조건을 따른다.

## 현재 단계와 후속 처리

메인터너 보정 후 수용 가능 — 입력 상자의 저장 개행과 Print 표시 범위의 로컬 검증 완료.

통합 PR의 실제 code head CI·merge는 로컬 검증과 별개다. 최신 지시에 따라
code head CI 성공 뒤 같은 PR의 self-review·오늘할일 trailing commit을 추가하며, 완료 시 실제 PR·CI·merge SHA와
원 PR별 comment·close 결과를 기록한다. 번호나 merge 결과를 미리 만들지 않는다.
삭제한 sample16·편람의 일반 회귀와 파생 참조를 되살리지 않는다. 명시적으로 지정한
#8188 네 변환 입력의 기능 보존 검증은 별도 예외 범위로 구분한다.

## 통합 PR #8319의 code head CI 확정

이 PR의 원 contributor head와 위 개별 검증 범위를 유지한 통합 후보 `aee691ae596d288382369b9d5243f4cd06e7fa65`의
[최종 code CI](https://github.com/edwardkim/rhwp/actions/runs/40489874571)가 성공했다. [PR #8319](https://github.com/edwardkim/rhwp/pull/8319)의 문서-only trailing head CI와 정책 성공 뒤 일반 병합한다.
원 contributor PR 자체를 직접 merge한 것으로 표시하지 않는다. 제품 source 불변인 추가 테스트
보정과 최종 원격 검증은 [통합 리뷰](pr_8319_review.md)·[CI 증거](../assets/s21_20261027/final-ci.json)에 구분했다.
최신 지시대로 같은 PR에 self-review·오늘할일을 trailing commit으로 반영한다. 아직 병합하지 않았으며 실제 병합 뒤 원 PR별 결과 comment와 종료를 진행한다.
