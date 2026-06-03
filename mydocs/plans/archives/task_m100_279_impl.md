# Task #279 구현계획서

상위: [task_m100_279.md](task_m100_279.md)
브랜치: `local/task279` (← origin/devel)
이슈: [#279](https://github.com/edwardkim/rhwp/issues/279)
원본 PR: [#282](https://github.com/edwardkim/rhwp/pull/282) ([@seanshin](https://github.com/seanshin))

## 사전 상태

- `local/task279` 브랜치 origin/devel(`f035538`) 기준 생성 완료
- 작성자 fork branch `pr282-task279` 로컬 fetch 완료 — 핵심 3 커밋 식별
- 인수 안내 코멘트 PR #282 게시 완료 ([comment-4318148845](https://github.com/edwardkim/rhwp/pull/282#issuecomment-4318148845))

## Stage 1 — 작성자 핵심 3 커밋 cherry-pick

### 작업

`local/task279` 브랜치에 다음 순서로 cherry-pick (전부 메인테이너 server-side merge 없음 — 단순 cherry-pick):

```bash
# 1) 수행계획서 (이미 메인테이너 버전 작성됨 → conflict 시 ours 채택)
git cherry-pick 5d1c80f
# → conflict 발생 시: git checkout --ours mydocs/plans/task_m100_279.md
#   본 메인테이너 버전이 강화된 기여 인정 포함하므로 우선
#   git cherry-pick --continue (author 정보 hyoun mouk shin 유지)

# 2) 코드 변경 (3 파일)
git cherry-pick d48af5c
# → conflict 없을 것으로 기대. svg.rs / web_canvas.rs / text_measurement.rs 모두
#   본 task 와 충돌하지 않는 origin/devel 영역.

# 3) Stage 3+4 보고서 (작성자 stage3 만 존재)
git cherry-pick 76436df
# → 76436df 가 task_m100_279_stage3.md 추가. 그대로 채택.
#   stage4 없음 (작성자 PR 미완) → 메인테이너가 stage4 + 최종 보고서 신규 작성.
```

### Co-Authored-By 처리

cherry-pick 은 commit author 만 보존 (hyoun mouk shin). committer 는 메인테이너로 변경됨. trailer 는 원본 메시지의 `Co-Authored-By: Claude Sonnet 4.6` 가 그대로 유지됨. 본 인수 작업의 `Co-Authored-By: Claude Opus 4.7 (1M context)` 추가는 **메인테이너 신규 커밋 (Stage 2 보고서, 최종 보고서)** 에서만 적용.

### 검증

```bash
git log --oneline origin/devel..HEAD
# 기대 (3 커밋, author=hyoun mouk shin):
#   76436df docs: Task #279 Stage 3+4 완료 보고서
#   d48af5c fix: Task #279 Stage 2+3 — 목차 리더 도트 + 소제목 탭 정렬
#   5d1c80f docs: Task #279 수행계획서 — ...

git log --pretty=format:'%h %an' origin/devel..HEAD
# 기대: 모두 hyoun mouk shin
```

### 산출물

- 코드: `src/renderer/svg.rs`, `src/renderer/web_canvas.rs`, `src/renderer/layout/text_measurement.rs` (3 파일, +12/-5)
- 문서: `mydocs/plans/task_m100_279.md` (메인테이너 강화 버전), `mydocs/working/task_m100_279_stage3.md` (작성자 원본)

### 완료 조건

- 3 커밋이 cherry-pick 되어 author=hyoun mouk shin 보존
- working tree clean
- conflict 해결 시 `5d1c80f` 의 task_m100_279.md 는 ours (메인테이너 버전)

---

## Stage 2 — 빌드 + 단위/통합 테스트 + clippy + wasm32

### 작업

1. `cargo build --release` — 정상 빌드 확인 (코드 변경 작아 ~25s)
2. `cargo test --lib` — 992+ passed / 0 failed 확인
3. `cargo test --test svg_snapshot` — 6 골든 영향 분석:
   - 무회귀 → 통과
   - 영향 발생 시 영향 페이지 식별 → 의도된 변경 (KTX 목차) 인지 분석:
     - 의도 → `UPDATE_GOLDEN=1 cargo test --test svg_snapshot` 으로 갱신
     - 비의도 → 원인 추가 분석 (text_measurement 의 right tab 외 영역 영향 가능성)
4. `cargo clippy --lib -- -D warnings` — clean 확인
5. `cargo check --target wasm32-unknown-unknown --lib` — clean 확인
6. (회귀 가드) `cargo test --test issue_301` (z-table) + 다른 기존 통합 테스트 영향 확인

### Co-Authored-By 적용 시점

본 stage 의 Stage 2 보고서 (`mydocs/working/task_m100_279_stage2.md`) 커밋 메시지에:

```
Task #279: Stage 2 - 빌드/테스트 검증

Co-Authored-By: hyoun mouk shin <hyounmoukshin@hyounui-MacBookPro.local>
Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
```

### 산출물

- `mydocs/working/task_m100_279_stage2.md` 신규 — 빌드/테스트 결과 표

### 완료 조건

- 모든 자동 검증 통과 (또는 골든 영향이 의도된 KTX 목차 영역으로 한정되어 UPDATE_GOLDEN 처리)
- `mydocs/working/task_m100_279_stage2.md` 작성

---

## Stage 3 — 시각 검증 + 좌표 측정 + 회귀 샘플

### 3.1 KTX.hwp 좌표 측정

```bash
./target/release/rhwp export-svg samples/basic/KTX.hwp -o output/debug/task279_ktx/
```

측정 항목 (작성자 stage3 보고서와 동일 검증, 메인테이너가 재실측):

| 항목 | Before (devel) | After (PR) | 기대 |
|------|----------------|------------|------|
| 장제목 페이지번호 우측 edge x | 717.5 | (측정) | 무변화 |
| 소제목 페이지번호 우측 edge x | ~700 | (측정) | 717.5±0.5 |
| 리더 dasharray attribute | "1 2" | "0.1 3" | 변경 확인 |
| 리더 stroke-linecap | (없음) | "round" | 추가 확인 |

좌표 측정은 SVG grep:
```bash
grep -E '<line.*stroke-dasharray="0.1 3"' output/debug/task279_ktx/*.svg | head
grep -oE 'x="[0-9.]+">[Ⅰ-Ⅴ0-9. ]+</text>' output/debug/task279_ktx/*.svg
```

### 3.2 6 핵심 샘플 회귀

| 샘플 | 기대 페이지 수 |
|------|----------------|
| 21_언어_기출_편집가능본.hwp | 15 |
| exam_math.hwp | 20 |
| exam_kor.hwp | 24 |
| exam_eng.hwp | 9 |
| basic/KTX.hwp | 1 |
| aift.hwp | 74 |
| biz_plan.hwp | 6 |

회귀 발생 시 영향 분석. 본 task 는 right tab + dasharray 만 변경하므로 페이지 수 영향 0 기대.

### 3.3 WASM Docker 빌드 + 브라우저 시각 확인

```bash
docker compose --env-file .env.docker run --rm wasm
```

작업지시자 직접 시각 확인:
- KTX.hwp 목차 페이지 리더 점선이 원형 점으로 보이는지
- 소제목 페이지번호가 장제목과 동일 우측 정렬되어 보이는지
- 다른 샘플에서 회귀 없는지

### Co-Authored-By 적용

`mydocs/working/task_m100_279_stage3_v2.md` (또는 stage3 보강) 커밋 메시지에 동일 trailer.

### 산출물

- `output/debug/task279_ktx/` SVG (디버그 출력 폴더 규칙 준수)
- `mydocs/working/task_m100_279_stage3_v2.md` (메인테이너 보강 버전, 작성자 stage3.md 인용 + 추가 측정)

### 완료 조건

- 좌표 측정 결과 기대값 부합
- 6 핵심 샘플 회귀 0
- WASM 시각 확인 통과

---

## Stage 4 — 최종 보고서 + 기여 인정 산출물 + PR force-push + admin merge

### 4.1 최종 보고서 작성

`mydocs/report/task_m100_279_report.md`:

- 머리말: "원본 분석·구현: [@seanshin](https://github.com/seanshin) — PR [#282](https://github.com/edwardkim/rhwp/pull/282)" 명시
- 결론 / 처리 절차 / 근본 원인 / 변경 내역 / 검증 결과 / 기여 인정 섹션 포함
- 작성자 stage3 보고서의 좌표 측정 표를 인용으로 보존 + 메인테이너 재측정 결과 병기

### 4.2 CHANGELOG.md 항목 추가

다음 미릴리즈 섹션 (또는 적절한 위치) 에:

```markdown
### Fixed

- 목차 right tab 리더 도트 렌더링 + 소제목 페이지번호 정렬 (#279)
  — 분석·구현 by [@seanshin](https://github.com/seanshin)
```

CHANGELOG 업데이트는 메인테이너 commit + Co-Authored-By trailer.

### 4.3 위키 페이지 등재

`rhwp.wiki/` 에 다음 결정:
- 옵션 a: `HWP-Tab-Leader-Rendering.md` 신규 — 리더 fill_type 별 SVG/Canvas 표현 가이드. 본인 크레딧 머리말
- 옵션 b: 기존 `Home.md` Documentation 색인의 "외부 기여자 분석" 섹션 갱신

작업지시자에게 옵션 의견 받음 (Stage 4 진행 시점). default 는 옵션 a (#265 / #309 선례에 가장 부합).

### 4.4 HWP Spec Errata 결정

`mydocs/tech/hwp_spec_errata.md` 에 다음 entry 추가 가치 판단:

> **TabStop.position 은 column-relative 절대 좌표 — 들여쓰기 문단의 right tab 클램핑 금지**
>
> ParaShape.left_margin 으로 들여쓰기된 문단의 right tab(tab_type=1) 도 단(column) 우측 끝 기준 절대 위치를 유지해야 한다. text_measurement 의 available_width 클램핑은 left/center 탭(0/2) 만 대상.
>
> 발견·구현: [@seanshin](https://github.com/seanshin) (Shin hyoun mouk) — #279, PR #282
> 검증: 메인테이너

errata 가치 있음 (HWP 스펙에 명시 부족 + 다른 구현체에서도 동일 함정 가능성). Stage 4 에서 entry 추가.

### 4.5 orders 갱신

`mydocs/orders/20260425.md` 에 Task #279 섹션 추가 (배경/원인/변경/검증/상태).

### 4.6 작성자 fork force-push

```bash
git push https://github.com/seanshin/rhwp.git local/task279:feature/task279-toc-leader-tab --force-with-lease
```

`maintainerCanModify=true` 활용. PR #282 의 head 가 갱신되어 commits 탭에 정리된 3 (작성자) + 4 (메인테이너) 커밋 표시.

### 4.7 PR 검증 + admin merge

- CI 통과 확인 (Build & Test, CodeQL)
- `gh pr review 282 --approve`
- `gh pr merge 282 --admin --merge` (작성자 author 보존을 위해 squash 미사용)
- `gh issue close 279`
- 머지 후 PR 코멘트: 머지 완료 + 작성자 감사 + 머지 commit 링크
- 이슈 close 코멘트: 분석·구현 본인 + PR 정리·검증 메인테이너 명기

### 4.8 PR archive

- `mydocs/pr/pr_282_review.md` (인수 시점 review 보강 — 현재 archives 의 close 보고서를 정정)
- `mydocs/pr/pr_282_report.md` 신규 (인수·머지 결과 보고서)
- `mydocs/pr/archives/pr_282_report.md` (이미 존재, close-only 결정 시점) → 삭제 또는 정정 후 보존

### 4.9 local/devel + devel 동기화

- `local/task279` → `local/devel` merge
- `local/devel` → `devel` merge + push (메인테이너 워크플로우)
- 또는 PR #282 admin merge 자체가 devel 갱신을 처리. local/devel sync.

### Co-Authored-By 적용

본 stage 의 모든 메인테이너 신규 commit 에 trailer:
```
Co-Authored-By: hyoun mouk shin <hyounmoukshin@hyounui-MacBookPro.local>
Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
```

### 산출물

- `mydocs/report/task_m100_279_report.md`
- `CHANGELOG.md` (항목 추가)
- `rhwp.wiki/HWP-Tab-Leader-Rendering.md` (또는 Home.md 갱신)
- `mydocs/tech/hwp_spec_errata.md` (entry 추가)
- `mydocs/orders/20260425.md` (Task #279 섹션)
- `mydocs/pr/pr_282_report.md` 신규 (또는 archives 정정)

### 완료 조건

- PR #282 admin merge 성공
- 이슈 #279 close
- 모든 기여 인정 7 항목 산출물 생성

---

## 커밋 스킴

| Stage | 커밋 메시지 패턴 | author |
|-------|------------------|--------|
| Stage 1 | (cherry-pick 그대로, 5d1c80f / d48af5c / 76436df) | hyoun mouk shin |
| Stage 2 | `Task #279: Stage 2 - 빌드/테스트 검증` | edwardkim (Co-Authored-By: hyoun mouk shin) |
| Stage 3 | `Task #279: Stage 3 - 좌표 측정 + 회귀 검증` | edwardkim (Co-Authored-By: hyoun mouk shin) |
| Stage 4-1 | `Task #279: Stage 4 - 최종 보고서 + CHANGELOG` | edwardkim (Co-Authored-By: hyoun mouk shin) |
| Stage 4-2 (wiki) | `wiki: HWP Tab Leader Rendering 가이드 (#279)` | edwardkim (Co-Authored-By) — 위키 별도 repo |
| Stage 4-3 (errata) | `docs: HWP Spec Errata 항목 추가 (#279)` | edwardkim (Co-Authored-By) |
| Stage 4-4 (orders/pr) | `docs(pr): PR #282 처리 완료 - Task #279 인수 머지` | edwardkim (Co-Authored-By) |

각 stage 완료 시 작업지시자 승인 받고 다음 stage 진행 (CLAUDE.md 절차 준수).

## 리스크와 대응

| 리스크 | 대응 |
|--------|------|
| svg_snapshot 골든 영향 (KTX 외 페이지) | Stage 2 에서 영향 페이지 식별. KTX 목차 한정이면 UPDATE_GOLDEN. 그 외면 stage 일시 중단 + 작업지시자 보고 |
| right tab 클램핑 제외로 다른 샘플 회귀 | Stage 3 의 6 샘플 회귀 검증으로 차단. 영향 발견 시 가드 정밀화 (예: `available_width > 0 && ts.tab_type != 1` 외 추가 조건) 검토 |
| Canvas line_cap restore 누락 | seanshin 코드에 `set_line_cap("butt")` 복원 있음. Stage 3 의 WASM 시각 검증으로 후속 leader 영향 0 확인 |
| force-push 시점 작성자 추가 push | `--force-with-lease` 사용. 거부되면 작성자 신규 커밋 검토 후 진행 결정 |
| #282 가 base BEHIND 상태 | local/task279 가 origin/devel 기반이므로 force-push 후 mergeable 자동 갱신 기대 |
