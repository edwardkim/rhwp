# PR #251 — 구현 계획서

## PR 커밋 5개

| # | SHA | 내용 |
|---|-----|------|
| 1 | 8e5183c | fix: PUA 심볼 문자 렌더링 — SVG/Canvas/HTML 세 렌더러에 일관 적용 (#249) |
| 2 | 7e2c6eb | fix: 문단 border_fill — margin_left/margin_right 반영 (#249) |
| 3 | 3f0538d | fix: 표 외곽 테두리 — border_fill_id fallback + 셀 미커버 영역 한정 (#249) |
| 4 | cd8dd8e | docs: Task #249 수행/구현/결과 계획서 + Task #250 수행계획서 추가 |
| 5 | 7d4c311 | docs: Task #249 단계별 완료보고서 추가 (stage 1~3) |

논리 단위가 깔끔 (기능 3건 + 문서 2건). Rebase 없이 그대로 수용 가능.

## 사전 검증 결과 (완료)

로컬 실험 브랜치 `pr251-merge-test` = `local/pr251` + `origin/devel` 머지:

- 자동 머지: **충돌 0건**
- `cargo build --lib`: 성공
- `cargo test --lib`: **941 passed**, 0 failed, 1 ignored
- `cargo clippy -- -D warnings`: **0 warning**
- `cargo test --doc`: 컴파일 에러 없음

→ admin merge 즉시 가능한 상태.

## 단계 구성

### Stage 1 — 작업지시자 승인

- `pr_251_review.md` 검토
- admin merge 승인 요청
- 기여자 코멘트 방침 (예: "재제출 피드백 반영 감사" 톤) 확인

### Stage 2 — admin merge

```
gh pr merge 251 --repo edwardkim/rhwp --merge --admin
```

- BEHIND 상태이나 자동 머지 클린 → admin 플래그로 통과
- 동일 패턴: 최근 사이클 PR #215, #221, #224 등

### Stage 3 — devel sync

```
git fetch origin
git checkout local/devel
git rebase origin/devel   # 머지 커밋 흡수
```

### Stage 4 — 이슈 #249 close 확인

- PR body 의 closes 표기 여부 확인 후 수동 close (이전 사이클 경험상 GitHub auto-close 실패 사례 있음)

### Stage 5 — 이슈 #250 후속 처리

- Right Tab 이슈 #250 은 이번 사이클 범위 외
- @seanshin 이 후속 PR 작성 중일 가능성 → 관찰만, 별도 사이클에서 처리

### Stage 6 — 로컬 브랜치 정리

- `local/pr251` 삭제
- 피드백 문서 `mydocs/pr/pr_251_review.md` + `pr_251_review_impl.md` 를 archives 로 이동 (기존 패턴)

## 작업지시자 확인 필요 사항

| 항목 | 제안 |
|------|------|
| Merge 방식 | admin merge (기본) |
| 기여자 코멘트 | 영어 또는 한국어. 재제출 피드백 반영 감사 표현. @postmelee / @planet6897 / @bapdodi 사례 참고 |
| 이슈 #249 close | merge 커밋 후 수동 확인 |
| Right Tab #250 | 후속 PR 기다림 (현 사이클 범위 외) |
| 본 문서 archives 이동 | merge 완료 후 수행 |

## 예상 소요

작업지시자 승인 후 5~10분 내 전체 완료 (이전 PR #215, #221 패턴 기준).

## 위험 요소 요약

| 위험 | 평가 |
|------|------|
| 충돌 | 0 (검증 완료) |
| 테스트 회귀 | 0 (941 pass) |
| Clippy 회귀 | 0 |
| 한컴 호환 회귀 | 낮음 (시각 수정만, IR 변경 없음) 단 Visual Diff 파이프라인 자체는 PR 에 없으므로 한컴 2020 샘플 수동 검증 **선택적** 권장 |
| 기여자 후속 응답 | 낮음 (이미 체계적으로 재제출) |
