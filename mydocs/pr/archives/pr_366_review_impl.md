# PR #366 구현계획서 — 메인테이너 흡수 (체리픽 방식)

## 배경

PR #366 (@planet6897) 의 결함이 이미 v0.7.7 의 Task #361 로 정정됨. 그러나 PR #366 의 다음 가치를 흡수:

1. **`PageNumberAssigner` 모듈 추출** — TypesetEngine/Paginator 두 경로의 같은 시멘틱을 단일 모듈로 통합
2. **Paginator 경로 정정** — Task #361 은 typeset.rs 만 수정. Paginator 의 신뢰성 낮은 부분 (PR 본문 지적) 도 동일 모듈로 정정
3. **회귀 테스트 신설** — `tests/page_number_propagation.rs` 2건

## 흡수 방식 (체리픽)

PR 작성자의 commit 을 그대로 cherry-pick 하여 attribution 보존. 단, Task #361 과의 충돌은 메인테이너가 해결.

## 흡수 대상 파일

| 파일 | 처리 |
|------|------|
| `src/renderer/page_number.rs` (신규) | **흡수** — PageNumberAssigner 모듈 |
| `src/renderer/mod.rs` | **흡수** — `pub mod page_number;` |
| `src/renderer/typeset.rs::finalize_pages` | **흡수** + Task #361 의 인라인 수정 환원 (Assigner 호출로 통일) |
| `src/renderer/pagination/engine.rs::finalize_pages` | **흡수** — 동일 Assigner 사용 |
| `tests/page_number_propagation.rs` (신규) | **흡수** — 회귀 테스트 |
| `mydocs/plans/task_m05x_353*.md` | **제외** — Task #361 의 문서로 이미 대체 (이슈 #353 도 Task #361 + 본 PR 흡수 로 처리) |
| `mydocs/working/task_m05x_353_stage{1..5}.md` | **제외** |
| `mydocs/report/task_m05x_353_report.md` | **제외** |
| `mydocs/orders/20260426.md` | **제외** (PR 의 변경 — 메인테이너 orders 와 충돌) |

## 단계

### Stage 1: 브랜치 + 체리픽

```bash
git checkout local/devel
git checkout -b local/pr366
git fetch origin pull/366/head:pr-366-head
# 코드 변경만 체리픽 (문서 제외)
```

체리픽 대상 commit 이 단일이면 수동 적용. 다중이면 단일 squash 후 적용.

### Stage 2: Task #361 의 인라인 수정 환원

`src/renderer/typeset.rs::finalize_pages` 의 Task #361 변경:
- 기존: `prev_page_last_para` 추적 + 인라인 NewNumber 적용 조건
- 환원 후: PR #366 의 `PageNumberAssigner::assign(page)` 호출

코드 일관성: TypesetEngine 과 Paginator 양쪽이 동일 모듈 사용.

### Stage 3: 검증

1. `cargo test --lib` — 1008+ passed (Task #361 효과 유지)
2. `cargo test --test svg_snapshot` — 6/6
3. `cargo test --test issue_301` — 1/1
4. **신규** `cargo test --test page_number_propagation` — 회귀 테스트 2건 통과
5. `cargo test --lib renderer::page_number` — 단위 테스트 6건 통과
6. `cargo clippy --lib -- -D warnings`
7. `cargo check --target wasm32-unknown-unknown --lib`
8. **샘플 회귀** — k-water-rfp page_num 1, 2, 3, ..., 26 정상 (Task #361 효과)
9. **kps-ai page_num** — 1, 2, 1, 1, 2~8 등 정상

### Stage 4: 결과 정리

- `mydocs/pr/pr_366_report.md` 작성 — 흡수 처리 결정 + planet6897 감사
- PR 댓글 (정중하고 친절하게) — 처리 절차 + 작성자 가치 인정
- PR close
- 이슈 #353 close
- local/devel 머지 + devel push

## 작성자 attribution

- 체리픽 commit 의 author 를 planet6897 로 보존
- 추가 메인테이너 변경 (Task #361 환원) 은 별도 commit
- 최종 보고서에 planet6897 의 기여 명시

## PR 댓글 메시지 (초안)

정중하고 친절한 톤으로:

```
@planet6897 님 안녕하세요!

쪽번호 처리 결함 #353 에 대한 정성스러운 PR 정말 감사합니다.

본 결함은 동시기에 메인테이너 (Task #361) 에서도 정정되어 v0.7.7 (2026-04-27 배포) 에 이미 포함되었습니다.
다만 본 PR 의 접근이 더 정돈된 부분이 있어서, 메인테이너가 다음 가치를 체리픽 방식으로 흡수하기로 결정했습니다:

1. **PageNumberAssigner 모듈 추출** — TypesetEngine 과 Paginator 두 경로의 같은 시멘틱을
   단일 모듈로 통합 (코드 중복 제거 + 재발 방지)
2. **Paginator 경로의 동일 정정** — Task #361 은 TypesetEngine 만 정정했는데,
   본 PR 이 Paginator 경로의 신뢰성 낮은 부분도 같이 정정한 점이 좋았습니다
3. **회귀 테스트 (`tests/page_number_propagation.rs`)** — 본 결함 재발 방지의 핵심

작성자 분의 attribution 은 체리픽 commit 으로 보존됩니다. 시각/회귀 검증도 통과했습니다.

본 PR 은 close 하지만, 작성자 분의 기여는 v0.7.x 사이클의 코드 정합성에 큰 도움이 됐습니다.
다음 사이클에서도 좋은 PR 부탁드립니다 — 감사합니다! 🙏

상세: `mydocs/pr/pr_366_report.md`
```

## 리스크

| 리스크 | 대응 |
|------|------|
| Task #361 의 시멘틱과 PR #366 의 시멘틱이 미세 차이 | 회귀 테스트로 검증 (k-water-rfp, kps-ai page_num 정상) |
| Paginator 경로 정정 시 다른 회귀 | 7 핵심 샘플 + RHWP_USE_PAGINATOR=1 모드 회귀 검증 |
| 체리픽 시 충돌 | Task #361 의 인라인을 먼저 환원 후 적용 |

## 다음 단계

본 구현계획서 승인 후:
1. Stage 1~4 진행
2. 결과 보고서 + PR 댓글 + close
