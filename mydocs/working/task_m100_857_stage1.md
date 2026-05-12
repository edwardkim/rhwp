# Task m100 #857 Stage 1 완료 보고서

> Issue: [#857](https://github.com/edwardkim/rhwp/issues/857)
> Stage 목표: 진단 산출물 commit (RED 캡처). 소스 코드 수정 없음.
> 작성일: 2026-05-12

## 1. 수행 결과

### 1.1 추가된 파일 (4개)
- `tests/issue_table_vpos_01_page5_cell_hit_test.rs` — RED 회귀 테스트 (13 케이스)
- `mydocs/troubleshootings/table_vpos_01_page5_cell_hit_test.md` — 진단 노트
- `mydocs/plans/task_m100_857.md` — 수행 계획서
- `mydocs/plans/task_m100_857_impl.md` — 구현 계획서 (옵션 D 확정본)

### 1.2 커밋
- 브랜치: `local/task857`
- Commit: `07168934 Task #857 Stage 1 (RED): 진단 노트 + 회귀 테스트 + 계획서 추가`
- 4 files changed, 890 insertions

### 1.3 사전 환경 정리
- `local/devel` 이 `devel` 보다 470 commit 뒤쳐져 있던 것 sync 완료 (Task #595/#685/#689 가 PR 머지 형태로 devel 에 이미 있어 손실 없음)
- `git reset --hard devel` 로 `local/devel` 및 `local/task857` 을 devel HEAD (`2bd50a3a`) 에 정렬

## 2. 검증

### 2.1 RED 테스트 상태 확인
```
$ cargo test --quiet --test issue_table_vpos_01_page5_cell_hit_test
test result: FAILED. 8 passed; 5 failed
```

**기대대로 5 FAIL / 8 PASS** — Stage 2 fix 가 PASS 시켜야 할 케이스 명확.

### FAIL 케이스 (5)
- `page5_inner_11x3_c2_row0_content_cell` — c=2 row=0 "국민 주도…" 클릭, cellPath 길이 1
- `page5_inner_11x3_c2_row1_content_cell` — c=2 row=1 "대국민 소통…"
- `page5_inner_11x3_c2_row3_content_cell` — c=2 row=3 "포용과 균형…"
- `page5_inner_11x3_c2_row6_content_cell` — c=2 row=6 "성과로 신뢰…"
- `page5_inner_11x3_c2_row0_insert_lands_in_inner_cell` — insert_text 가 inner 셀에 안 들어감 (사용자 증상 직접 재현)

### PASS 케이스 (8)
- pi=30 header c0/c1, pi=32 title, pi=34 inner 1x1 title (4)
- inner 11x3 c=0 라벨 row=0/3/6/9 (4)

## 3. Git Tree 상태

```
local/task857 ← 07168934 Task #857 Stage 1 (RED)
local/devel   ← 2bd50a3a (= devel)
devel         ← 2bd50a3a PR #818 처리 후속 (origin/devel)
```

## 4. 잔존 작업 (Stage 2-3 예정)

- Stage 2 (GREEN): cursor_rect.rs L648-666 first-match → min area best-match 변경 (옵션 D)
- Stage 3 (회귀 sweep): 전체 cargo test + clippy + SVG 시각 회귀 + rhwp-studio E2E + 최종 보고서

## 5. 작업지시자 승인 요청

본 Stage 1 완료. **Stage 2 (cursor_rect.rs 수정) 진행 승인** 부탁드립니다.
