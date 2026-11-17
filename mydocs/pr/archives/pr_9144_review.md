# PR #9144 리뷰 — 합성 줄을 저장 높이 근거에서 제외

## 최종 판정

**로컬 검토 충족 — 최신 통합 PR CI·후속 기록 확인 후 병합 가능.** 원 기여와 필요한 메인터너 보정을 구분해 검증했다.

## 변경과 검토 범위

작성자는 `signspark`, 원 head는 `5cffd27772756f21796569060de9bc7132a9a1e1`이다.
관련 이슈는 [#9142](https://github.com/edwardkim/rhwp/issues/9142)다. 첫 기여 이력을 확인했다.
기여자의 합성 LineSeg bit 31 판별은 `dump-pages`의 저장 높이 추정에 적용되며
조판·paint를 바꾸는 값이 아니다. 직접 호출처와 실제 CLI를 함께 확인했다.

## 실제 저장·재열기 진단

기존 빈 HWP에 CLI로 2×2 표를 삽입·저장한 같은 바이트를 base와 후보에서 다시 열었다.
양쪽 쪽수는 1이다. base는 합성 줄로 `hwpUsedHeight=42.6666666667`,
`usedDiff=20.4`를 보고했지만 후보는 두 값을 `null`로 보고한다.
실제 저장 줄 대조는 원 PR 회귀에서 유지한다. 진단 입력은
`samples/issue9142/inserted-table.hwp`, SHA는
`64c1f2cf0e2c4adf49cfe51c330c876e1a931816b6f8e038e20ac07d1989f965`다.
[실제 CLI 기록](../assets/open_prs_20261117/diagnostic9144-actual-cli.json)을 보존한다.
진단용 null 계약을 PDF 일치율이나 시각 개선으로 표현하지 않는다.

## 접수와 통합 신원

리뷰어는 `jangster77`이며 지정 결과를 API에서 확인했다. 검토 브랜치는
`review/integration-20261117-open-prs`, base는 `1e2488ebd432deb7c326be532f8a91945f260231`,
현재 보정 source는 `2c5e3fe38726d23bec1c970ba3f92a49e15cae34`다. #9144 → #9149 → #9152의 11개 커밋을
원 SHA를 기록하며 체리픽했고 개별·쌍별·누적 시뮬레이션 9건은 충돌 없이 종료했다.
원 fork에는 push하지 않았다. [선행 분석](../assets/open_prs_20261117/intake-analysis.md)을 따른다.

## 최종 로컬 검증과 후속 조건

[통합 검증 정본](../assets/open_prs_20261117/README.md)과 각 원본 TSV·PNG를 확인했다.
최소 글꼴 환경 focused 7건과 전체 nextest **13,708 PASS / 0 FAIL / 45 skip**,
fmt·Native/WASM/all-target Clippy·workspace build·manifest/unit-tier, Native Skia **2·2·36건**을 통과했다.
원본 글꼴 일반 환경의 정본 보호 18종·1,771쪽은 Native/fresh WASM 모두 하락·상승 0쪽이다.
명시적 편람 Native 383쪽(최저 99.97%)과 기여자 공통 보호 8건도 통과했다.
[회귀 집계](../assets/open_prs_20261117/rust-validation.json),
[보호 Native](../assets/open_prs_20261117/protected-native-validation.json)·
[보호 WASM](../assets/open_prs_20261117/protected-wasm-validation.json),
[실제 소비 경로](../assets/open_prs_20261117/consumer-validation.json)를 따른다.

최종 통합 PR의 code CI·오늘할일 trailing head CI·병합과 원 PR/실제 해결 이슈 close는 아직 확정하지 않는다.
원 fork를 수정하지 않으며, 통합 병합 뒤 검토 결과와 확정 SHA를 원 PR에 안내한다.

## 후속 코멘트 계획

통합 병합과 CI 확인 후 첫 기여에 감사·환영을 전하고, 합성 줄 진단 수정과 확인한 실제 호출을
한국어 존댓말로 설명한다. 관련 없는 다른 PR의 시각 그림을 이 PR 증거로 사용하지 않는다.
