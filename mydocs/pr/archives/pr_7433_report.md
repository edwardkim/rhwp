# PR #7433 사전 판정 보고서

## 수용 판단

**머지 보류.** [리뷰 기록](pr_7433_review.md)의 두 P2를 구분합니다: 0.5px 경계는 기존 결함이 남은 불완전한 수정, 페이지 예산 경계는 성장한 footprint의 새 본문 overflow입니다. 원본 두 사례의 개선과 녹색 CI는 확인했으나 승인·완료 판정으로 바꾸지 않습니다.

## 직접 보정과 수정 요청

현재 권고는 기여자의 branch에서 수정 후 **Request changes 재검토**입니다. 첫 번째는 가드 숫자만 넓히면 내용 하한 계약을 입증할 수 없고, 두 번째는 높이 생산·페이지 예약·fit·paint의 실제 소비를 함께 맞춰야 합니다. 수정 줄 수가 작다는 사실만으로 작은 보정이라고 판단하지 않습니다. 이번 기록에는 코드 보정이 없습니다.

collaborator가 직접 보정하는 것도 [직접 보정 규칙](../../manual/pr_review/collaborator_external_pr.md#931-contributor-pr-head-직접-보정)에 따라 가능합니다. 사용자가 해당 작업을 지시하면 maintainerCanModify와 원격 head를 다시 고정하고 같은 visibility branch의 contributor source 위에 code/regression commit을 분리합니다. 기여자 원 commit의 rebase·amend·force-push나 메인터너의 별도 integration PR로 우회하지 않습니다. 숫자 clamp·의도 없는 임계값 확대·baseline 완화로 수용하지 않습니다.

## 원격 조치 분리

2026-10-01 KST 사용자는 우선 검토 결과와 Visual Sweep 스크린샷 게시를 지시했습니다. 이 승인은 검토 증적을 원 PR source 위에 보존하고 일반 PR conversation comment를 게시하는 범위입니다. 정식 `REQUEST_CHANGES` 또는 `APPROVE`, 코드 보정·그 push, merge는 아직 실행하지 않았으며 별도로 확인할 조치입니다. 일반 코멘트 게시 뒤 URL·한글·본문·이미지 SHA를 API로 확인합니다.

- 문서/증적 source parent: `4905a31d0746fa86e23fe03655f4538079e2d0a1`.
- 원 기여: `52a51c0a97d0df7baab1e815ed820c6dff7ae2bb`, `4905a31d0746fa86e23fe03655f4538079e2d0a1`, author imsebeom. 두 commit·credit 보존.
- 로컬 current-base 검토 mapping: `52a51c0a…` → `804bbb955d9e8c2ca8cf90626b24454c1c3be65d`; `4905a31d…` → `e6503bcc315f9efd142e9d66e8a54d500668b578`. 이 검토 이력은 원격에 보내지 않고 보존 ref로 유지합니다.
- review-only 묶음은 archive 2문서·asset README·대표 PNG·실제 반례 입력 2개·새 기준 PDF 2개뿐입니다. 제품 source/test/workflow/Cargo.lock/sample/baseline은 변경하지 않습니다.
- source에 없는 20261001 오늘할일은 최신 devel에 다른 PR 기록이 있어 복사하거나 add/add 충돌을 만들지 않았습니다. 기존 orders 파일 전체를 병합 simulation에서 보존 확인합니다. 별도 기록 PR을 만들지 않습니다.
- push 전: source/base 재확인, single parent, merge-tree 충돌/공백/문서 링크/기존 orders 보존, blob 해시, LFS 사전 판독·dry-run 확인.
- push 후: 실제 새 head와 parent·changed paths·mergeability 확인. CI fast-pass를 예상할 수 있으나 실제 실행 결과와 별개이며, 녹색이 되어도 현재 보류 판정을 해소하지 않습니다.

## 재검토·병합 전 조건

1. 동일 내용·유효한 재조판 줄의 점유 하한이 arbitrary metadata 일치 경계에서 사라지지 않을 것.
2. 자란 표의 실제 높이를 예약/fit/배치가 공유하고, 선언만 fit하는 예산·이월·뒤 내용이 정식 case에서 검증될 것.
3. 새 head의 반례·정상 대조군 및 Native/fresh WASM Visual Sweep을 직접 확인할 것. 소유 컷·rowspan·종료 등 변경된 경로에 적용되는 경계도 확인할 것.
4. code/test 변경이면 최신 head의 필수 로컬 회귀·세 lint·정책 검사·Full CI와 관련 checks를 통과할 것. review-only fast-pass로 코드 검증을 생략하지 않을 것.
5. 보존한 대표 review/overlay를 정확한 head SHA에 고정하여 PR 본문에서 직접 표시하고 남은 약 4px 차이·미검증 범위를 정리할 것.
6. 해결 범위에 맞춰 PR의 `Closes #7419` 표현을 다시 판단하고, 별도 리뷰 및 merge 승인을 받을 것.

merge SHA·merge 시각·issue close 완료·merge 후 contributor comment는 아직 존재하지 않습니다. 승인 근거가 마련되면 같은 대표 asset과 실제 검증 결과를 사용한 merge SHA 고정 contributor comment 계획을 작성합니다. 미래 실행을 완료로 기록하지 않습니다.
