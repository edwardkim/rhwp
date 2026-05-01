# PR #256 구현 계획서

## PR 커밋 12개 (기여자 본인 + merge)

| # | SHA | 성격 |
|---|-----|------|
| 1 | bca5599 | 샘플 편입 + 비교 파이프라인 고정 |
| 2 | ffaf488 | Geometric Shapes(U+25A0-U+25FF) 전각 처리 |
| 3 | 8642159 | v2 계획서 + 단계2 보고서 |
| 4 | b38a126 | svg_snapshot golden + v2 결과보고서 |
| 5 | 10c36e2 | TAC 표 선행 텍스트 폭 반영 |
| 6 | a4175e5 | v3 계획서 + 단계4 보고서 |
| 7 | 6f9fc85 | Heavy display face 를 visual bold 로 |
| 8 | 9dfeb8b | v4 계획서 + 단계5 보고서 |
| 9 | 789c984 | 단계6 통합 검증 + v4 결과보고서 |
| 10~12 | 7f8cbbd · 09e9596 · e934ad5 | merge 커밋 (task146 → devel → origin/devel) |

## 사전 검증 결과 (완료)

| 항목 | 결과 |
|------|------|
| 자동 머지 충돌 | 1 파일 (`mydocs/orders/20260423.md`) |
| cargo test --lib | 947 passed / 0 failed |
| cargo clippy -- -D warnings | 0 warning |
| cargo test --test svg_snapshot | 3 passed |

## 단계 구성

### Stage 1 — 작업지시자 승인

본 문서 및 `pr_256_review.md` 검토 후 승인.

### Stage 2 — 로컬 머지 (충돌 해결 동반)

```bash
# 1. PR fetch 는 이미 완료 (local/pr256)
# 2. local/devel 로 전환
git checkout local/devel

# 3. PR 머지
git merge local/pr256 --no-ff -m "Merge pull request #256 from planet6897:devel

Task #146: text-align.hwp SVG ↔ 한컴 PDF 렌더링 일치
- Geometric Shapes U+25A0-U+25FF 전각 처리
- TAC 표 선행 텍스트 폭 inline x 좌표 반영
- Heavy display face visual bold 렌더

closes #146"

# 4. 충돌 해결 (mydocs/orders/20260423.md)
#    - 우리 orders 전체 유지
#    - 작업지시자 지시: 기여자 타스크만 우리 orders 에 간결하게 추가
git checkout --theirs mydocs/orders/20260423.md

# 5. orders 에 "9. Task #225 (기여자 보고)" + "10. PR #256 (@planet6897) Task #146" 섹션 추가
#    (간결 포맷, 기여자 상세는 mydocs/plans/task_m100_146_v4.md 로 링크)

# 6. 스테이지 + 커밋 완료
git add mydocs/orders/20260423.md
git commit
```

### Stage 3 — 최종 검증 (머지 후 상태에서 재확인)

```bash
cargo test --lib       # 947 passed 재확인
cargo clippy --lib -- -D warnings
cargo test --test svg_snapshot
```

### Stage 4 — devel sync + push

```bash
git checkout devel
git merge local/devel --ff-only
git push origin devel
```

### Stage 5 — GitHub 쪽 확인 + 수동 close

- PR #256 이 자동 `MERGED` 로 판정되는지 확인
- 안 되면 GitHub UI 에서 수동 "Close with comment" + 링크 커밋
- 이슈 #146 자동 close 안 될 경우 수동 close

### Stage 6 — 기여자 감사 + 리뷰 문서 archives

```bash
gh pr comment 256 --repo edwardkim/rhwp --body "..."
gh issue close 146 --repo edwardkim/rhwp --comment "..."

mv mydocs/pr/pr_256_review.md mydocs/pr/archives/
mv mydocs/pr/pr_256_review_impl.md mydocs/pr/archives/
```

### Stage 7 — 로컬 브랜치 정리

```bash
git branch -D local/pr256
```

## 충돌 해결 — orders 편집 가이드

작업지시자 지시: **"기여자 타스크만 우리쪽에 추가"**.

구체 편집 방안:

1. 우리 orders 의 "## 8. Firefox AMO 초기 제출" 뒤, "## 커밋 이력" 앞에 **두 섹션 추가**:

```markdown
## 9. Task #225 — MEMORY.md 중복 제거 (by @InsuJeong496 보고)

...

## 10. PR #256 — Task #146 텍스트 정렬 SVG↔PDF 일치 (by @planet6897)

### 배경
text-align.hwp 1페이지 SVG 출력이 한컴 PDF 와 시각적으로 달라 `mutool draw -F stext` 로 좌표 정밀 비교하여 원인 3가지 규명.

### 수정 3건
1. Geometric Shapes (U+25A0-U+25FF) 전각 처리 (`text_measurement.rs`)
2. TAC 표 선행 텍스트 폭 → inline x 좌표 반영 (`layout.rs`)
3. Heavy display face → visual bold (`style_resolver.rs` · `mod.rs` · `svg.rs`)

### 기각된 초기 가설 (진단 가치)
- Justify SVG 미반영 · Hanging indent 어긋남 — 실측 0.04~0.12 pt 오차로 정상 동작 확인

### 검증
- cargo test --lib: 947 passed (기여자 신규 6건 포함)
- svg_snapshot 3 passed (form-002 golden 갱신)
- clippy 0 warning
- 스모크 회귀 (exam_kor / biz_plan) 문제 없음

### 상세 문서 (기여자 작성)
- 수행계획서 v1~v4: mydocs/plans/task_m100_146{,_v2,_v3,_v4}{,_impl}.md
- 단계별 보고서: mydocs/working/task_m100_146_stage{1,2,4,5,6}.md
- 최종 결과보고서: mydocs/report/task_m100_146_report_v4.md
```

2. "## 감사" 섹션에 `@planet6897` + `@InsuJeong496` 추가

3. "## 커밋 이력" 에 새 머지 커밋 추가

## 위험 요소

| 위험 | 평가 |
|------|------|
| 자동 머지 충돌 | 1건 해결 완료 |
| 테스트 회귀 | 0 (947 pass) |
| Clippy 회귀 | 0 |
| 샘플 파일 크기 | `samples/text-align.hwp` 추가 (저장소 크기 미미 증가) |
| Base = devel, head = devel | 실질적 문제 없음. 다음 기여 때 feature 브랜치 권장 코멘트 |
| 기여자 후속 대응 | 낮음 (기여자가 이미 체계적 작업 완료) |

## 예상 소요

작업지시자 승인 후 15~20분 (orders 편집 포함).
