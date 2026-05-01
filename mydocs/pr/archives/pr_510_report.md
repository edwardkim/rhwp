# PR #510 처리 보고서

**PR**: [#510 Task #508: PageLayerTree image brightness/contrast JSON 필드 추가](https://github.com/edwardkim/rhwp/pull/510)
**작성자**: @postmelee (Taegyu Lee) — alhangeul-macos downstream 관점, 외부 컨트리뷰터
**Linked Issue**: [#508](https://github.com/edwardkim/rhwp/issues/508)
**처리일**: 2026-05-02
**결정**: ✅ **머지 (cherry-pick)** — 검토 문서 §5 권장 + 작업지시자 시각 판정 승인

---

## 1. 처리 결과 요약

| 항목 | 결과 |
|------|------|
| Cherry-pick 대상 commit | `50a9f63` (단일) |
| local/devel cherry-pick commit | `548df60` (메시지 보강: PR #510 footer + `closes #508`) |
| 충돌 | 없음 (PR #510 base = devel, 깨끗한 fast-forward) |
| 1차 게이트 | ✅ build / paint::json (4) / lib (1110) / clippy 모두 통과 |
| 2차 게이트 (통합 회귀) | ✅ issue_418 / issue_501 / svg_snapshot (6/6) 모두 통과 |
| 시각 판정용 SVG | ✅ before/after **byte-identical** (380,804 bytes) — 회귀 0 결정적 검증 |
| 작업지시자 시각 판정 | ✅ 승인 (한컴 2010 + 2020 vs SVG 비교) |
| WASM 영향 | 없음 (Rust API surface 변경 없음) |
| origin/devel push | (Stage 3 진행 중) |

---

## 2. Cherry-pick 상세

### 2.1 commit 메시지 (보강)

```
Task #508: PageLayerTree 이미지 보정 JSON 필드 추가

PR #510 cherry-pick (postmelee, alhangeul-macos downstream).

PaintOp::Image JSON 에 brightness/contrast 필드 추가. core SVG
renderer 와 동일한 image filter 입력값을 PageLayerTree 기반
downstream renderer 에서도 재현 가능.

closes #508
```

### 2.2 변경 파일

| 파일 | 변경 |
|------|------|
| `src/paint/json.rs` | +6 / -2 (PaintOp::Image serialization 4줄 + test assertion 4줄) |
| `mydocs/orders/20260501.md` | +6 / -0 (M100 #508 항목 추가) |
| `mydocs/plans/task_m100_508.md` | 신규 (수행 계획서, 125 줄) |
| `mydocs/plans/task_m100_508_impl.md` | 신규 (구현 계획서, 178 줄) |
| `mydocs/working/task_m100_508_stage{1-3}.md` | 신규 (단계별 보고서 3 파일) |
| `mydocs/report/task_m100_508_report.md` | 신규 (최종 보고서, 110 줄) |

소스 1 파일 + 문서 7 파일, 총 +655 / -2.

---

## 3. 검증 결과

### 3.1 1차 게이트 (Stage 1)

```
cargo build --lib                                                      ✅ Finished
cargo test --lib paint::json                                           ✅ 4 passed (0 failed)
cargo test --lib paint::json::tests::serializes_backend_replay_payload_fields  ✅ 1 passed
cargo test --lib                                                       ✅ 1110 passed, 0 failed, 1 ignored
cargo clippy --lib -- -D warnings                                      ✅ 통과
```

PR 본문 보고치 1102 → 현재 시점 1110 (메모리 동기화 / Task #509 등 devel 진행분 반영분). 회귀 0.

### 3.2 2차 게이트 (Stage 2)

```
cargo test --test issue_418   ✅ 1 passed (Task #418 셀 padding 회귀)
cargo test --test issue_501   ✅ 1 passed (Task #501 mel-001 회귀)
cargo test --test svg_snapshot ✅ 6/6 passed (PR #506 머지 후 사전 회귀 정정 확인)
```

### 3.3 시각 판정 자료 (Stage 2)

- 베이스라인: `output/svg/pr510_before/복학원서.svg` (380,804 bytes)
- 정정 후: `output/svg/pr510_after/복학원서.svg` (380,804 bytes)
- **`diff before/after`**: **byte-identical** ★

byte-identical 결과는 본 PR 의 본질 (PageLayerTree JSON contract 보강) 정합과 일치. core SVG 출력 변경 없음 (이슈 #508 / PR #510 본문 전제) 결정적 입증.

### 3.4 작업지시자 시각 판정

- 한컴 2010 + 한컴 2020 으로 `samples/복학원서.hwp` 직접 출력 → cherry-pick 후 SVG 비교
- 메모리 `feedback_visual_regression_grows` / `feedback_pdf_not_authoritative` / `reference_authoritative_hancom` 정합
- **결과: 승인** — 회귀 0 확인

---

## 4. 후속 정합 사항

| 항목 | 처리 |
|------|------|
| 이슈 #508 close | cherry-pick commit 의 `closes #508` 로 origin/devel push 시 자동 close |
| 이슈 #508 milestone v1.0.0 | (Stage 3 후속) 추가 |
| 이슈 #508 enhancement 라벨 | (Stage 3 후속) 추가 |
| 이슈 #508 assignee | (Stage 3 후속) 메인테이너 지정 |
| README 기여자 목록 갱신 | (별도 PR 또는 후속 commit) postmelee 추가 |
| PR #510 close | `gh pr close 510 --comment "cherry-pick 머지 완료"` |

---

## 5. 메모리 정합 (적용 사례)

| 메모리 | 적용 |
|--------|------|
| `feedback_check_open_prs_first` | 이슈 #508 → PR #510 연결 확인 |
| `feedback_release_sync_check` | Stage 3 시작 전 `git fetch origin` → 분기 없음 확인 |
| `feedback_commit_reports_in_branch` | 본 보고서를 local/devel 에서 cherry-pick 과 함께 commit 후 push |
| `feedback_assign_issue_before_work` | Stage 3 후속에서 이슈 #508 assignee 지정 |
| `feedback_close_issue_verify_merged` | origin/devel push 후 `git branch --contains 548df60 origin/devel` 검증 |
| `feedback_update_daily_orders` | `mydocs/orders/20260502.md` 갱신 |
| `feedback_visual_regression_grows` | Stage 2 시각 판정 게이트 필수 적용 |
| `feedback_pdf_not_authoritative` / `reference_authoritative_hancom` | 한컴 2010 + 2020 직접 시각 판정 |
| `feedback_pr_comment_tone` | PR/이슈 댓글 차분, 사실 중심 |

---

## 6. PR #507 / #510 처리 결과 비교

| 항목 | PR #507 | PR #510 |
|------|---------|---------|
| 본질 | 시각 결함 정정 (squashing) | 데이터 contract 보강 (JSON 필드) |
| core SVG 출력 변화 | O | X (byte-identical 입증) |
| 시각 판정 게이트 | 필수 | 필수 |
| 시각 fixture | 미적분03.hwp 미존재 → 수정 요청 | 복학원서.hwp + PDF 존재 |
| **결정** | **수정 요청 (open)** | **머지 완료** |

---

## 7. 다음 단계

1. local/devel → devel merge + push (Stage 3 진행 중)
2. PR #510 close + 이슈 #508 자동 close 검증
3. 이슈 메타데이터 정합 (milestone / labels / assignee)
4. `mydocs/orders/20260502.md` 갱신
