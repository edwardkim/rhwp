# PR #611 처리 보고서 — 핀셋 cherry-pick 머지 + 시각 판정 ★ 통과 (3개 권위 샘플)

**PR**: [#611 fix: add undo history for non-text resize operations (closes #458)](https://github.com/edwardkim/rhwp/pull/611)
**작성자**: @kihyunnn (김기현, mable0927@gmail.com) — **첫 PR 컨트리뷰터**
**관련**: closes #458
**처리 결정**: ✅ **핀셋 cherry-pick 머지 + push + PR/Issue close + 시각 판정 자료 영역 영구 보존 (3개 권위 샘플)**
**처리일**: 2026-05-06

## 1. 처리 결과 요약

| 항목 | 결과 |
|------|------|
| 결정 | ✅ 핀셋 cherry-pick (`a4944d2b` 단독, rhwp-studio TS 5 파일 +128/-19) + 3개 권위 샘플 추가 + devel merge + push + PR/Issue close |
| 시각 판정 | ★ **통과** (작업지시자 web 환경 직접 시각 판정, 3개 권위 샘플) |
| Devel merge commit | `40d1da8` (본질 + 시각 판정 자료 영역 통합) |
| Cherry-pick commit (local/devel) | `5cdf8e5` |
| Cherry-pick 충돌 | 0 (auto-merge picture.ts 깨끗 통과) |
| Author 보존 | ✅ kihyunnn (mable0927@gmail.com) 보존 |
| PR #611 close | ✅ 한글 댓글 등록 + close |
| Issue #458 close | ✅ 수동 close (closes #458 키워드는 cherry-pick merge 로 자동 처리 안 됨, 안내 댓글 등록) |
| **권위 샘플 영구 보존** | ✅ `samples/calc-cell.hwp` / `samples/p122.hwp` / `samples/mix-shape-01.hwp` (158 → 161 직속 hwp) |
| 광범위 페이지네이션 sweep | 164 fixture / 1,684 페이지 / 회귀 0 (Rust 영역 변경 0 정합) |

## 2. 본질 결함 (Issue #458 인용)

> 표 크기를 실수로 수정하고 되돌리기를 하면 표는 원래대로 돌아오는게 아닌 무조건 텍스트만 뒤로가기 가능. (...) 이것은 text를 제외한 나머지(이미지, 표, 기타등등) 모두 포함입니다. 오직 text 만 되돌리기가 됩니다.

→ rhwp-studio editor 의 표 리사이즈 + 그림/도형 리사이즈 경로가 `CommandHistory` 우회 (WASM mutation 직접 호출) → Ctrl+Z / Ctrl+Y 미동작.

**크롬 확장 사용자분들이 많이 요청해주셨던 핵심 사용성 영역** — 외부 사용자가 직접 등록한 Issue #458 의 권위 본질을 첫 PR 컨트리뷰터 @kihyunnn 께서 정확히 정정해주신 사례.

## 3. 본질 정정

### 3.1 `command.ts` 신규 `ResizeObjectCommand` (+54 LOC)

```typescript
export type ObjectResizeTarget = {
  sec: number;
  ppi: number;
  ci: number;
  type: string;
  before: Record<string, unknown>;
  after: Record<string, unknown>;
};

export class ResizeObjectCommand implements EditCommand {
  readonly type = 'resizeObject';
  // setProps: shape/line/group → setShapeProperties / image → setPictureProperties 분기
  // execute / undo: before/after 속성 set
  // mergeWith(): null { return null; }
}
```

**before/after snapshot 영역**: `width/height/horzOffset/vertOffset` 변경 영역 모두 보존, `setShapeProperties` / `setPictureProperties` 분기 정합.

### 3.2 표 리사이즈 — `input-handler-table.ts` (+30/-12)

3 분기 (`finishResizeDrag` / `resizeCellByKeyboard` / `resizeTableProportional`) 모두 동일 패턴으로 통일:

```typescript
// Before
this.wasm.resizeTableCells(...)
this.eventBus.emit('document-changed');

// After
this.executeOperation({
  kind: 'snapshot',
  operationType: 'resizeTableCells',
  operation: (wasm: any) => {
    wasm.resizeTableCells(...);
    return this.cursor.getPosition();
  },
});
```

→ snapshot 기반 Undo 패턴 활용 — 표 리사이즈는 복합 셀 보상 변경으로 단일 속성 set 보다 snapshot 이 정합.

### 3.3 그림/도형 리사이즈 — `input-handler-picture.ts` (+40/-7)

**다중 선택 리사이즈** (`finishPictureResizeDrag` multi-resize):

```typescript
const historyTargets = [];
for (const r of state.multiRefs) {
  // ... 새 width/height/horzOffset/vertOffset 산출
  const updated: Record<string, unknown> = { width: newW, height: newH };
  const before: Record<string, unknown> = { width: r.origWidth, height: r.origHeight };
  if (deltaH !== 0) { updated['horzOffset'] = ...; before['horzOffset'] = r.origHorzOffset; }
  if (deltaV !== 0) { updated['vertOffset'] = ...; before['vertOffset'] = r.origVertOffset; }
  const changed = Object.keys(updated).some(key => updated[key] !== before[key]);
  if (!changed) continue;  // 무변경 record 차단 (Undo 스택 오염 방지)
  setObjectProperties.call(this, r, updated);
  historyTargets.push({ sec: r.sec, ppi: r.ppi, ci: r.ci, type: r.type, before, after: updated });
}
if (historyTargets.length > 0) {
  this.executeOperation({ kind: 'record', command: new ResizeObjectCommand(historyTargets) });
}
```

→ **changed 가드** (Object.keys 비교) + **다중 선택 모두 단일 ResizeObjectCommand 로 묶음** (Undo 1회로 LIFO 스택 순서 정합).

### 3.4 보조 변경

- `input-handler-mouse.ts` (+2): `onClick` 의 PictureResizeState 초기화 시 `origHorzOffset` / `origVertOffset` 추가
- `input-handler.ts` (+2): state interface 에 옵셔널 필드 추가

→ `ResizeObjectCommand` before snapshot 에 필요한 origin 값 보존.

### 3.5 회귀 위험 영역 (PR 본문 명시)

- 텍스트 undo 경로 변경 0
- 표/개체 이동 undo 경로 보존 (`MovePictureCommand` / `MoveShapeCommand` 무변경)
- 기존 `executeOperation` API 활용 — 신규 인프라 도입 0

## 4. 결정적 검증 결과

| 게이트 | 결과 |
|--------|------|
| `cargo build --release` | ✅ Finished |
| `cargo test --lib --release` | ✅ **1140 passed** / 0 failed (회귀 0) |
| `cargo clippy --release --lib` | ✅ 0건 |
| **rhwp-studio `npm run build`** (`tsc && vite build`) | ✅ TypeScript 타입 체크 통과 + dist 빌드 성공 (`dist/index.html` 55KB / `index-cbRqvVRC.js` 689KB) |
| Docker WASM 빌드 | ✅ **4,590,307 bytes** (1m 31s, PR #629 baseline 과 **정확 일치** — Rust 영역 변경 0 정합 정량 입증) |

## 5. 광범위 페이지네이션 회귀 sweep

| 통계 | 결과 |
|------|------|
| 총 fixture (cherry-pick 전) | **164** (158 hwp + 6 hwpx) |
| 총 페이지 (BEFORE) | **1,684** |
| 총 페이지 (AFTER) | **1,684** |
| **fixture 별 페이지 수 차이** | **0** |
| 총 fixture (3개 권위 샘플 추가 후) | **167** (161 hwp + 6 hwpx, +3 본 PR 시각 판정 자료 영역) |

→ rhwp-studio TypeScript 변경은 페이지네이션 (Rust 영역) 에 영향 없음 (자명).

## 6. WASM 정량 정합

| Baseline | bytes |
|---|---|
| PR #578 (5/6 첫 처리) | 4,583,156 |
| PR #629 (5/6 두 번째 처리) | **4,590,307** |
| PR #611 (5/6 세 번째 처리, 본 PR) | **4,590,307** ← **정확 일치** |

→ **본 PR 의 변경 영역 (rhwp-studio TypeScript) 이 Rust → WASM 산출물에 영향 0** 정량 입증. 본 사이클 첫 web editor 본질 정정 PR 의 영역 정합.

## 7. 시각 판정 (★ 게이트 — web 환경 3개 권위 샘플)

작업지시자 web 환경 시각 판정 결과:
> 모두 검증했습니다.

→ ★ **통과**.

### 7.1 권위 샘플 (본 PR 영구 보존, samples/ 직속 추가)

| 샘플 | 권위 영역 | 검증 결과 |
|------|---------|----------|
| **`samples/calc-cell.hwp`** (15,872 bytes) | 표 리사이즈 → Ctrl+Z | ✅ 정상 복원 |
| **`samples/p122.hwp`** (89,088 bytes) | 단일 이미지 리사이즈 → Ctrl+Z | ✅ 정상 복원 |
| **`samples/mix-shape-01.hwp`** (78,848 bytes) | 다중 이미지 리사이즈 → Ctrl+Z 스택 순서 | ✅ LIFO 정합 |

→ 본 사이클 (5/6) 에서 본 PR 의 시각 판정 자료 영역으로 **영구 보존** (158 → 161 직속 hwp, 본 PR 회귀 차단 가드 영역 영구화).

## 8. PR / Issue close 처리

### 8.1 PR #611 close
- 댓글 등록 (한글, cherry-pick 결과 + 시각 판정 ★ 통과 + 본질 정합 평가 + 후속 영역 안내)
- close 처리

### 8.2 Issue #458
- closes #458 키워드는 cherry-pick merge 로 자동 처리 안 됨 (PR #570/#629 등 동일 패턴) → 수동 close + 안내 댓글
- 후속 영역 (셀 분할/병합/삽입/삭제 등 "기타등등" 영역) 별도 후속 task 후보 명시

## 9. 메모리 정합

- ✅ `feedback_essential_fix_regression_risk` — 광범위 페이지네이션 sweep (164 fixture / 1,684 페이지) + 1140 passed 회귀 0
- ✅ `feedback_hancom_compat_specific_over_general` — 단일 영역 (rhwp-studio editor 리사이즈) 본질 정정, 다른 영역 무영향
- ✅ `feedback_rule_not_heuristic` — 기존 `executeOperation` API 재사용, 휴리스틱 미도입
- ✅ `feedback_visual_regression_grows` — web 환경 시각 판정 게이트 정합 운영 (3개 권위 샘플로 영구 보존, 향후 회귀 차단 가드)
- ✅ `feedback_pdf_not_authoritative` — PDF 미사용 (web editor 영역)
- ✅ `feedback_per_task_pr_branch` — Task #458 단일 본질 PR 정합
- ✅ `feedback_pr_comment_tone` — 첫 PR 컨트리뷰터 환영 + 차분/사실 중심 톤 (close 댓글에 "크롬 확장 사용자분들이 많이 요청해주셨던 기능을 첫 번째 기여로 구현해주셔서 감사합니다." 명시 — 작업지시자 안내 정합)
- ✅ `feedback_check_open_prs_first` — PR OPEN 상태 + 본 사이클 처리분 점검 후 진행
- ✅ `feedback_assign_issue_before_work` — Issue #458 assignee 미지정 (외부 사용자 등록 + 외부 컨트리뷰터 정정 자율 영역)
- ✅ `feedback_small_batch_release_strategy` — v0.7.10 후 세 번째 처리 PR 영역 정합

## 10. 본 PR 의 본질 — v0.7.10 후 세 번째 처리 PR (첫 web editor 영역)

본 PR 의 처리 본질에서 가장 우수한 점:

1. **첫 web editor 본질 정정 PR** — 본 사이클 (5/6) 의 첫 rhwp-studio TypeScript 영역 정정 (이전 PR #561~#600 + #578 + #629 모두 Rust 렌더러 영역). DTP 엔진 (`project_dtp_identity`) 의 web editor 영역 정합 진입.
2. **단일 영역 본질 정정** — rhwp-studio TypeScript 5 파일 (Rust 영역 0 변경) + 기존 `executeOperation` API 재사용 (신규 인프라 0)
3. **다중 선택 LIFO 스택 정합** — `historyTargets` 배열로 다중 그림 동시 리사이즈 후 Ctrl+Z 한 번으로 모두 복원, 작업지시자 시각 판정 (`mix-shape-01.hwp` 권위 샘플) 으로 정합 입증
4. **changed 가드** — `Object.keys(updated).some(...)` 로 무변경 record 차단 (Undo 스택 오염 방지)
5. **WASM 영향 0 정량 입증** — PR #629 baseline 과 정확 일치 (4,590,307 bytes), Rust 영역 변경 없음 정합 측정
6. **3개 권위 샘플 영구 보존** — 본 사이클 시각 판정 자료 영역으로 samples/ 직속 추가, 향후 회귀 차단 가드 영역으로 영구화 (작업지시자 직접 지정)
7. **첫 PR 컨트리뷰터 + 크롬 확장 사용자 사용성 영역** — 외부 사용자 권위 영역 (Issue #458) 의 핵심 본질 정확 정정

## 11. 본 사이클 사후 처리

- [x] PR #611 close (cherry-pick 머지 + push + 한글 댓글 — "크롬 확장 사용자분들이 많이 요청해주셨던 기능" 인정 명시)
- [x] Issue #458 close (수동 close + 안내 댓글)
- [x] 처리 보고서 (`mydocs/pr/archives/pr_611_report.md`, 본 문서)
- [x] 3개 권위 샘플 영구 보존 (`samples/calc-cell.hwp` / `samples/p122.hwp` / `samples/mix-shape-01.hwp`)
- [ ] 검토 보고서 archives 이동 (`mydocs/pr/pr_611_review.md` → `mydocs/pr/archives/pr_611_review.md`)
- [ ] 5/6 orders 갱신 (PR #611 항목 추가)

## 12. 후속 영역 (별도 task 후보)

Issue #458 의 "기타등등" 영역 중 본 PR 범위 외:
- 표 셀 분할/병합/삽입/삭제 등 표 구조 변경의 Undo
- 이미지 회전/뒤집기 등 이미지 변환 Undo
- 도형 색상/스타일 변경 Undo

→ 별도 후속 이슈로 분리 검토 예정 (사용자 사용성 영역 누적 정합).
