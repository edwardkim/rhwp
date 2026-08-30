# 최종 보고서 — #4121 머리말/꼬리말 텍스트 선택

- **이슈**: [#4121](https://github.com/edwardkim/rhwp/issues/4121)
- **PR**: [#6394](https://github.com/edwardkim/rhwp/pull/6394)
- **작업 브랜치**: `codex/issue-4121-hf-selection`
- **기준**: `upstream/devel@2deb3dd61`
- **자동 검증 완료일**: 2026-08-30 KST
- **판정**: 구현·focused 자동 검증·PR 생성 완료, 최신 PR head의 CI와 merge 승인 대기

## 1. 결과

머리말/꼬리말 텍스트를 본문과 독립된 논리 범위로 선택하고, 같은 HF 정의가 적용되는
반복 페이지에 선택 overlay를 투영하도록 구현했다.

- Both/Odd/Even 모두 해당 정의가 속한 구역의 첫 페이지에서 대표 편집
- `꼬리말 · 짝수 쪽 편집 중` 도구 상자 표시와 `꼬리말(짝수 쪽)` canvas 라벨
- 대표 페이지에서 HF를 편집하고, 기존 페이지 여백 꺾쇠를 재사용해 내용을 가리지 않는 영역 표시
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

한글 2024처럼 홀짝 정의도 구역 첫 페이지에서 편집하도록 바꾸고, 타겟 정의와 편집 밴드를
텍스트·색으로 명시했다. 다만 한글 2024의 전용 대화상자·리본 전체, 본문 클릭 차단과 강제 포커스
잠금은 복제하지 않았다. rhwp에서는 본문 클릭으로 HF 모드를 끝낼 수 있고, 대표 페이지 밖의 다른
홀짝 정의를 클릭하면 target만 전환한 뒤 다시 구역 첫 페이지에서 편집한다.

## 3. 자동 검증

| 게이트 | 결과 |
| --- | --- |
| `cargo fmt --all` / `-- --check` / `git diff --check` | 통과 |
| Rust unit tier 정책 | 4,221 tests / 299 modules, 통과 |
| #4121 focused Rust | 7/7 통과 |
| #2724 passthrough guard | 5/5 통과 |
| `cargo clippy --locked --all-targets -- -D warnings` | 통과 |
| 전체 Rust nextest (Stage 4 baseline) | 8,558/8,558 통과, 43 skipped |
| Studio 전체 test | 1,266 passed, 0 failed, 1 skipped |
| Studio production build | 통과 |
| 최적화 WASM | `wasm-pack-locked.sh`, 통과 |
| E2E manifest | tracked 121 / rows 121, 통과 |
| 실제 Google Chrome #4121 E2E | 56/56 통과 |

첫 전체 nextest는 `copy_selection_in_header_footer_native()`가 #2724 패스스루 분류 장부에
없어 1건 실패했다. 이 함수는 문서 IR을 바꾸지 않고 내부 클립보드만 기록하므로 기존 본문·셀
복사 API와 동일한 `SessionState` 예외로 등록했다. 이후 focused guard와 전체 nextest를 다시
실행해 모두 통과했다.

최종 `upstream/devel` rebase 뒤에는 사용자의 최소 검증 요청에 따라 focused Rust 7/7, 관련 Studio
Node test 43/43, production build 241 modules와 실제 Chrome E2E 56/56을 다시 실행했다. 전체
nextest·Clippy·최적화 WASM은 동일한 논리 변경 tree의 이전 단계 결과를 유지하고 최종 rebase 뒤에는
재실행하지 않았다.

로컬 브라우저 증적은 ignored 산출물이며 소스 PR에는 stage하지 않는다.

- `rhwp-studio/e2e/screenshots/issue4121-stage4-both-header-multiline-selection.png`
- `rhwp-studio/e2e/screenshots/issue4121-stage4-odd-even-footer-switch.png`
- `output/e2e/header-footer-selection-issue4121-report.html`

## 4. 남은 확인과 close 판정

자동 검증과 사용자 피드백 기준으로 #4121의 요구 범위와 대표 편집 UX는 해결됐다. 구현은 원격 branch에
push했고 PR #6394를 생성했으며, PR 본문에 `Closes #4121`을 연결했다.

최신 PR head의 required checks와 review가 통과한 뒤 별도 merge 승인을 받는다. merge 전에는 이슈를
직접 닫지 않고, PR이 merge될 때 연결 문구에 따라 #4121을 close한다.
