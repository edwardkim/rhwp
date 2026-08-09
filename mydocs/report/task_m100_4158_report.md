# 최종 보고서 — task_m100_4158

- **Issue**: [#4158](https://github.com/edwardkim/rhwp/issues/4158)
- **브랜치**: `task_m100_4158_char_overlap_boxed_pua`
- **최초 기준**: `upstream/devel` `5119ea498`
- **현재 기준**: `upstream/devel` `5a4f26d0d`, merge commit `5356207db`
- **계획서**: [`mydocs/plans/task_m100_4158.md`](../plans/task_m100_4158.md)
- **단계 기록**: [`mydocs/working/task_m100_4158_stage1.md`](../working/task_m100_4158_stage1.md)
- **작성 시각**: 2026-08-07 KST

## 1. 결과

실제 `CharOverlap`의 `U+F02B1` 사각 숫자가 브라우저 글꼴의 PUA glyph에 의존해 tofu로
출력되던 결함을 고쳤다. IR 원문은 보존하면서 Canvas2D·SVG·Native Skia가 한 공통 규칙으로
사각형과 숫자를 합성한다.

실제 HWP 물리 10쪽의 `공정거래위원회` 앞 표식은 수정 후 정답지와 같은 사각형 안 숫자 1로
출력된다. 문서는 17쪽을 유지하고, #4139가 고정한 물리 2쪽 일반 `TextRun` 경로도 통과했다.

## 2. 수정 계약

```
single CharOverlap U+F02B1..U+F02C4 → number 1..20
raw border 0                            → effective square border 3
explicit border 1..4                    → preserve
IR text / CharOverlapInfo               → preserve
```

다중 문자 PUA 숫자, 표준 Unicode 원문자와 범위 밖 PUA 동작은 바꾸지 않았다.

## 3. 검증 요약

- focused Rust 3건 PASS
- Native Skia feature release-test focused 2건 PASS
- `clippy --lib`, fmt, diff PASS
- release WASM build의 compile·wasm-bindgen·wasm-opt·packaging PASS
- 신규 물리 10쪽 Canvas2D E2E 7개 계약 PASS
- 기존 물리 2쪽 #536 E2E 6개 계약 PASS
- 실제 SVG에서 `<rect>`+숫자 1과 raw PUA 부재 확인
- 17쪽 한컴 PDF 물리 10쪽과 시각 대조 완료
- release build와 library 3,292 passed / 10 ignored / 0 failed
- `release-test --tests` 전체 PASS, Native Skia 공식 3종 58 + 2 + 4 PASS
- 전체 타깃 Clippy, fmt, diff, doc test, Studio TypeScript PASS
- CI 동등 Node 22 Studio test 765 passed / 0 failed
- 새 WASM으로 #4158 7개와 기존 #536 6개 브라우저 계약 재통과
- E2E manifest tracked 86개 / 86행 일치

증적은 `output/4158/`에 있다. 전체 PR-CI형 로컬 게이트까지 작업지시자 승인 아래 완료했다.

샌드박스 내부에서는 Node `spawnSync` 자식 stdout 캡처가 비어 마커 기반 5개가 실패했으나, 같은
Node 22 전체 명령을 샌드박스 밖에서 실행하면 765개가 모두 통과했다. 이는 제품 코드 실패와
분리해 기록한다.

## 4. 2026-08-08 current-head 재검증

이미 병합된 #4159를 포함한 최신 `devel`을 작업 브랜치에 통합했다. #4158·#4159의 오늘할일과
E2E manifest 항목을 모두 보존했고, 현재 head에서 focused Rust 3건, Native Skia 2건, release
WASM build, #4158 7개·#536 6개·#4159 2개 브라우저 계약, manifest 87/87, fmt·diff 검사를
통과했다. 새 crop에서도 물리 10쪽 표식은 사각형 안 숫자 1이다.

2026-08-07의 전체 PR 게이트 결과는 당시 code head의 근거다. 최신 merge commit이 포함된 현재
head의 전체 게이트는 집중 재검증 결과 보고 뒤 별도 승인을 받아 실행한다.

작업지시자의 rhwp-studio 시각 판정도 통과했다. 이어서 발견된 일반 `TextRun`의
`U+F02FB` tofu는 #4158의 실제 `CharOverlap` 사각 숫자와 다른 결함이므로 별도 처리한다.

## 5. 원격 상태

로컬 단계 커밋까지만 수행한다. GitHub push, PR 생성, #4158 comment·close는 수행하지 않았다.
