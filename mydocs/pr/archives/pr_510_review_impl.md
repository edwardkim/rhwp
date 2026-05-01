# PR #510 구현 계획서 (cherry-pick)

**대상 PR**: [#510 Task #508: PageLayerTree image brightness/contrast JSON 필드 추가](https://github.com/edwardkim/rhwp/pull/510)
**Linked Issue**: [#508](https://github.com/edwardkim/rhwp/issues/508)
**검토 결정**: 머지 권장 (cherry-pick) — `mydocs/pr/pr_510_review.md` §5
**작성일**: 2026-05-02

---

## 1. 사전 점검 결과

| 항목 | 상태 |
|------|------|
| 현재 브랜치 | `local/devel` ✅ |
| 워킹트리 | clean (`mydocs/pr/pr_507_review.md`, `pr_510_review.md` untracked, cherry-pick 영향 없음) |
| local/devel vs origin/devel | 1 ahead, 0 behind (메모리 동기화 커밋 `016cf27`) |
| origin/main vs origin/devel | devel 59 commits ahead (정상) |
| PR #510 head fetch | ✅ `local/pr510` 로 fetch 완료 |
| 대상 commit | `50a9f63` (단일 — Task #508 본질) |
| Merge commit | `c924d3c` (cherry-pick 대상 아님 — devel→task508 merge) |

---

## 2. cherry-pick 전략

### 2.1 단일 commit 일괄 cherry-pick

PR #510 본질 commit 은 1건 (`50a9f63`) 으로, 소스 1 파일 + 문서 7 파일 (수행/구현 계획서, 단계별 보고서 3건, 최종 보고서, 오늘할일 갱신) 이 모두 포함되어 있다. 분할 cherry-pick 의 가치가 없다 → **단일 일괄 cherry-pick** 으로 처리.

### 2.2 작업 브랜치

내부 정책 (메모리 `feedback_commit_reports_in_branch` 정합) 으로 임시 브랜치를 거치지 않고 **`local/devel` 직접 cherry-pick**. 검증 게이트 통과 후 그대로 push 전 확인.

근거:
- PR #510 의 변경 영역이 매우 작음 (소스 +6/-2)
- 회귀 위험 0 (검토 문서 §4)
- merge commit 분리 불필요 (단일 commit)

리스크 회피용으로 임시 브랜치 (`local/pr510-cherry`) 가 필요하면 4.1 단계에서 분기 가능.

### 2.3 cherry-pick 메시지 정책

PR 작성자 (postmelee) 의 원본 commit 메시지를 보존하되, 메인테이너 머지 정합을 위해 footer 추가:

```
Task #508: PageLayerTree 이미지 보정 JSON 필드 추가

PR #510 cherry-pick (postmelee, alhangeul-macos downstream).

closes #508
```

`-x` 옵션 (cherry-pick source SHA 자동 footer) 또는 수동 footer. 메인테이너 정합을 위해 **수동 footer** 로 작성 (PR 번호 + 컨트리뷰터 명시).

---

## 3. 검증 게이트

### 3.1 1차 게이트 (cherry-pick 직후)

| 게이트 | 명령 | 통과 기준 |
|--------|------|----------|
| 컴파일 | `cargo build` | exit 0 |
| 핵심 회귀 | `cargo test --lib paint::json` | 4 passed (0 failed) |
| 본 PR 회귀 | `cargo test --lib paint::json::tests::serializes_backend_replay_payload_fields` | 1 passed |
| 전체 lib | `cargo test --lib` | 1102+ passed (PR 본문 보고치) |
| Clippy | `cargo clippy --lib -- -D warnings` | 통과 |
| 통합 테스트 | `cargo test --test issue_418` + `--test issue_501` | 회귀 0 |

### 3.2 2차 게이트 (필수 — 작업지시자 시각 판정)

PR #510 은 **데이터 contract 보강** 이고 core SVG 출력 자체는 변경되지 않는 전제이지만, **회귀 부재를 보장하기 위해 시각 판정을 필수 게이트로 적용** 한다.

근거:
- 메모리 `feedback_visual_regression_grows` — cargo test 통과만으로는 시각 결함 검출 불가. 작업지시자 시각 판정이 절차의 핵심 게이트.
- 메모리 `feedback_pdf_not_authoritative` / `reference_authoritative_hancom` — 한컴 2010 + 한컴 2020 직접 시각 판정만 정답지로 인정.
- 본 PR 은 paint serialization 영역이지만 상호 영역 (image filter 입력 경로) 에서 부수 효과 발생 가능성 — 결정적 검증 외 시각 판정으로 보강.

**메인테이너 작업 (Stage 2):**

1. cherry-pick 전 (베이스라인) SVG 생성:
   ```bash
   git stash  # cherry-pick 결과 임시 보관 (또는 별도 브랜치 사용)
   rhwp export-svg samples/복학원서.hwp -o output/svg/pr510_before/
   ```
   또는 cherry-pick 전에 미리 베이스라인 생성 후 진행.

2. cherry-pick 후 SVG 생성:
   ```bash
   rhwp export-svg samples/복학원서.hwp -o output/svg/pr510_after/
   ```

3. (선택) PageLayerTree JSON 출력에 `brightness=-50`, `contrast=70` 필드 존재 확인.

**작업지시자 시각 판정 (게이트 통과 조건):**

- `samples/복학원서.hwp` 의 워터마크 페이지를 한컴 2010 + 한컴 2020 으로 직접 출력
- 본 PR cherry-pick 후 SVG 와 시각 비교
- 동일 출력 (회귀 0) 또는 의도된 보정 효과 정합 확인
- 작업지시자 승인 후 Stage 3 진행

**판정 자료 첨부:**
- `output/svg/pr510_before/` 및 `output/svg/pr510_after/` 의 워터마크 페이지 SVG
- 한컴 PDF (`samples/복학원서.pdf`) 와 비교
- 작업지시자 환경의 한컴 2010/2020 출력 (작업지시자 제공)

### 3.3 svg_snapshot 점검

PR #507 검토 (§3.5) 에서 PR #506 머지 후 svg_snapshot 6/6 통과 확인 필요했음. 현재 시점 (2026-05-02, local/devel) 에서 svg_snapshot 상태 재확인:

```bash
cargo test --test svg_snapshot
```

통과 기준: 6/6 통과 (회귀 0).

---

## 4. 단계 분리 (3 stages)

### Stage 1 — cherry-pick 실행 + 1차 게이트

**작업:**
1. 현재 브랜치 `local/devel` 확인
2. `git cherry-pick 50a9f63` (단일 commit)
3. cherry-pick 충돌 점검 — 예상 충돌: `mydocs/orders/20260501.md` (PR #510 의 M100 #508 추가 + 타스크 #509 / 메모리 동기화 등 본 브랜치 작업분)
4. 충돌 발생 시: 본 브랜치의 기존 항목 보존 + PR #510 의 #508 항목 병합. 충돌 해소 후 `git cherry-pick --continue`
5. cherry-pick 메시지 수정 — 위 §2.3 정책으로 footer 추가
6. `cargo build` + `cargo test --lib paint::json` + `cargo test --lib` + `cargo clippy --lib -- -D warnings` 실행
7. 단계 보고서 작성 (선택)

**예상 충돌:** `mydocs/orders/20260501.md` 에서 발생 가능. PR #510 의 base (`devel` 시점) 와 현재 `local/devel` 의 orders 파일 차이 예상.

**산출물:**
- cherry-pick commit (local/devel 위)
- 1차 게이트 통과 로그

**완료 기준:**
- exit 0 (모든 게이트)
- `git log --oneline -3` 으로 cherry-pick commit 확인

### Stage 2 — 통합 회귀 + svg_snapshot + 작업지시자 시각 판정

**작업 (메인테이너):**
1. `cargo test --test issue_418` (Task #418 회귀 — 셀 padding)
2. `cargo test --test issue_501` (Task #501 회귀 — mel-001)
3. `cargo test --test svg_snapshot` (6/6 통과 확인)
4. (선택) `cargo test` 전체 — lib + integration + doc-tests
5. WASM 빌드 갱신 — 본 PR 은 WASM API surface 추가 없음 (확인). WASM 산출물 갱신 commit 필요 시 별도 commit
6. **시각 판정용 SVG 생성** (§3.2 2차 게이트):
   - cherry-pick 전 베이스라인: 별도 브랜치 또는 cherry-pick 전에 `rhwp export-svg samples/복학원서.hwp -o output/svg/pr510_before/` 실행 후 보관
   - cherry-pick 후: `rhwp export-svg samples/복학원서.hwp -o output/svg/pr510_after/`
7. **작업지시자에게 시각 판정 요청** — 워터마크 페이지 SVG 비교 + 한컴 2010/2020 출력 비교

**작업 (작업지시자):**
- 한컴 2010 + 한컴 2020 으로 `samples/복학원서.hwp` 출력
- 메인테이너가 생성한 `output/svg/pr510_before/` / `pr510_after/` SVG 와 시각 비교
- 회귀 0 또는 의도된 정합 확인 후 Stage 3 승인

**완료 기준:**
- 모든 통합 테스트 통과
- svg_snapshot 6/6
- WASM 영향 없음 확인
- **작업지시자 시각 판정 승인 완료** ★

### Stage 3 — local/devel push + PR/이슈 close + 보고

**작업:**
1. local/devel push 사전 점검 — `git pull --ff-only origin devel` (메모리 `feedback_release_sync_check` 정합)
2. local/devel → devel push 절차:
   - `git checkout devel`
   - `git merge local/devel --no-ff -m "Merge local/devel: PR #510 cherry-pick (Task #508 PageLayerTree image brightness/contrast)"`
   - `git push origin devel`
3. PR #510 close — `gh pr close 510 --repo edwardkim/rhwp --comment "cherry-pick 머지 완료. 감사합니다."`
4. 이슈 #508 close — cherry-pick commit 메시지에 `closes #508` 포함되므로 자동 close 확인. 미작동 시 `gh issue close 508 --repo edwardkim/rhwp`
5. 이슈 #508 메타데이터 정합 — milestone v1.0.0 + enhancement 라벨 + assignee 지정 (검토 문서 §5 후속)
6. 최종 보고서 `mydocs/pr/pr_510_report.md` 작성
7. `mydocs/orders/20260502.md` 갱신 (오늘할일 — 메모리 `feedback_update_daily_orders` 정합)

**완료 기준:**
- origin/devel 에 cherry-pick commit 반영
- PR #510 closed (merged 상태 아님 — cherry-pick 정합)
- 이슈 #508 closed
- `pr_510_report.md` 작성 완료

---

## 5. 위험 영역 + 회피책

| 위험 | 가능성 | 회피책 |
|------|--------|--------|
| `mydocs/orders/20260501.md` cherry-pick 충돌 | 🟨 중간 | 수동 충돌 해소. PR #510 의 #508 항목 + 본 브랜치의 기존 항목 모두 보존 |
| `paint::json` 단위 테스트 실패 (drift) | 🟢 매우 작음 | PR CI 통과 + 검토 문서 §3.4 의 기존 통과 보고. devel 진행분과 충돌 없음 (PR #510 base = devel) |
| svg_snapshot 회귀 (PR #506 / #509 영향) | 🟢 매우 작음 | PR #506 / #509 머지 후 svg_snapshot 6/6 통과 보고 (Task #509 commit `1a3322b`) |
| WASM 산출물 drift | 🟢 0 | 본 PR 은 WASM API surface 추가 없음. WASM 빌드 갱신 commit 불필요 |
| 메인테이너 push 시점 불일치 | 🟢 매우 작음 | Stage 3 의 `git pull --ff-only origin devel` 게이트로 점검 |

---

## 6. 롤백 계획

cherry-pick 후 검증 실패 시:
1. `git reset --hard HEAD~1` 로 cherry-pick commit 제거
2. 충돌/실패 원인 진단 → 작업지시자 보고
3. PR #510 에 메인테이너 댓글 — 추가 정정 요청 또는 close 사유 명시

origin/devel push 후 회귀 발견 시:
1. revert commit 생성 — `git revert <cherry-pick-sha>`
2. `git push origin devel`
3. 이슈 재개 + 작업지시자 보고

메모리 `feedback_close_issue_verify_merged` 정합 — close 전 `git branch --contains <commit>` 으로 devel 머지 검증.

---

## 7. 메모리 정합

- `feedback_release_sync_check` — Stage 3 시작 전 `git pull --ff-only origin devel` 점검
- `feedback_commit_reports_in_branch` — 보고서 (`pr_510_report.md`, `orders/20260502.md`) 는 local/devel 에서 cherry-pick 과 별도 commit 으로 작성 후 push
- `feedback_assign_issue_before_work` — 이슈 #508 머지 전 assignee 지정 (Stage 3.5)
- `feedback_close_issue_verify_merged` — close 전 cherry-pick commit 의 devel 머지 검증
- `feedback_update_daily_orders` — `orders/20260502.md` 갱신 (PR #510 처리 항목 추가)
- `feedback_visual_regression_grows` — Stage 2 의 작업지시자 시각 판정을 필수 게이트로 적용
- `feedback_pdf_not_authoritative` / `reference_authoritative_hancom` — 한컴 2010 + 2020 직접 시각 판정만 정답지로 인정
- `feedback_v076_regression_origin` — 결정적 검증 (cargo test) 외 시각 판정 게이트 보강으로 회귀 origin 패턴 차단

---

## 8. 다음 단계

작업지시자 승인 후 Stage 1 (cherry-pick 실행) 진행.
