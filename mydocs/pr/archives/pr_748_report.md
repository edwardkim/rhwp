---
PR: #748
제목: Task #158 — 표 크기 조절 Undo/Redo (SnapshotCommand 적용)
컨트리뷰터: @oksure (Hyunwoo Park) — 20+ 사이클 핵심 컨트리뷰터 (5/10 사이클 16번째 PR)
처리: 옵션 (b) — cherry-pick + empty commit (PR #728 supersede 흡수)
처리일: 2026-05-10
머지 commit: 5ce0bcc9
---

# PR #748 처리 보고서

## 1. 처리 결과

✅ **머지 완료** — 옵션 (b) cherry-pick + empty commit

| 항목 | 값 |
|------|-----|
| 머지 commit | `5ce0bcc9` (--no-ff merge) |
| Cherry-pick commits | `312fc4c5` + `d20a6a45` (양쪽 모두 empty, --allow-empty) |
| 원본 commits | `70cc0209` + `a15336d2` |
| closes | #158 |
| 판정 | 본질 정정 PR #728 흡수 — 커밋 이력 + author 보존 |
| 시각 판정 | ✅ 작업지시자 웹 에디터 인터랙션 검증 통과 |
| 자기 검증 | tsc + cargo test ALL GREEN + sweep 170/170 same + WASM 4.66 MB |

## 2. 본질 — PR #728 supersede 흡수

PR #728 (closes #204, 5/9 머지) 가 **표 편집 11 커맨드 + 표 크기 조절 3 함수** 까지 모두 SnapshotCommand 패턴을 적용한 상태:

| 함수 | PR #728 (HEAD) | PR #748 incoming |
|------|----------------|-------------------|
| `finishResizeDrag` | ✅ SnapshotCommand + try/catch | ✅ 동일 (operationType 명칭 차이) |
| `resizeCellByKeyboard` | ✅ 동일 | ✅ 동일 |
| `resizeTableProportional` | ✅ 동일 | ✅ 동일 |

**비교**:
- operationType 명칭: PR #728 (`resizeTableCells`, `resizeCellByKeyboard`, `resizeTableProportional`) 더 구체적
- document-changed emit: PR #728 측 afterEdit 자동, PR #748 incoming 은 수동 (Copilot 리뷰 후 일부 영역 잔존)
- try/catch: 양측 모두 적용

**3 영역 충돌 수동 해결** 시도 결과 HEAD 측 유지 시 변경 부재 → PR #748 의 정정 본질이 이미 devel 영역 반영 확인.

## 3. 처리 방식 — 옵션 (b) cherry-pick + empty commit

작업지시자 결정: 옵션 (b) — cherry-pick 으로 컨트리뷰터 commit 이력 + author 정보 보존.

```bash
git cherry-pick 70cc0209 a15336d2  # CONFLICT 3 영역
# 수동 해결 → HEAD 측 채택 → 변경 부재
git add rhwp-studio/src/engine/input-handler-table.ts
git commit --allow-empty -C 70cc0209  # 312fc4c5
git cherry-pick --continue --allow-empty  # 두번째 commit 도 충돌
# 동일 패턴 수동 해결
git commit --allow-empty -C a15336d2  # d20a6a45
git checkout devel
git merge local/devel --no-ff -m "Merge PR #748 (Task #158): ..."  # 5ce0bcc9
```

## 4. Copilot 리뷰 반영 (commit `a15336d2`)
- 중복 `document-changed` 이벤트 발행 제거
- `try/catch` 추가 (안전성)

PR #728 측이 동일 의도 (afterEdit 자동 emit + try/catch) 를 이미 반영했음.

## 5. 본 환경 검증

| 검증 | 결과 |
|------|------|
| `cherry-pick` 충돌 (3 영역) | 수동 해결 — HEAD 측 채택 |
| `tsc --noEmit` (rhwp-studio) | ✅ 통과 |
| `cargo test --release` | ✅ ALL GREEN |
| 광범위 sweep (7 fixture / 170 페이지) | ✅ **170 same / 0 diff** |
| WASM 빌드 (Docker) | ✅ 4.66 MB |

## 6. 작업지시자 웹 에디터 인터랙션 검증 ✅ 통과
3가지 표 크기 조절 경로 + Ctrl+Z Undo 정합 동작:
- 마우스 드래그 리사이즈 → Ctrl+Z
- 키보드 셀 리사이즈 → Ctrl+Z
- Ctrl+방향키 비율 리사이즈 → Ctrl+Z

> 본질 정정이 PR #728 머지로 이미 devel 반영된 상태 — 검증은 PR #728 의 표 Undo/Redo 동작이 회귀 없이 유지됨을 확인.

## 7. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure **20+ 사이클** (5/10 사이클 16번째 PR) |
| `feedback_image_renderer_paths_separate` | TypeScript 단일 파일 — Rust 렌더링 경로 무영향 |
| `feedback_process_must_follow` | 인프라 재사용 (PR #728 SnapshotCommand 패턴) — 신규 인프라 도입 부재 |
| `feedback_pr_supersede_chain` (a + b) | PR #728 이 PR #748 본질 흡수 (a 패턴 + (b) 옵션 cherry-pick 으로 author 보존) |
| `feedback_visual_judgment_authority` | 작업지시자 웹 에디터 인터랙션 검증 ✅ 통과 |

## 8. 영향 범위

### 8.1 변경 영역
- `rhwp-studio/src/engine/input-handler-table.ts` — 변경 부재 (PR #728 흡수)

### 8.2 무변경 영역
- WASM 코어 (Rust) — 변경 부재
- HWP3/HWPX 변환본 시각 정합 (sweep 170/170 same 입증)

## 9. 잔존 후속

- 본 PR 본질 정정의 잔존 결함 부재
- Issue #158 close 완료
- PR #728 의 11 표 커맨드 + 3 크기 조절 함수 SnapshotCommand 정합 운영 중

---

작성: 2026-05-10
