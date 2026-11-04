# PR #8389 검토 기록

- 코드 head: `d84569dfc3ce05869e15f4fa67f54e4108a918fa`
- PR base: `2d535febe00a61585b6770cb95860fee912bbe54`
- 관련 이슈: #8153 (130쪽 WMF 차이 예외; 이슈 완료 판정 별도)
- merge: `c417310cab62bd7dcf8ef131f72001bb4a1fd3d9` (2026-10-30).
- [PR #8389](https://github.com/edwardkim/rhwp/pull/8389)

## 변경과 범위

편람 HWP·HWPX의 저장 조판, 표·그림·문자 출력 공통 경로를 보정했다. #7905의 1쪽을 함께 보정했다. #7905의 3~6쪽은 기존 90% 미만이며 사용자는 이번 범위를 1쪽으로 한정했다.

## 검증 근거와 한계

리베이스 전 후보 `01f66c18a63f27cad85fae1367f8c877060a872c`에서는 편람 HWP·HWPX 각 383쪽 중 130쪽의 raw 표시값이 99.96%, 나머지는 100.00%였다. 130쪽 WMF의 차이는 사용자 승인 예외다. 현 code head에서 편람 HWP·HWPX의 Native와 fresh WASM 전쪽을 재측정했고 네 TSV는 모두 이전과 바이트 단위로 동일했다. 그 외 문서의 리베이스 후 전쪽 비교와 전체 회귀는 실행하지 않았다.

입력별 Native·fresh WASM TSV와 이전 기준, PNG, 입력·PDF 해시는 원 PR 본문과 `mydocs/pr/assets/issue_8153/final-50cdf-validation-20261030/README.md`에 있다. 시스템 글꼴 디렉터리를 격리한 95건은 리베이스 전 검증에서 통과했다. 새 폰트 파일은 추가하지 않았다.

전체 nextest 선행 실행은 12,174건 통과·1건 실패였고 수정된 #4969 모듈 11건을 따로 통과했다. release lib는 3,790 통과·11 실패·12 ignored였으며 출력 디렉터리 보정 후 동일 바이너리에서 실패 11건을 각각 재실행해 통과했다. 전체 검사를 최신 head에서 다시 실행한 것으로 표시하지 않는다.

## CI 브라우저 보정

#8160 로고 기준 PNG의 이전 정수 버퍼 맞춤과 현재 96dpi 물리 배율 차이를 같은 독립 PDF로 확인했다. 검사 허용치와 제품 코드는 바꾸지 않았으며 최적화 fresh WASM 브라우저 9건과 Visual Sweep 정책 201건이 통과했다. 근거는 `mydocs/plans/issue_8160_jpeg_exact_dpi_20261030.md`에 있다.

## CI 및 merge

최종 head `d84569dfc3ce05869e15f4fa67f54e4108a918fa`의 [CI](https://github.com/edwardkim/rhwp/actions/runs/41038362573), [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/41038362585), [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/41038362523), [Adapter](https://github.com/edwardkim/rhwp/actions/runs/41038362603), [Proptest](https://github.com/edwardkim/rhwp/actions/runs/41038362519)가 성공했다. CI의 첫 실행에서 lint의 fetch가 정체되어 실패한 작업을 재실행했고 두 번째 시도에서 통과했다. Merge 가능 상태와 정확한 head를 확인한 뒤 squash merge했다.
