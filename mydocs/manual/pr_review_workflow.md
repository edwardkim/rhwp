# PR 리뷰 · 통합 워크플로우 매뉴얼

**작성일**: 2026-04-23
**대상**: rhwp 메인테이너 (외부 PR 처리 담당)
**교훈 기반**: v0.2.1 사이클 외부 기여자 9명 / PR 10+ 건 통합 경험

---

## 1. 개요

rhwp 는 v0.2.1 사이클부터 외부 기여자 PR 이 급증했다. 하이퍼-워터폴 방법론에 맞춰 **PR 처리도 표준화된 절차** 로 운영한다. 본 매뉴얼은 외부 PR 도착 시 메인테이너가 따르는 순서를 기록한다.

## 2. PR 도착 시 확인 체크리스트

새 PR 이 열리면 다음을 순서대로 확인한다.

### 2.1 기본 메타

- [ ] **base 브랜치**: `devel` 이어야 함. `main` 이면 rebase 요청 (이전 재리젝 사례: PR #234)
- [ ] **연관 이슈**: PR description 에 `closes #N` 또는 `#N` 참조가 있어야 함. 없으면 코멘트로 요청
- [ ] **mergeable state**: `MERGEABLE` + `CLEAN` / `BEHIND` / `DIRTY` 중 확인
- [ ] **CI 상태**: `Build & Test` / `CodeQL` 실행 결과

### 2.2 규모 분석

```bash
gh pr view N --repo edwardkim/rhwp --json additions,deletions,files,commits
```

- 규모 파악 → 리뷰 시간 예측
- **대형 PR** (>1000 라인) → 별도 검토 사이클, 바로 머지 불가
- **소형 PR** (<100 라인) → admin merge 고려 가능

### 2.3 작성자 확인

- **FIRST_TIME_CONTRIBUTOR**: 환영 인사 + 세심한 피드백 톤
- **기존 기여자**: 이전 PR 컨텍스트 참고

## 3. 리뷰 문서 작성

각 PR 마다 `mydocs/pr/pr_{N}_review.md` + `pr_{N}_review_impl.md` 2건 작성한다.

### 3.1 리뷰 문서 (`pr_N_review.md`)

포함 항목:
- PR 메타 표 (번호 / 작성자 / base / 규모 / mergeable)
- 관련 이슈 요약
- 변경 범위 분석 (핵심 기능 / 메타 변경 / 범위 외)
- 사전 검증 결과 (로컬 빌드 / 테스트 / Clippy / doctest)
- 주요 문제점 / 리스크
- 최종 권고 (admin merge / rebase 요청 / 재작업 요청 / close)

예시: `mydocs/pr/archives/pr_234_review.md` (재작업 요청), `mydocs/pr/archives/pr_251_review.md` (admin merge 권고)

### 3.2 구현 계획서 (`pr_N_review_impl.md`)

포함 항목:
- 커밋별 SHA + 제목
- Stage 구성 (승인 → merge → sync → cleanup)
- 작업지시자 확인 필요 사항 (merge 방식, 코멘트 톤, 후속 이슈)

## 4. 로컬 사전 검증

### 4.1 PR 브랜치 fetch

```bash
git fetch origin pull/N/head:local/prN
```

### 4.2 Merge 시뮬레이션

```bash
git checkout -b prN-merge-test local/prN
git merge origin/devel --no-commit --no-ff
# 충돌 여부 확인
git status
```

충돌 없으면 그대로 진행, 충돌 시 해결 방침 작업지시자 결정 요청.

### 4.3 빌드 · 테스트

```bash
cargo build --lib
cargo test --lib
cargo clippy -- -D warnings
cargo test --doc
```

**렌더 영향 PR 추가 체크**:

```bash
cargo test --test svg_snapshot
```

실패 시 `UPDATE_GOLDEN=1` 으로 재생성해야 하지만 **PR 머지 후가 아닌 머지 전에도 확인 필요**. 실패하면 작업지시자와 상의 (의도된 렌더 변경인지 버그인지).

### 4.4 정리

```bash
git merge --abort
git checkout local/devel
git branch -D prN-merge-test
```

## 5. 작업지시자 승인 요청

리뷰 문서 2건을 근거로 승인 요청. 예시 포맷:

```
PR #N 검토 결과 · admin merge 준비 완료.

- mergeable: MERGEABLE / BEHIND
- 충돌 시뮬레이션: 0건
- cargo test --lib: XYZ passed
- Clippy: 0 warning
- 리뷰 문서: mydocs/pr/pr_N_review.md

어떻게 진행할까요?
- A) admin merge
- B) 추가 검증
- C) 보류
```

## 6. Admin Merge 수행

```bash
gh pr merge N --repo edwardkim/rhwp --merge --admin
```

**주의**: `--admin` 플래그는 BEHIND 상태도 강제 머지한다. 프로젝트가 "devel 만 push · main 은 릴리즈 시" 정책이므로 `--admin` 이 기본.

## 7. 후속 처리 (필수 순서)

### 7.1 이슈 Close 확인

GitHub auto-close 가 **자주 실패**한다. 수동 확인:

```bash
gh issue view N --repo edwardkim/rhwp --json state,closedAt
```

`state: OPEN` 이면 수동 close + 감사 코멘트:

```bash
gh issue close N --repo edwardkim/rhwp --comment "PR #M 머지로 해결 (by @작성자). ..."
```

### 7.2 기여자 감사 코멘트

PR 에 감사 + 검증 결과 요약 + 다음 PR 격려:

```
@기여자 감사합니다. 머지 완료했습니다.

[검증 결과 요약]
- 충돌 0 / cargo test ... passed / Clippy 0 warning

[재제출 피드백이 있었던 경우] 이번에 반영해주신 점:
- ... (구체 항목 1)
- ... (구체 항목 2)

[다음 작업 언급 — 있으면] 후속 이슈 #X 도 같은 방식으로 올려주시면 됩니다.

감사합니다.
```

### 7.3 devel Sync

```bash
git fetch origin
git checkout local/devel
git rebase origin/devel   # 로컬 작업분이 있으면 머지 커밋 위로 재적용
```

### 7.4 렌더 영향 PR 의 경우 · Golden 재생성 체크

**반드시** 다음을 확인:

```bash
cargo test --test svg_snapshot
```

실패 시:

```bash
UPDATE_GOLDEN=1 cargo test --test svg_snapshot
cargo test --test svg_snapshot   # 결정성 재확인
git add tests/golden_svg/
git commit -m "test(svg_snapshot): regenerate golden after #N (...)"
git push origin devel
```

2회 연속 재현된 실수 (PR #221 / PR #251 사이클) 로 인해 **체크리스트 수준의 필수 절차**.

### 7.5 리뷰 문서 archives 이동

```bash
mv mydocs/pr/pr_N_review.md mydocs/pr/archives/
mv mydocs/pr/pr_N_review_impl.md mydocs/pr/archives/
```

다음 커밋에 포함하거나 오늘할일 커밋에 동반.

### 7.6 로컬 브랜치 정리

```bash
git branch -D local/prN
```

### 7.7 오늘할일 갱신

`mydocs/orders/yyyymmdd.md` 에 해당 PR 처리 내역 기록:
- PR 번호 + 제목 + 작성자
- merge SHA
- 관련 이슈 close 여부
- 후속 작업 (있으면)

## 8. 재작업 요청 패턴

리뷰 결과 **재작업이 필요**한 PR (예: base 가 main, 메타 변경 혼입, 관련 이슈 없음 등):

1. **PR 에 정중한 피드백 코멘트** + 구체적 수정 요청 목록
2. **PR close** (재제출 기대 의사 표명)
3. 재제출 시 새 PR 번호로 처리

실제 성공 사례: PR #234 (close) → PR #251 (재제출, 모든 피드백 반영 후 admin merge).

피드백 톤 가이드:
- **결정 자체는 단호**: "현 상태로는 머지 불가"
- **사유는 구체적**: "base 가 main 이라 릴리즈 브랜치에 직접 커밋되는 구조"
- **재제출 경로는 명확**: "feature 브랜치에서 devel 타깃으로 재제출 부탁드립니다"
- **크레딧 약속**: "재제출 시 PR description / commit author 보존 그대로"

## 9. 예외 케이스

### 9.1 Dependabot PR

`dependabot/npm_and_yarn/...` 브랜치 PR:
- 보통 base 가 `main` (설정 이슈) → `.github/dependabot.yml` 에 `target-branch: devel` 추가로 해결
- 현재 main 타깃 PR 은 close + 수동으로 devel 에 버전 bump 커밋

### 9.2 오래된 base PR (대량 커밋 혼입)

예: PR #213 같이 수십 커밋 전의 base 에서 분기 → diff 에 이미 머지된 과거 커밋들이 포함됨

처리:
- 해당 기여자의 **신규 커밋만 cherry-pick** (저자 보존)
- PR 은 close + 설명 코멘트 ("이번 기여 2 커밋만 cherry-pick 반영했습니다")
- 중복 PR (같은 브랜치 main 타깃) 도 함께 close

### 9.3 대형 PR (>1000 라인)

- 즉시 admin merge 불가
- 코드 검토 + 사전 시뮬레이션 충분히 수행 후 결정
- 예: PR #165 (skia renderer · +100K 라인) — 장기 보류

## 10. 메모리 등록 항목 (자동 참조)

다음 상황은 `~/.claude/.../memory/` 에 등록되어 있다:

- `feedback_search_troubleshootings_first.md` — 작업 전 트러블슈팅 폴더 검색
- `feedback_external_docs_self_censor.md` — 외부 공개 문서 자기검열
- (신규 제안) `feedback_golden_regen_after_render_pr.md` — 렌더 PR 머지 후 golden 재생성

## 11. 참고 아카이브

- `mydocs/pr/archives/pr_234_review.md` — 재작업 요청 사례
- `mydocs/pr/archives/pr_235_review.md` · `pr_237_review.md` — 다양한 리뷰 패턴
- `mydocs/pr/archives/pr_251_review.md` — 재제출 후 머지 사례 (모든 피드백 반영)
