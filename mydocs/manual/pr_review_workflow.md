# PR 리뷰 · 통합 워크플로우 매뉴얼

**작성일**: 2026-04-23
**대상**: rhwp 메인테이너 (외부 PR 처리 담당) + collaborator self-merge 후보 예외 경로
**교훈 기반**: v0.2.1 사이클 외부 기여자 9명 / PR 10+ 건 통합 경험

---

## 1. 개요

rhwp 는 v0.2.1 사이클부터 외부 기여자 PR 이 급증했다. 하이퍼-워터폴 방법론에 맞춰 **PR 처리도 표준화된 절차** 로 운영한다. 본 매뉴얼의 기본 경로는 외부 PR 도착 시 메인테이너가 따르는 순서를 기록한다.

운영 경로는 다음 세 가지로 구분한다.

- **maintainer 일반 경로**: 메인테이너가 외부 기여자 PR 을 검토, merge, 후속 보고한다. 이 문서의 기본 절차다.
- **collaborator self-merge 후보 예외 경로**: collaborator 가 본인 PR 을 merge 후보로 준비하면서 review 문서를 PR head 에 포함하는 경우다. 이 경로는 8장 조건을 만족할 때만 적용한다.
- **collaborator-mediated 외부 PR 경로**: 외부 contributor PR 을 repository collaborator 가 검토·merge 준비하면서 review 문서를 PR head 에 포함하는 경우다. 이 경로는 9장 조건을 만족할 때만 적용한다.

명령 예시는 원본 저장소 remote 가 `upstream` 인 현재 로컬 checkout 기준이다. 원본 저장소를 `origin` 으로 둔 maintainer checkout 에서는 같은 명령의 remote 이름만 `origin` 으로 치환한다.

## 2. PR 도착 시 확인 체크리스트

새 PR 이 열리면 다음을 순서대로 확인한다.

### 2.1 reviewer assign 선행

PR review 처리를 시작하기 전에 먼저 reviewer 를 assign 한다. reviewer assign 이 빠지면 PR 검토 상태가
GitHub 상에서 추적되지 않으므로, 메타 확인·로컬 fetch·문서 작성보다 앞서 수행한다.

```bash
gh pr edit N --repo edwardkim/rhwp --add-reviewer <reviewer>
```

여러 PR 을 체리픽 누적 검토하는 경우에도 각 원 PR 번호별로 reviewer 를 먼저 assign 한 뒤 개별 review
문서를 작성한다.

### 2.2 기본 메타

- [ ] **base 브랜치**: `devel` 이어야 함. `main` 이면 rebase 요청 (이전 재리젝 사례: PR #234)
- [ ] **연관 이슈**: PR description 에 `closes #N` 또는 `#N` 참조가 있어야 함. 없으면 코멘트로 요청
- [ ] **mergeable state**: `MERGEABLE` + `CLEAN` / `BEHIND` / `DIRTY` 중 확인
- [ ] **CI 상태**: `Build & Test` / `CodeQL` 실행 결과

### 2.3 규모 분석

```bash
gh pr view N --repo edwardkim/rhwp --json additions,deletions,files,commits
```

- 규모 파악 → 리뷰 시간 예측
- **대형 PR** (>1000 라인) → 별도 검토 사이클, 바로 머지 불가
- **소형 PR** (<100 라인) → admin merge 고려 가능

### 2.4 작성자 확인

- **FIRST_TIME_CONTRIBUTOR**: 환영 인사 + 세심한 피드백 톤
- **기존 기여자**: 이전 PR 컨텍스트 참고

### 2.5 update branch 이후 이전 SHA CI 강제 취소

PR 검토 중 contributor 또는 maintainer 가 `Update branch` / `devel` merge 로 PR head 를 갱신하면, 이전 head
SHA 에서 시작된 GitHub Actions run 이 그대로 남아 새 head 의 required check 와 섞여 보일 수 있다. 이 경우
다음 순서로 **이전 SHA run 만** 정리한다. 최신 PR head 의 CI 는 취소하지 않는다.

1. 최신 PR head SHA 를 확인한다.

```bash
gh pr view N --repo edwardkim/rhwp --json headRefOid
```

2. 취소 대상이 되는 이전 SHA 의 run 을 확인한다.

```bash
gh run list --repo edwardkim/rhwp --commit <old-sha> \
  --json databaseId,workflowName,status,conclusion,headSha,url --limit 20
```

3. `queued` / `pending` / `in_progress` run 은 일반 cancel 을 시도하지 않고, 처음부터 GitHub Actions
   **force-cancel API** 로 취소한다.

```bash
gh api --method POST repos/edwardkim/rhwp/actions/runs/<run-id>/force-cancel
```

4. 최종 상태가 `status: completed`, `conclusion: cancelled` 인지 확인한다.

```bash
gh run list --repo edwardkim/rhwp --commit <old-sha> \
  --json databaseId,workflowName,status,conclusion,headSha,url --limit 20
```

기록 예시: PR #1738 검토 중 이전 SHA `00bc0428173c9f413171f3b070c923089320d050` 의 CI/CodeQL run 은
처음부터 `force-cancel` API 로 최종 `cancelled` 처리했다.

## 3. 리뷰 문서 작성

maintainer 일반 경로에서는 각 PR 마다 리뷰 문서 2건을 active 경로에 작성한다.

```text
mydocs/pr/pr_{N}_review.md
mydocs/pr/pr_{N}_review_impl.md
```

처리 완료 후 7.6 절에서 `mydocs/pr/archives/` 로 이동한다. collaborator self-merge 후보 또는
collaborator-mediated 외부 PR 처럼 처음부터 archive 경로에 작성하는 방식은 8장·9장 예외 경로에서만
사용한다.

### 3.1 리뷰 문서 (`pr_N_review.md`)

포함 항목:
- PR 메타 표 (번호 / 작성자 / base / 규모 / mergeable 작성 시점 참고값)
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

### 3.3 volatile 상태값 기록 규칙

PR review 문서는 merge 후에도 모순되지 않아야 한다. 따라서 다음 값은 확정 사실처럼 기록하지 않는다.

- `draft`
- `mergeable`
- `head SHA`
- `CI 상태`

필요하면 다음 방식으로만 기록한다.

- "문서 작성 시점 참고값: ..."
- "merge 전 최신 상태 확인 필요"
- "최종 merge 조건: PR head 최신 커밋 기준 GitHub Actions 통과 + 작업지시자 승인"

금지 예시:

- `draft: true` 를 현재 상태처럼 단정
- `mergeable: CLEAN` 을 최종 merge 가능 판정처럼 기록
- 특정 `head SHA` 를 "현재 head" 로만 적고 merge 전 재확인 조건을 남기지 않음
- 과거 GitHub Actions 통과 상태를 최신 통과 조건 없이 최종 판정처럼 기록

### 3.4 가설 기각/재분류 PR 판단 규칙

조사 PR 이 어떤 가설을 **기각**하거나 다른 원인 계통으로 **재분류**하는 경우, 그것만으로 merge 보류나
reject 사유로 보지 않는다. PR 의 목적이 "처음 가설대로 고치는 것"이 아니라 "가설이 틀렸음을 증명하고
후속 분류를 확정하는 것"이면, 그 증명 자체가 merge 대상이다.

이 유형의 PR 은 다음 조건을 만족하면 merge 후보로 판단할 수 있다.

- 최종 보고서, stage 문서, README, 샘플 설명이 모두 같은 결론을 가리킨다.
- 초기 가설은 "초기 가설" 또는 "검증 대상"으로만 남기고, 확정 사실처럼 쓰지 않는다.
- 기각/재분류 근거가 샘플, 기준 PDF, 하니스 결과, visual sweep, 로그 중 적절한 자료로 남아 있다.
- 후속 추적 이슈가 필요하면 어떤 이슈로 이관되는지 명확히 적는다.

반대로, 최종 보고서는 기각/재분류를 말하는데 계획서나 README가 초기 가설을 사실처럼 유지하면 merge
보류 사유다. 이때는 PR 작성자에게 수정을 요청하거나, 메인터너 권한으로 문서를 보정한 뒤 "가설 기각이
맞음을 증명하는 PR"로 merge 판단한다.

### 3.5 시각 검증 및 asset 기록 규칙

일반 PR review, collaborator-mediated review, 여러 PR 체리픽 누적 검토 모두에서 PR 내용상 렌더링 결과
확인이 필요하면 [PDF/SVG visual sweep 가이드](visual_sweep_guide.md)를 사용할 수 있다. 시각 검증은 모든
샘플 PR 에 기계적으로 수행하는 절차가 아니라, PR 의 수정 목적과 검증해야 할 사용자-visible 동작에 맞춰
선택한다.

시각 검증 후보 예시는 다음과 같다.

- renderer/layout/typeset/paint/wasm 출력이 바뀌는 PR
- HWP/HWPX 샘플, 기준 PDF, visual regression 자료가 추가된 PR
- 페이지 수, 쪽 경계, 표/그림 자리차지, 텍스트 wrap, clipping, 색상/밑줄처럼 눈으로 확인해야 하는 이슈
- 코드 검증은 통과하지만 PR 설명상 기준 PDF 또는 한컴 출력과의 비교가 필요한 PR

Codex 와 Claude 는 visual sweep 을 수행한 경우 `compare`, `overlay`, `review` PNG 경로와
`visual_accuracy_proxy_percent` 를 제시하고, 검증 이미지를 확인한 뒤 작업지시자 승인 없이 시각 판정을
최종 통과로 단정하지 않는다.

시각 검증이 반드시 필요한 PR review 에 기준 PDF 가 첨부되어 있지 않으면, maintainer 또는 collaborator 에게
한컴 2020/2024 등 실제 기준 프로그램에서 저장한 PDF 파일 업로드를 요청한다. 기준 PDF 없이 생성한 자동
비교 결과는 임시 참고 자료로만 기록하고 최종 시각 판정 근거로 사용하지 않는다.

maintainer 또는 collaborator 가 PR review 검증을 위해 추가한 기준 PDF/HWP/HWPX 샘플이 untracked 로
존재하면, 해당 파일은 임시 디버그 산출물이 아니라 검증 근거 샘플로 간주한다. PR review 문서, visual
asset 과 함께 커밋 대상에 포함하고, review 문서에는 샘플 경로와 어떤 검증에 사용했는지 기록한다.

시각 검증 PNG 를 PR 기록 자산으로 남길 때는 먼저 PR review 문서에 visual sweep 산출물의 임시 경로
(`output/.../review/review_NNN.png` 등), 페이지 수, 자동 후보 수, `visual_accuracy_proxy_percent`, 사람
판정 메모를 기록한다. 이후 asset 반영은 작업 상황에 따라 다음 두 방식 중 하나를 선택한다.

**옵션 1. 현재 PR 에 review 문서와 asset 을 함께 포함**

현재 PR 에 문서/asset 을 함께 포함해도 되는 경우 사용한다. PR review 문서를 `mydocs/pr/archives/` 로
이동하고, 선택한 검증 PNG 를 `mydocs/pr/assets/` 아래로 옮긴 뒤 같은 PR branch 에 remote push 한다.
그 PR 이 merge 된 뒤 원시 PR 또는 supersede 된 PR close/comment 시 `devel` 기준 asset 링크와 리뷰 결론을
남긴다.

**옵션 2. 원 PR merge 후 docs-only 문서/asset PR 로 분리**

코드 PR 과 review 기록/asset 을 분리해야 하거나 fast-pass 문서 PR 로 후속 처리하는 것이 더 안전한 경우
사용한다. 원 코드 PR 을 먼저 merge 한 뒤, 선택한 검증 PNG 를 `mydocs/pr/assets/` 아래로 옮기고 review
문서/오늘할일과 함께 별도 후속 문서/자산 PR 로 올린다. 후속 문서/자산 PR 을 merge 하여 PNG 가 `devel` 에
존재하게 만든 다음, 원시 PR 또는 supersede 된 PR 에 review comment 를 남기며 asset 링크와 리뷰 결론을
안내한다.

원시 PR comment 에 넣는 이미지 링크는 다음 형식을 사용한다.

```markdown
![PR #N visual review](https://raw.githubusercontent.com/edwardkim/rhwp/devel/mydocs/pr/assets/<file>.png)
```

## 4. 로컬 사전 검증

### 4.1 PR 브랜치 fetch

```bash
git fetch upstream pull/N/head:local/prN
```

### 4.2 Merge 시뮬레이션

```bash
git checkout -b prN-merge-test local/prN
git merge upstream/devel --no-commit --no-ff
# 충돌 여부 확인
git status
```

충돌 없으면 그대로 진행, 충돌 시 해결 방침 작업지시자 결정 요청.

### 4.2.1 여러 PR 체리픽 누적 검토

여러 PR 이 같은 영역을 단계적으로 수정하고 오래된 순서대로 merge 해야 하는 경우, 별도 로컬 브랜치에서
`upstream/devel` 기준 체리픽 누적 검토를 할 수 있다. 이때도 리뷰 기록은 반드시 원 PR 번호별로 분리한다.

- 체리픽 순서는 오래된 PR 번호 또는 작업지시자가 지정한 순서를 따른다.
- PR 내부의 `Merge branch 'devel' ...` 커밋은 검토 목적 체리픽에서 제외하고, 실제 기능/문서 커밋만 적용한다.
- 누적 브랜치는 충돌, 테스트, 시각 검증 확인용 임시 브랜치일 뿐이며 review 문서를 묶어서 한 파일로 만들지 않는다.
- `mydocs/pr/pr_{N}_review.md`, `mydocs/pr/pr_{N}_review_impl.md` 는 각 PR 번호별로 작성한다.
- 각 review 문서에는 체리픽 순서, 적용한 커밋 SHA, 충돌 여부, 선행 PR 의존 여부를 해당 PR 기준으로 기록한다.
- 시각 검증이 필요한 경우 3.5 절의 공통 visual sweep 및 asset 기록 규칙을 따른다.
- 여러 PR 을 한꺼번에 로컬 검증했더라도 GitHub merge 전에는 각 PR 의 최신 head, mergeable, required checks 를 개별 확인한다.

### 4.3 빌드 · 테스트

```bash
cargo build --release
cargo test --release --lib
cargo test --profile release-test --tests
cargo fmt --check
git diff --check
cargo clippy --all-targets -- -D warnings
cargo test --doc
cd rhwp-studio && npx tsc --noEmit
cd rhwp-studio && npm test
wasm-pack build --target web --out-dir pkg
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

- mergeable: MERGEABLE / BEHIND (승인 요청 시점 참고값, merge 전 재확인)
- 충돌 시뮬레이션: 0건
- cargo test --lib: XYZ passed
- Clippy: 0 warning
- 리뷰 문서: mydocs/pr/pr_N_review.md
- merge 전 조건: PR head 최신 커밋 기준 GitHub Actions 통과 + 작업지시자 승인

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

### 7.3 후속 문서 처리 여부 확정

이슈 close 확인과 기여자/PR 후속 코멘트를 남긴 뒤에는 **반드시 후속 문서 처리 여부를 확정**한다.
PR 처리 사실이 GitHub metadata 에만 남고 `mydocs/orders/{yyyymmdd}.md`, PR review/report, task report 에
누락되는 것을 막기 위한 게이트다.

다음 중 하나로 결론을 남긴다.

- **PR head 에 이미 포함됨**: collaborator self-merge 후보 또는 collaborator-mediated 외부 PR 처럼 review 문서,
  오늘할일, report 가 merge 된 PR diff 에 함께 포함된 경우. 이때도 merge SHA, 이슈 close 여부, supersede close
  결과처럼 merge 후에야 확정되는 값이 빠졌는지 확인한다.
- **별도 후속 기록 PR 필요**: merge 후 확정된 사실을 기존 문서에 보강해야 하거나, 오늘할일에 처리 기록이
  빠진 경우. `mydocs/**` 와 신규 기준 샘플/PDF 만 포함하는 후속 기록 브랜치를 만들고 PR 로 반영한다.
- **추가 문서 불필요**: PR review/report/오늘할일이 모두 최신이고, merge 후 확정값 누락도 없는 경우. 이 판단은
  상태 보고에 명시한다.

후속 문서 PR 을 만드는 경우 원칙:

- 코드, 기존 샘플, workflow 파일을 섞지 않고 `mydocs/**` 문서와 신규 기준 샘플/PDF 만 변경한다.
- 후속 기록 변경이어도 `git diff --check` 를 실행한다.
- PR 본문에는 대상 PR/이슈 번호, merge SHA, 이슈 close 여부, fast-pass 조건을 기록한다.
- fast-pass 가 적용되어 heavy job 이 `skipped` 로 보이더라도 preflight 성공과 merge 가능 상태를 확인한 뒤 merge 한다.
- 후속 문서 PR merge 후 `devel` 을 다시 sync 하고, 문서 PR 브랜치도 로컬/원격 모두 정리한다.

### 7.4 devel Sync

```bash
git fetch upstream
git checkout local/devel
git rebase upstream/devel   # 로컬 작업분이 있으면 머지 커밋 위로 재적용
```

### 7.5 렌더 영향 PR 의 경우 · Golden 재생성 체크

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
git push upstream devel
```

2회 연속 재현된 실수 (PR #221 / PR #251 사이클) 로 인해 **체크리스트 수준의 필수 절차**.

### 7.6 리뷰 문서 archives 이동

maintainer 일반 경로의 PR 리뷰 문서는 처리 완료 후 archive 경로로 이동한다.

```bash
mv mydocs/pr/pr_N_review.md mydocs/pr/archives/
mv mydocs/pr/pr_N_review_impl.md mydocs/pr/archives/
```

다음 커밋에 포함하거나 오늘할일 커밋에 동반한다. collaborator self-merge 후보 예외 경로에서는
처음부터 archive 경로에 두므로 이 이동 단계를 수행하지 않는다.

### 7.7 로컬/원격 PR 작업 브랜치 정리

PR 처리용으로 별도 worktree 를 만들었다면 브랜치 삭제 전에 worktree 를 먼저 제거한다. worktree 가 남아 있으면
해당 브랜치가 checkout 중인 상태라 `git branch -D` 가 실패한다.

```bash
git worktree list
git worktree remove /path/to/pr-worktree
```

```bash
git branch -D local/prN
```

collaborator self-merge 후보처럼 원본 저장소에 PR head 브랜치를 직접 만든 경우에는 merge 후 원격
작업 브랜치도 삭제한다. 예를 들어 PR head 가 `upstream/task_m100_1470` 또는
`upstream/task_m100_1601_mydocs_fast_pass` 형태라면 다음을 수행한다.

```bash
git checkout devel
git merge --ff-only upstream/devel
git worktree list
# 해당 브랜치를 checkout 한 별도 worktree 가 있으면 먼저 제거한다.
git worktree remove /path/to/pr-worktree
git push upstream --delete task_m100_1470
git branch -D task_m100_1470
git fetch upstream --prune
```

삭제 후에는 worktree, 로컬 브랜치, 원격 추적 브랜치가 남지 않았는지 확인한다.

```bash
git worktree list
git branch --list 'task_m100_1470'
git branch -r | rg 'task_m100_1470' || true
git ls-remote --heads upstream task_m100_1470
```

### 7.8 오늘할일 갱신

maintainer 일반 경로에서는 PR merge 와 후속 처리를 끝낸 뒤 이 절을 수행한다. collaborator 경로의
오늘할일 생성·갱신 시점은 8.2.1 절 또는 9.2.1 절을 따른다.

`mydocs/orders/yyyymmdd.md` 에 해당 PR 처리 내역 기록:
- PR 번호 + 제목 + 작성자
- merge SHA
- 관련 이슈 close 여부
- 후속 작업 (있으면)

## 8. Collaborator self-merge 후보 예외 경로

이 절은 collaborator 가 본인 PR 을 self-merge 후보로 준비하는 경우에만 적용한다. maintainer 가 외부
기여자 PR 을 검토하는 일반 경로를 대체하지 않는다.

### 8.1 적용 조건

- PR 작성자 또는 작업 준비자가 repository collaborator 이다.
- PR 번호가 이미 생성되어 review 문서명을 확정할 수 있다.
- merge 후 별도 문서 커밋을 만들지 않기 위해 review 문서를 PR diff 에 함께 포함해야 한다.
- 작업지시자 승인 전에는 ready 전환 또는 merge 판단을 하지 않는다.

### 8.2 문서 경로

collaborator self-merge 후보에서는 처음부터 archive 경로에 review 문서 2건을 작성할 수 있다.

```text
mydocs/pr/archives/pr_{N}_review.md
mydocs/pr/archives/pr_{N}_review_impl.md
```

이 방식은 PR head 에 운영 문서를 포함해 merge 후 추가 문서 커밋을 방지하기 위한 예외다.
maintainer 일반 경로의 active 경로 작성 규칙까지 대체하지 않는다.

이미 active 경로에 잘못 만들었더라도 다음 PR 에 임시 동반하는 식으로 일반화하지 말고, 같은 PR 준비
단계에서 archive 경로로 바로 정리한 뒤 PR head 에 포함한다.

### 8.2.1 오늘할일 생성·갱신 시점

collaborator self-merge 후보에서 오늘할일 갱신이 필요한 경우, PR 착수나 최초 조사 단계에서
`mydocs/orders/{yyyymmdd}.md` 를 미리 생성하지 않는다. merge 판단과 후속 처리 계획이 확정된 최종 PR
review 문서 묶음을 작성할 때 오늘할일을 같은 커밋으로 생성·갱신한다.

여기서 최종 PR review 문서 묶음은 다음 중 해당 PR 에 실제로 필요한 문서다.

- `mydocs/pr/archives/pr_{N}_review.md`
- `mydocs/pr/archives/pr_{N}_review_impl.md`
- `mydocs/pr/archives/pr_{N}_report.md` (필요 시)

### 8.3 remote push 규칙

collaborator 는 PR용 작업 브랜치를 fork 저장소(`origin`)에 우회 생성하지 않는다. 로컬 브랜치에서
원본 저장소 remote(`upstream`)의 작업 브랜치로 직접 push 하는 것을 기본 규칙으로 삼는다.

```bash
git push upstream HEAD:task_m100_1158
```

fork 브랜치를 head 로 쓰는 방식은 권한 제약 때문에 직접 push 가 불가능한 경우에만 예외로 둔다.

### 8.4 merge 전 최종 조건

collaborator self-merge 후보라도 최종 merge 판단은 다음 조건을 모두 만족해야 한다.

- PR head 최신 커밋 기준 GitHub Actions 통과
- review 문서와 처리 계획서가 PR diff 에 포함됨
- 작업지시자 승인

`draft`, `mergeable`, `head SHA`, `CI 상태`는 3.3 절에 따라 작성 시점 참고값 또는 merge 전 최신 확인
조건으로만 기록한다.

## 9. Collaborator-Mediated 외부 PR 처리 경로

이 절은 외부 contributor PR 을 repository collaborator 가 검토하고 merge 준비하는 경우에 적용한다.
maintainer 일반 경로를 대체하지 않으며, 별도 문서 PR 을 만들지 않기 위해 review 문서를 해당 PR head 에
포함하는 예외 경로다.

이 경로가 필요한 이유는 collaborator 권한 모델 때문이다. collaborator 는 원본 저장소 `devel` 에 직접
문서 커밋을 push 하지 않고 PR 을 통해서만 변경을 반영한다. 따라서 외부 PR 을 merge 한 뒤
`pr_{N}_report.md` 만 별도 문서 PR 로 올리는 방식은 PR 처리 비용을 불필요하게 늘린다. 또한 문서만 있는 PR 은
CI `paths-ignore` 조건 때문에 핵심 검증이 실행되지 않을 수 있어, "처리 후 report 작성 -> 별도 문서 PR merge" 를
기본 흐름으로 삼지 않는다.

실제 선례:

- PR #1376: `mrshinds` 외부 PR 에 maintainer 보정·review 문서·오늘할일을 PR head 에 포함한 뒤 merge
- PR #1429: `seo-rii` 외부 PR 에 review 문서·오늘할일을 PR head 에 포함한 뒤 merge
- PR #1447: `seo-rii` 외부 PR 에 review/report 문서·오늘할일을 PR head 에 포함한 뒤 merge

### 9.1 적용 조건

- PR 작성자는 외부 contributor 이다.
- repository collaborator 가 리뷰, 문서화, merge 준비를 담당한다.
- GitHub PR 의 `maintainer_can_modify` 가 `true` 이거나, contributor 가 collaborator 의 문서 커밋 push 를
  명시적으로 허용한다.
- review 문서만 별도 PR 로 만들지 않기 위해 PR head 에 운영 문서를 포함하는 편이 더 단순하다.
- 작업지시자 승인 전에는 GitHub review approval, ready 전환, merge 판단을 완료하지 않는다.

`maintainer_can_modify=false` 이면 이 경로를 쓰지 않는다. 이 경우 maintainer 일반 경로로 active review 문서를
작성하거나, 작업지시자 지시에 따라 별도 문서 커밋/PR 로 처리한다.

### 9.2 문서 경로

collaborator-mediated 외부 PR 에서는 PR head 에 다음 문서를 직접 포함할 수 있다.

```text
mydocs/pr/archives/pr_{N}_review.md
mydocs/pr/archives/pr_{N}_review_impl.md   # 필요 시
mydocs/pr/archives/pr_{N}_report.md        # 필요 시, 사전 처리 판단 보고서로 작성
```

오늘할일 갱신이 필요한 경우 `mydocs/orders/{yyyymmdd}.md` 도 같은 PR head 에 포함한다.

### 9.2.1 오늘할일 생성·갱신 시점

collaborator-mediated 외부 PR 경로에서는 오늘할일을 PR 착수나 최초 조사 단계에서 미리 생성하지 않는다.
외부 contributor PR 에 얹을 최종 PR review 문서 묶음을 작성할 때 오늘할일을 같은 커밋으로
생성·갱신한다. 이렇게 해야 작업 중간 상태가 오늘할일에 먼저 확정 기록처럼 남지 않고, PR head 에 포함되는
운영 문서와 오늘할일이 같은 판단 시점을 공유한다.

여기서 최종 PR review 문서 묶음은 다음 중 해당 PR 에 실제로 필요한 문서다.

- `mydocs/pr/archives/pr_{N}_review.md`
- `mydocs/pr/archives/pr_{N}_review_impl.md`
- `mydocs/pr/archives/pr_{N}_report.md`

단순·소형 PR 은 `pr_{N}_review.md` 안에 처리 계획을 포함하고 별도 `review_impl` 을 생략할 수 있다.
`pr_{N}_report.md` 를 함께 포함할 때는 merge 완료 후 사후 보고서가 아니라 **사전 처리 판단 보고서**로 작성한다.
따라서 아직 확정되지 않은 merge SHA, 실제 merge 시각, 이슈 close 완료 여부를 단정하지 않는다. 대신 다음을
기록한다.

- merge 수용/보류/재작업 권고와 사유
- merge 전 최종 조건
- merge 후 확인해야 할 이슈 close, 감사 코멘트, 후속 작업

merge 완료 사실과 이슈 close 결과는 GitHub PR/Issue metadata 를 원천 기록으로 삼고, 별도 문서 PR 을 만들지
않는다. 사후에 반드시 장기 보관 보고서가 필요한 예외는 작업지시자 승인 후 별도 PR 로 처리한다.

### 9.3 PR head push 규칙

외부 contributor 브랜치에 collaborator 가 커밋을 얹을 때는 다음을 지킨다.

- contributor 의 원 코드 커밋을 rewrite 하지 않는다.
- review 문서, 오늘할일, maintainer 보정 코드는 별도 커밋으로 분리한다.
- maintainer 보정 코드가 포함되면 review 문서에 contributor 원 변경과 collaborator 추가 변경을 구분한다.
- 문서 커밋 push 후 GitHub Actions 결과를 확인한다. full CI 재실행 또는 9.3.1 절의 fast-pass 결과가
  merge 가능 상태여야 한다.

예시:

```bash
git fetch upstream pull/N/head:local/prN
git checkout local/prN
# review 문서 작성 및 검증
git commit -m "docs: PR #N 검토 기록"
git push https://github.com/{contributor}/rhwp.git HEAD:{head-branch}
```

### 9.3.1 후속 기록 PR fast-pass

collaborator 가 외부 PR head 또는 별도 후속 PR 에 review/운영 문서와 기준 샘플을 보강하는 경우,
다음 조건을 모두 만족하면 heavy CI 가 job-level 로 skip 될 수 있다.

- PR head 의 뒤쪽 후속 커밋들이 아래 항목만 변경한다.
  - `mydocs/**`
  - 신규 추가(`added`) 상태의 `samples/**/*.hwp`
  - 신규 추가(`added`) 상태의 `samples/**/*.hwpx`
  - 신규 추가(`added`) 상태의 `samples/**/*.pdf`
  - 신규 추가(`added`) 상태의 `pdf/**/*.pdf`
- 기존 `samples/**` 또는 `pdf/**` 파일을 수정, 삭제, rename 한 경우 fast-pass 대상이 아니다.
- 해당 후속 커밋들은 single-parent commit 이다.
- 후속 문서 커밋을 제외한 직전 코드 검증 대상 SHA 에 기존 GitHub Actions check-run 이 존재한다.
- 직전 코드 검증 대상 SHA 의 relevant check 가 `success`, `skipped`, `neutral` 중 하나다.

fast-pass 는 merge 조건을 약화하는 예외가 아니다. 이전 코드 검증 결과를 재사용하는 좁은 최적화일 뿐이다.
다음 변경이 포함되면 반드시 최신 PR head 기준 heavy CI 를 다시 기다린다.

- 코드, 테스트, workflow 파일 변경
- 기존 샘플, baseline, golden, 렌더링 fixture 변경
- `mydocs/**` 밖의 파일 변경 또는 신규 기준 샘플/PDF 추가가 아닌 실행/검증 입력 파일 변경
- check-run 조회 실패, missing check, failed check, merge commit 형태의 문서 후속 커밋

fast-pass 가 적용되면 PR UI 에서 heavy job 이 `skipped` 로 보일 수 있다. 이때도 collaborator 는 GitHub Actions
결과가 merge 가능 상태인지 확인하고, branch protection 이 요구하는 check 가 pending/failing 이면 merge 하지
않는다.

### 9.4 merge 전 최종 조건

- PR head 최신 커밋 기준 GitHub Actions 통과 또는 9.3.1 절의 후속 기록 PR fast-pass 결과 확인
- review 문서가 PR diff 에 포함됨
- `pr_{N}_report.md` 를 작성한 경우 사전 판단 보고서 형식이며, merge 후 사실을 미리 단정하지 않음
- GitHub review 또는 PR comment 로 검토 결과를 contributor 에게 남김
- merge 전 최신 `mergeable` / `mergeStateStatus` 재확인
- 작업지시자 승인

이 경로로 merge 한 뒤에는 7장 후속 처리를 동일하게 수행한다. 특히 `devel` 이 default branch 가 아니어서
`closes #N` 자동 close 가 실패할 수 있으므로 관련 이슈 state 를 반드시 확인한다.

## 10. 재작업 요청 패턴

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

### 10.1 영어 요청 contributor 응답 언어

contributor 가 영어로 PR 설명, 질문, 검토 요청을 남긴 경우에도 메인테이너 기록성과 외부 contributor 배려를
함께 만족하도록 **하나의 코멘트 안에 한글 문단과 영어 문단을 구분해 병기**한다.

- 한글 문단을 먼저 작성하고, `---` 구분선 뒤에 같은 의미의 영어 문단을 작성한다.
- 문장 단위로 한영을 섞지 않는다. 읽기 쉽도록 언어별 문단을 분리한다.
- 칭찬, 감사, CI 실패 원인, 수정 요청, 다음 검토 조건을 두 언어 모두에 같은 수준으로 포함한다.
- draft PR 인 경우 "현재 draft 는 정식 PR 검토 대상에 포함하지 않으며, 검토를 원하면 Ready for review 로
  전환해 달라"는 요청도 두 언어 모두에 포함한다.

예시 구조:

```markdown
@contributor 한글 감사/긍정 피드백 문단.

한글 CI 상태와 수정 요청 문단.

---

English thanks / positive feedback paragraph.

English CI status and requested fix paragraph.
```

## 11. 예외 케이스

### 11.1 Dependabot PR

`dependabot/npm_and_yarn/...` 브랜치 PR:
- 보통 base 가 `main` (설정 이슈) → `.github/dependabot.yml` 에 `target-branch: devel` 추가로 해결
- 현재 main 타깃 PR 은 close + 수동으로 devel 에 버전 bump 커밋

### 11.2 오래된 base PR (대량 커밋 혼입)

예: PR #213 같이 수십 커밋 전의 base 에서 분기 → diff 에 이미 머지된 과거 커밋들이 포함됨

처리:
- 해당 기여자의 **신규 커밋만 cherry-pick** (저자 보존)
- PR 은 close + 설명 코멘트 ("이번 기여 2 커밋만 cherry-pick 반영했습니다")
- 중복 PR (같은 브랜치 main 타깃) 도 함께 close

### 11.3 대형 PR (>1000 라인)

- 즉시 admin merge 불가
- 코드 검토 + 사전 시뮬레이션 충분히 수행 후 결정
- 예: PR #165 (skia renderer · +100K 라인) — 장기 보류

## 12. 메모리 등록 항목 (자동 참조)

다음 상황은 `~/.claude/.../memory/` 에 등록되어 있다:

- `feedback_search_troubleshootings_first.md` — 작업 전 트러블슈팅 폴더 검색
- `feedback_external_docs_self_censor.md` — 외부 공개 문서 자기검열
- (신규 제안) `feedback_golden_regen_after_render_pr.md` — 렌더 PR 머지 후 golden 재생성

## 13. 참고 아카이브

- `mydocs/pr/archives/pr_234_review.md` — 재작업 요청 사례
- `mydocs/pr/archives/pr_235_review.md` · `pr_237_review.md` — 다양한 리뷰 패턴
- `mydocs/pr/archives/pr_251_review.md` — 재제출 후 머지 사례 (모든 피드백 반영)
