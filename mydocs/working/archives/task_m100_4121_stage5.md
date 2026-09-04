# Task M100-4121 Stage 5 완료 보고 — 구역 첫 페이지 대표 HF 편집

## 결과

Both/Even/Odd 머리말·꼬리말 정의를 물리 홀짝과 무관하게 해당 source 구역의 첫
페이지에서 편집한다. 편집 표면은 비인쇄 가상 preview이며 문서 IR, pagination의
`active_header`/`active_footer`, 저장·인쇄 결과를 바꾸지 않는다.

- 도구 상자: `머리말 · 양쪽 편집 중`, `꼬리말 · 짝수 쪽 편집 중` 등으로 타겟 표시
- 대표 페이지: 정확한 `PageAreas` HF 밴드에 preview canvas, 강한 테두리, 타겟 배지
- 실제 적용 페이지: 같은 `(section, kind, applyTo)` 정의의 밴드만 약하게 연관 표시
- 타겟 전환: 다른 실제 홀짝 페이지를 클릭해도 선택을 교차하지 않고 첫 페이지로 복귀
- 종료: 본문 클릭 또는 닫기 명령으로 preview 제거, 첫 페이지의 실제 HF 표시 복원

## 의도적 경계

한글 2024의 “홀짝 정의도 첫 페이지에서 편집” 모델과 타겟 표시는 반영했다. 전용 HF
대화상자·리본 전체와 본문 클릭 차단/강제 포커스 잠금은 이 이슈의 범위에 포함하지
않았다. HF 내부 표·그림 개체 편집도 후속 범위로 남겼다.

## 자동 검증

| 게이트 | 결과 |
| --- | --- |
| `cargo fmt --all` / `cargo fmt --all -- --check` / `git diff --check` | 통과 |
| `cargo check --locked` | 통과 |
| #4121 focused Rust | 6/6 통과 |
| `cargo clippy --locked --all-targets -- -D warnings` | 통과 |
| Rust unit tier policy | 4,221 tests / 299 modules 통과 |
| Studio 전체 test | 1,266 passed / 0 failed / 1 skipped |
| Studio production build | 통과 |
| 최적화 WASM | 통과 |
| E2E manifest | 121/121 통과 |
| 실제 Google Chrome #4121 E2E | 56/56 통과 |

시각 증적은 ignored 산출물이며 source PR에 stage하지 않는다.

- `rhwp-studio/e2e/screenshots/issue4121-stage4-both-header-multiline-selection.png`
- `rhwp-studio/e2e/screenshots/issue4121-stage4-odd-even-footer-switch.png`
- `output/e2e/header-footer-selection-issue4121-report.html`

## 사용자 확인 대기

`http://127.0.0.1:7700/`에 최신 WASM을 사용하는 Studio 서버를 유지한다. Even/Odd 정의
생성, 구역 첫 페이지 preview/배지, 실제 홀짝 페이지 반영과 편집 종료 복원을 사용자가
확인한 뒤 원격 제출 단계로 전환한다.
