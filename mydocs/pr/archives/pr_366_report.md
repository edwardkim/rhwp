# PR #366 처리 보고서 — 메인테이너 흡수 (체리픽 방식)

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#366](https://github.com/edwardkim/rhwp/pull/366) |
| 작성자 | [@planet6897](https://github.com/planet6897) (Jaeook Ryu) |
| base / head | `devel` ← `task353` |
| 이슈 | [#353](https://github.com/edwardkim/rhwp/issues/353) (NewNumber 매 페이지 재적용 결함) |
| 처리 결정 | **메인테이너 흡수 (체리픽 방식) + close** |
| 처리 일자 | 2026-04-27 |

## 처리 결정 사유

본 결함 (NewNumber 매 페이지 재적용) 은 작업지시자 + Claude 가 동시기에 **Task #361** 으로 정정하여 v0.7.7 (2026-04-27 배포) 에 이미 포함됨.

그러나 PR #366 의 다음 가치를 흡수:

1. **`PageNumberAssigner` 모듈 추출** — TypesetEngine 과 Paginator 두 경로의 같은 시멘틱을 단일 모듈로 통합 (코드 중복 제거 + 재발 방지)
2. **Paginator 경로의 동일 정정** — Task #361 은 TypesetEngine 만 수정. PR #366 가 Paginator 의 신뢰성 낮은 부분 (`page_last_para = max(...)` 단조성 의존) 도 같이 정정
3. **회귀 테스트** (`tests/page_number_propagation.rs` 2건 + 단위 테스트 6건) — 본 결함 재발 방지

## 흡수 절차

### Stage 1: 체리픽
- `local/pr366` 브랜치 (`local/devel` 분기)
- PR #366 의 두 commit (`0a5fbad`, `8955bf2`) 그대로 cherry-pick
- 작성자 (planet6897 = Jaeook Ryu) attribution 보존

### Stage 2: 메인테이너 정리
- PR 의 `task_m05x_353*.md` 문서 8개 제거 (Task #361 의 `task_m100_361*.md` 문서로 이미 대체)
- `mydocs/orders/20260426.md` 의 M05x 섹션 제거 (Task #361 항목과 통합)

### Stage 3: 검증
| 항목 | 결과 |
|------|------|
| `cargo test --lib` | **1014 passed, 0 failed** (1008 + 6 page_number 단위 테스트) |
| `cargo test --test svg_snapshot` | 6/6 통과 |
| `cargo test --test issue_301` | 1/1 통과 |
| `cargo test --test page_number_propagation` | **2/2 통과** (회귀 테스트) |
| `cargo test --lib renderer::page_number` | 6/6 통과 (단위 테스트) |
| `RHWP_USE_PAGINATOR=1 cargo test --test page_number_propagation` | 2/2 통과 (Paginator 경로) |
| `cargo clippy --lib -- -D warnings` | 통과 |
| `cargo check --target wasm32-unknown-unknown --lib` | 통과 |

### Stage 4: 페이지네이션 회귀 검증

| 샘플 | 페이지 | LAYOUT_OVERFLOW |
|------|------|------|
| form-01 | 1 | 0 |
| aift | 77 | 3 |
| KTX | 27 | 1 |
| k-water-rfp | 27 | 0 |
| exam_eng | 11 | 0 |
| kps-ai | 79 | 5 |
| hwp-multi-001 | 10 | 0 |

→ **모든 샘플 페이지 수 + LAYOUT_OVERFLOW 무변화** (Task #361 효과 유지).

### page_num 회귀

- **k-water-rfp section=1**: page_num 1, 2, 3, ... (정상)
- **kps-ai**: page_num 1, 2, 1, 1, 2~9 (NewNumber 정상 처리, v0.7.3 와 일치)

## 흡수 commit 목록

```
d40d15f PR #366 흡수: task_m05x_353 문서 제거 (Task #361 의 문서로 대체)
fa10ead Task #353: page_number_propagation 테스트 — 샘플 부재 시 skip [planet6897]
6cd97f0 Task #353: 쪽번호 처리 — NewNumber 매 페이지 재적용 회귀 수정 [planet6897]
```

## 작성자 기여 인정

**@planet6897 (Jaeook Ryu)** 의 본 PR 은 v0.7.x 사이클의 코드 정합성에 큰 도움이 됐습니다:
- PageNumberAssigner 모듈 설계 — TypesetEngine 과 Paginator 두 경로 통합
- Paginator 경로의 신뢰성 낮은 부분 정확히 식별 + 정정
- 회귀 테스트 신설로 본 결함의 재발 방지

체리픽 commit 으로 Author attribution 보존됨.

## PR 댓글 + close 완료

- 정중하고 친절한 톤으로 처리 절차 + 가치 인정 + 감사 메시지
- 작성자 다음 PR 환영
- PR close 처리

## 이슈 #353 처리

- Task #361 (이미 v0.7.7 에 포함) + 본 PR 흡수로 해결됨
- 이슈 #353 close 예정 (작업지시자 승인 후)

## 다음 단계

- local/devel 머지 (작업지시자 승인 후)
- devel push
- 다음 v0.7.8 사이클에서 PageNumberAssigner 흡수 효과 포함

## 참고

- 검토 문서: `mydocs/pr/pr_366_review.md`
- 구현계획서: `mydocs/pr/pr_366_review_impl.md`
- Task #361 (이미 처리됨): `mydocs/report/task_m100_361_report.md`
- v0.7.7 릴리즈: 2026-04-27 (https://github.com/edwardkim/rhwp/releases/tag/v0.7.7)
