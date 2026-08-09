# 최종 보고서 — task_m100_4159

- **Issue**: [#4159](https://github.com/edwardkim/rhwp/issues/4159)
- **브랜치**: `task_m100_4159_nested_table_bottom_clip`
- **초기 기준**: `upstream/devel` `06f8ebcca`
- **PR 직전 통합 기준**: `upstream/devel` `23ff5b6f1` (merge commit `e6f09003b`)
- **계획서**: [`mydocs/plans/task_m100_4159.md`](../plans/task_m100_4159.md)
- **단계 기록**: [`mydocs/working/task_m100_4159_stage1.md`](../working/task_m100_4159_stage1.md)
- **작성 시각**: 2026-08-07 KST

## 1. 결과

실제 HWP 물리 3쪽의 종료 재귀 중첩 표 bottom 선이 조상 분할 셀 clip에 잘리던 결함을
고쳤다. 선 생성이나 backend paint를 바꾸지 않고, 마지막 유닛까지 소비한 terminal clip 셀만
재귀 Table 자손의 stroke 하단을 포섭하도록 bbox 계약을 정합시켰다.

수정 전 outer clip은 `824.880px`, bottom stroke는 `827.273px`였다. 수정 후 render tree와
SVG의 outer clip 하단은 `827.273px`로 stroke 하단과 일치한다.

## 2. 회귀 방지

- terminal/nonterminal 합성 unit 2개
- 실제 물리 3쪽의 모든 clip 조상 포섭 구조 래칫
- 실제 SVG clip과 bottom stroke 수치 래칫
- 물리 2쪽의 제2호·terminal border 조기 노출 금지
- #2007 기존 17쪽·cursor·저장 프레임·15/16쪽 자식 표 계약 유지
- 새 WASM Canvas2D에서 bottom 선 픽셀 1,196/1,203 검출

## 3. 검증 요약

- release build 및 library 3,305 passed / 10 ignored
- 전체 `release-test --tests` PASS
- Native Skia library 58 passed, 지정 integration 2 passed / 4 passed
- 전체 타깃 Clippy, fmt, diff, doc-test 4 passed / 2 ignored
- Studio TypeScript 및 전체 `npm test` 802 passed
- 새 release WASM compile·wasm-bindgen·wasm-opt·packaging PASS
- 신규 #4159 E2E와 기존 #536 E2E PASS
- E2E manifest tracked 86개 / 86행 일치
- 한컴 2020 물리 3쪽과 수정 후 Canvas2D 시각 대조 완료
- 작업지시자 rhwp-studio 시각 판정 PASS: 물리 2쪽 사각형 숫자 및 물리 3쪽 표 하단선 정상

증적은 `output/4159/`에 있다. 작업지시자 승인에 따른 `local_validation.md` 4.3 전체
PR-CI형 로컬 게이트까지 통과했다. 원격 `devel` 전진을 통합한 `e6f09003b`에서도 같은 전체
게이트와 새 WASM 브라우저 검증을 다시 통과했다.

## 4. 원격 상태

원본 저장소의 `task_m100_4159_nested_table_bottom_clip` branch를 push하고 `devel` 대상 Open PR
[#4174](https://github.com/edwardkim/rhwp/pull/4174)를 생성했다. #4159 comment·close와 PR merge는
수행하지 않았다.
