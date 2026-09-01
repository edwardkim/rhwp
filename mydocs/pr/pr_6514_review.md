# PR #6514 검토 - fit-test letter spacing trim 계약

- 검토일: 2026-09-01
- 작성자: `planet6897`
- 기준: `upstream/devel@336c4526e`
- 원 PR head: `b643b3822edccaa234133fc4cf2701910b090b8f`
- 원 적용 commit: `c8708e2d8`
- 메인터너 보정 commit: `ad877288b`
- 상태: 통합 candidate 수용 가능 — 특성화 범위 제한

## 범위와 보정

원 PR은 line-breaking fit-test의 trailing letter-spacing trim을 typed `FitWidthHwp`로 구속했다.
메인터너 보정에서는 `#[doc(hidden)] pub` test helper를 제거하고, exact test font를 등록한
`DocumentCore` 공개 조판 경로에서 양수·음수 자간의 실제 첫 줄을 관찰하도록 integration case를
바꿨다.

누적 candidate에서 두 경우 모두 첫 줄은 `가나다라`다.

- 양수 자간 `20`, content width `4,500 HWPUNIT`
- 음수 자간 `-20`, content width `3,950 HWPUNIT`

## 판정 경계

이 검사는 자간 부호에 따라 마지막 후보의 trailing spacing을 trim하는 현재 줄 나눔 계약을 고정한다.
한컴 정답지의 glyph ink 경계와 per-character allocation이 맞다는 correctness oracle은 아니다.
해당 잔여 과제는 #5678에 남기며 이 통합만으로 이슈를 닫지 않는다.

## 누적 검증

- `issue_5678_fit_test_letter_spacing_trim`: 2/2 통과
- SVG snapshot 두 건과 #6543 font chain을 함께 실행해 첫 줄 계약 무회귀 확인
- 전체 nextest: 8,914 passed, 46 skipped, 실패 0
- Rust lint 묶음, unit-tier, Native Skia, Docker WASM 통과

## 결론

공개 test-only API 문제는 제거됐고 특성화의 주장 범위도 코드 주석과 검토 기록에서 일치한다.
따라서 #6541 통합 candidate에는 수용하되, #5678의 잉크 오라클 완료로 확대 해석하지 않는다.
