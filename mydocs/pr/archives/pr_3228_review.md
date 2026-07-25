---
kind: reference
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-07-24
---

# PR #3228 검토 기록 — 머리말/꼬리말 진입 존재 확인

## 메타

| 항목 | 값 |
| --- | --- |
| 원 PR | [#3228](https://github.com/edwardkim/rhwp/pull/3228) |
| 작성자 | `lpaiu-cs` |
| 원 head | `cb5aaec889bdb879ad86619c392b24d8ddd49ea2` |
| base / 상태 | `devel` / `BEHIND` |
| 누적 검토 브랜치 | `review/lpaiu-cs-20260724` (`upstream/devel` `c8611dd84d002d2a776c040387bf21cf270f6448`) |
| 적용 순서 / 로컬 SHA | 1) `c21a65063` → `cabfec7ef`, 2) `cb5aaec88` → `d676b7054` |
| 충돌 | 없음 |

## 변경과 판단

현재 쪽을 건너뛰도록 설계된 `navigate_header_footer_by_page_native`를 존재 확인으로 재사용해 1쪽에서는
`currentPage - 1`이 unsigned underflow하는 결함을, `getHeaderFooter(sectionIdx, isHeader, applyTo)`로
바꿨다. 생성과 존재 확인이 같은 네이티브 범위 질의를 쓰므로 재진입 시 중복 생성 거부와 UI 판단이
일치한다. 쪽 이동 명령의 본래 의미는 변경하지 않는다.

초기 커밋에 대한 P2는 `cb5aaec88`에서 현재 구역의 active header/footer 정보를 기준으로 편집 대상과
fallback을 고쳐 해소됐다. #3208의 선행 변경은 이미 `upstream/devel`에 포함돼 있으며, #3240의 활성
머리말 선택 통일을 같은 누적 브랜치에서 뒤이어 적용해 대상 선택과 화면 표시 규칙도 함께 확인했다.

## 독립 검증

| 검증 | 결과 |
| --- | --- |
| `cargo test --profile release-test --test issue_3206_hf_edit_target` | 3 passed |
| `rhwp-studio: npx tsc --noEmit` | PASS |
| `rhwp-studio: npm test` | 636 passed, 0 failed |
| `cargo test --profile release-test --tests` | PASS |
| `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --doc` | PASS (doc 4 passed, 2 ignored) |
| `wasm-pack build --target web --out-dir pkg` | PASS |

모든 cargo 검증은 `CARGO_INCREMENTAL=0`, 검토 전용
`CARGO_TARGET_DIR=target/lpaiu-cs-20260724-review`에서 직렬 실행했다.

## 최종 권고

**통합 PR 수용 후보.** 원 PR은 `BEHIND`이므로 직접 merge하지 않고, 위의 누적 체리픽과 review 문서를
포함한 collaborator 통합 PR의 최신 CI 성공 및 작업지시자 승인 후 반영한다.
