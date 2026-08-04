# PR #3136 검토 기록 — 브라우저 인쇄/PDF 저장 경로 안정화

## 메타와 통합 판단

| 항목 | 내용 |
| --- | --- |
| 원 PR | [#3136](https://github.com/edwardkim/rhwp/pull/3136) |
| 작성자 / 관련 이슈 | `postmelee` / [#3126](https://github.com/edwardkim/rhwp/issues/3126) (open 유지) |
| 원 PR 기준 | `devel` / head `c0dc1f6a1cd98c3331747aa08005ef2d7c437b0a` |
| 원 PR 최신 상태 | OPEN Draft, CONFLICTING/DIRTY, CI 없음, maintainer_can_modify=false (2026-07-24 확인) |
| 검토 브랜치 | `integrate/postmelee-20260724` |
| 통합 기준 / 순서 | `upstream/devel@1b5950a95` / 3/3 |
| Draft 처리 | 작업지시자의 명시 지시에 따라 Draft도 검토·통합 후보에 포함 |
| 처리 결론 | 원 PR direct merge는 불가하므로 메인터너 보정 통합 PR로 수용 권고 |

이 기록은 contributor가 이전에 남긴 self-review를 현재 통합 결과와 분리해 갱신한 것이다.
Windows Edge의 contributor 실증은 참고 근거이고, 아래 로컬 검증은 검토자가 실제 실행한 결과로 구분한다.

## 범위와 적용

#3126은 PDF bytes를 직접 생성하지 않고 브라우저 native print를 사용한다. PDF 저장 안내/생략 설정,
same-origin PDF iframe, 별도 인쇄 미리보기, `renderPageSvgWithProfile(page, 'print')`,
named `@page`, 파일명 보존을 구현한다. HWP/HWPX/HML의 SaveFormat PDF 확장은 범위 밖이다.

적용 SHA는 다음 13개다.

```text
c63abac 1028451 3536174 aafe29d bb75241 c87cdf9 6044550
f058343 c06a2f8 df30ade fc1f947 0e76a25 c0dc1f6
```

## 메인터너 충돌 조정

- 첫 문서 커밋의 오늘할일 add/add 충돌에서는 현재 #2308 기록을 보존하고 #3126 항목을 추가했다.
- `rhwp-studio/src/command/commands/file.ts`에서는 #3125의 explicit output 전
  deferred pagination flush와 #3136의 `runPrintPreview` 진입을 함께 보존했다.
- `src/paint/builder.rs`에서는 현재 text-control/character-overlap 회귀 테스트를 유지하면서
  print profile의 editor-only node 제외 테스트를 별도 추가했다.

따라서 원 PR의 conflict를 단순 update branch로 덮지 않고, 누적된 동작을 보존한 메인터너 보정으로
통합했다.

## 검증과 브라우저 시각 근거

| 게이트 | 결과 |
| --- | --- |
| Studio `npx tsc --noEmit` | PASS |
| Studio `npm test` | 565 passed, 0 failed |
| E2E manifest | 76 tracked / 76 manifest, PASS |
| Chrome headless `print-pdf-issue3126.test.mjs --mode=headless` | embedded font, 7쪽/named `@page`, 안내 생략, same-origin preview 모두 PASS |
| Rust 통합 tree gate | fmt/check/release build, library 2,888 passed, 최초 1회 release-test integration/Native Skia/clippy/doctest 완료 |
| 사용자 실행 WASM build | `wasm-pack build --target web --out-dir pkg` PASS |

변경은 opt-in print profile과 브라우저 surface이므로 일반 한컴 화면 sweep 대신 브라우저 E2E와
실제 화면 자산을 동등 근거로 사용했다.

- PDF 안내 모달은 저장 대상 선택·파일명·저장 위치가 브라우저의 native dialog 책임임을 명시하고,
  현재 문서의 filename/dirty 상태를 변경하지 않는다는 안내를 보여 준다.
- print preview는 same-origin 창에서 7쪽 미리보기와 사용자의 `인쇄` 진입점을 보여 준다.
- 안정 자산: `mydocs/pr/assets/pr_3136_postmelee_pdf_guidance.png`,
  `mydocs/pr/assets/pr_3136_postmelee_print_preview.png`
- 임시 E2E 보고서/자산: `output/e2e/print-pdf-issue3126-report.html`,
  `output/e2e/issue-3126/`

contributor는 Windows VDI Edge에서 native dialog와 focused E2E를 수동 확인했다고 보고했다.
검토자는 해당 Windows native dialog를 자동화로 재실증하지 않았으므로, 통합 PR CI와 후속 실제 사용
확인에서 계속 관찰한다.

## 리스크와 권고

브라우저 API는 PDF 대상/저장 경로/사용자의 저장 성공을 강제하거나 판별하지 않는다. 따라서
성공을 허위로 표시하지 않고 남은 단계를 안내하는 현재 설계가 적절하다. Draft의 원 PR은
CONFLICTING/DIRTY이고 maintainer 수정 권한도 없으므로, 통합 PR 최신 head CI 성공을 수용 조건으로 한다.

#3126은 후속 관찰과 browser matrix를 추적해야 하므로 open으로 둔다. 통합 PR merge 후 원 PR을
close/comment할지는 이슈와 원 PR의 미해결 항목을 다시 확인하고 별도 승인으로 처리한다.
