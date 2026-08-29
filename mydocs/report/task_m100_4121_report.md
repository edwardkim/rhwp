# 최종 보고서 — #4121 머리말/꼬리말 텍스트 선택

- **이슈**: [#4121](https://github.com/edwardkim/rhwp/issues/4121)
- **작업 브랜치**: `codex/issue-4121-hf-selection`
- **기준**: `upstream/devel@f6a6bee8f3`
- **자동 검증 완료일**: 2026-08-29 KST
- **판정**: 구현·자동 통합 검증 완료, 사용자 수동 확인과 원격 제출 승인 대기

## 1. 결과

머리말/꼬리말 텍스트를 본문과 독립된 논리 범위로 선택하고, 같은 HF 정의가 적용되는
반복 페이지에 선택 overlay를 투영하도록 구현했다.

- 마우스 드래그, Shift+클릭과 Shift 방향/Home/End 선택
- 단일·다문단 범위와 화면 밖 페이지의 scroll-in 재투영
- Both는 모든 적용 페이지, Odd/Even은 같은 정의의 페이지만 강조
- Delete/Backspace, 입력, IME, 평문 붙여넣기, copy/cut과 부분 글자 서식
- 연산별 Undo/Redo 선택 계약과 `preferredPage` 복원
- 다른 홀짝 정의 클릭 시 교차 선택 없이 target 전환
- 본문 클릭 시 HF 모드 종료, 선택이 있는 Esc는 선택만 해제

## 2. 한글 2024 관찰과 rhwp 결정

사용자 VDI에서 확인한 한글 2024의 반복 머리말 선택 동작을 데이터 모델 근거로 사용했다.
화면 밖 반복 페이지는 스크롤해 렌더될 때 선택이 나타나고, 같은 정의를 편집하면 다른 적용
페이지에도 결과가 반영됐다. Both/Odd/Even 정의별 투영도 이 관찰과 맞췄다.

다만 한글 2024의 전용 머리말/꼬리말 대화상자·리본, 본문 클릭 차단과 강제 포커스 잠금은
복제하지 않았다. rhwp에서는 본문 클릭으로 HF 모드를 자연스럽게 끝내며, 다른 홀짝 정의를
클릭하면 안전하게 target을 전환한다. 이는 #4121의 텍스트 선택·편집 범위를 충족하면서 기존
Studio 상호작용을 유지하는 의도적 차이다.

## 3. 자동 검증

| 게이트 | 결과 |
| --- | --- |
| `cargo fmt --all` / `-- --check` / `git diff --check` | 통과 |
| Rust unit tier 정책 | 4,221 tests / 299 modules, 통과 |
| integration suite manifest | 1,018 sources / 4,508 static attrs / 48 targets, 통과 |
| #4121 focused Rust | 6/6 통과 |
| #2724 passthrough guard | 5/5 통과 |
| `cargo clippy --locked --all-targets -- -D warnings` | 통과 |
| 전체 Rust nextest | 8,558/8,558 통과, 43 skipped |
| Studio 전체 test | 1,254 passed, 0 failed, 1 skipped |
| Studio production build | 통과 |
| 최적화 WASM | `wasm-pack-locked.sh`, 통과 |
| 실제 Google Chrome #4121 E2E | 50/50 통과 |

첫 전체 nextest는 `copy_selection_in_header_footer_native()`가 #2724 패스스루 분류 장부에
없어 1건 실패했다. 이 함수는 문서 IR을 바꾸지 않고 내부 클립보드만 기록하므로 기존 본문·셀
복사 API와 동일한 `SessionState` 예외로 등록했다. 이후 focused guard와 전체 nextest를 다시
실행해 모두 통과했다.

로컬 브라우저 증적은 ignored 산출물이며 소스 PR에는 stage하지 않는다.

- `rhwp-studio/e2e/screenshots/issue4121-stage4-both-header-multiline-selection.png`
- `rhwp-studio/e2e/screenshots/issue4121-stage4-odd-even-footer-switch.png`
- `output/e2e/header-footer-selection-issue4121-report.html`

## 4. 남은 확인과 close 판정

자동 검증 기준으로 #4121의 요구 범위는 해결됐다. 다만 사용자가 직접 사용할 수 있는 로컬
서버에서 mouse/Shift 선택, 다문단 편집, 반복 페이지 투영과 홀짝 분리를 한 번 확인한 뒤
close 가능으로 최종 판정한다.

원격 push, PR 생성과 이슈 close는 아직 수행하지 않았다. 사용자 수동 확인 후 별도 승인을
받아 PR을 제출하고, CI 및 review 결과까지 통과하면 #4121을 close해도 된다.
