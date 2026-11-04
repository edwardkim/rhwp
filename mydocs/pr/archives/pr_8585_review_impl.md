# PR #8585 메인터너 보정 결과

[검토 기록](pr_8585_review.md)의 한 단위 로컬 보정·검증을 완료했다.
구현은 `a55c53a393`, 측정 근거·원장·증적을 포함한 후보는 `c3e28d2c6e`다.
원 head `7e7a86fef9`와 기여 commit 4개를 그대로 보존했다.

1. 본문과 글상자 여러 줄 개체가 가운데 정렬의 말미 공백 적용 조건과 기존 공통 공백 폭 helper를 공유하게 했다.
   개체 사이 공백·밑줄 공백·저장 줄/fresh 구분을 보존했고 좌표 상수를 더하지 않았다.
2. 원 head에서 실패하는 대응 열 관계를 확인하고, 보정 뒤 2열·3줄·포함·겹침·열 정렬을 같은 검사에서 보호했다.
   첫머리에 실제 Native/fresh WASM 측정과 이전 file commit을 기록하고 기존 suite 자동 수집 경로를 유지했다.
3. 독립 Print의 전 1쪽과 대응 사진 6개 보존을 확인하고 oracle 원장에 등록했다.
   추가 행의 위치를 조정해 최신 devel과 clean merge하고 기존 원장 값이 모두 보존됨을 검증했다.
4. 최소 글꼴 환경의 전체 회귀 12,426개와 모든 Rust lint·workspace build·Native 공식 3종을 통과했다.
   새 sample 보안 입력도 실제 전달했다. Native 99.95893%·fresh optimized WASM 99.96404%,
   정상 #8122 대조군은 Native 99.55124%·WASM 99.93233%로 base와 픽셀까지 같다.
5. 제목·마지막 사진 줄·후속 본문과 정상 글상자를 같은 배율로 직접 확대 판독했다.
   코드와 측정 주석만 다른 artifact 출처는 runtime source hash와 동등성 기록으로 구분했다.

로컬 범위와 통합 PR #8601의 동일 code head Full CI·최종 정책 감사를 완료했다. fork push 권한 403으로
작업지시자가 승인한 upstream Open PR 경로를 사용했다. 원 기여 SHA를 보존하며 self-review에서도 차단 사항이 없다.
최종 review·증적·오늘할일만 trailing commit으로 반영하고, 최신 head 검사·merge simulation 확인 뒤
승인된 merge commit·원 PR 간접 MERGED 확인·감사 댓글·이슈 완료·정리를 진행한다.
