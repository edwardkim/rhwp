---
PR: #738
제목: Task #144 — 수식 편집 UI 개선 (듀얼 모드, 자동완성, 탭 템플릿, 기호 검색)
컨트리뷰터: @oksure (Hyunwoo Park) — 20+ 사이클 핵심 컨트리뷰터 (5/10 사이클 영역 영역 7번째 PR)
base / head: devel / contrib/equation-editor-ux
mergeStateStatus: BEHIND
mergeable: MERGEABLE — `git merge-tree` 충돌 0건
CI: ALL SUCCESS (Build & Test / Canvas visual diff / CodeQL / Analyze rust+js+py / WASM SKIPPED)
변경 규모: +601 / -98, 2 files (rhwp-studio TypeScript + CSS)
검토일: 2026-05-10
PR_supersede: PR #738 영역 영역 PR #765 (closes #763) 영역 영역 supersede — (a) 패턴 정합
---

# PR #738 검토

## 1. 메타

| 항목 | 값 |
|------|-----|
| PR 번호 | #738 |
| 제목 | Task #144 — 수식 편집 UI 개선 (듀얼 모드, 자동완성, 탭 템플릿, 기호 검색) |
| 컨트리뷰터 | @oksure — 20+ 사이클 (5/10 사이클 영역 영역 PR #728/#729/#730/#734/#735/#737 후속 영역 7번째) |
| base / head | devel / contrib/equation-editor-ux |
| mergeStateStatus | BEHIND, mergeable: MERGEABLE — 충돌 0건 |
| CI | ALL SUCCESS (Canvas visual diff 포함) |
| 변경 규모 | +601 / -98, 2 files |
| 커밋 수 | 2 (Task 1 + Copilot 리뷰 반영 1) |
| closes | #144 |

## 2. ⚠️ PR supersede 관계 — PR #738 supersedes PR #765

### 2.1 발견 영역
PR #765 (closes Issue #763, 5/9 19:35 등록, @oksure) 영역 영역 본 PR #738 (5/9 11:51 등록, @oksure) 영역 영역 후속 영역 영역 — 동일 두 파일 영역 영역 작은 부분 변경 영역 영역.

### 2.2 PR #738 vs PR #765 비교

| 항목 | PR #738 (5/9 11:51) | PR #765 (5/9 19:35) |
|------|---------------------|---------------------|
| Issue | #144 (4/14 OPEN) | #763 (5/9 18:00 OPEN, 작업지시자 등록) |
| 변경 규모 | +601/-98, 2 files | +116/-18, 2 files |
| 듀얼 모드 토글 | ✅ HWP/LaTeX | ✅ hwpeq/LaTeX |
| 모드 단축키 | (점검 영역 영역) | Ctrl+M |
| 템플릿 수 | 130+ (탭 7개) | 30개 |
| 자동완성 | ✅ | (부재) |
| 기호 검색 | ✅ | (부재) |
| LaTeX 전환 힌트 | ✅ | (부재) |

PR #738 영역 영역 PR #765 영역 영역 의 본질 (모드 토글 UI + 모드별 템플릿) 영역 영역 **완전 포함** 영역 영역 → PR #765 영역 영역 의 의도 영역 영역 PR #738 영역 영역 통합 영역 영역.

### 2.3 처리 결정 영역 영역 (`feedback_pr_supersede_chain` (a) 패턴 정합)

PR #738 머지 후 영역 영역 PR #765 close + Issue #763 close (PR #738 영역 영역 통합 영역 영역). 동일 컨트리뷰터 영역 영역 의 (a) 패턴 정합 — 작업지시자 결정 권장.

### 2.4 작업지시자 답글 영역 영역 정합 점검
PR #729 영역 영역 의 답글 (작업지시자):
> "듀얼모드 구현을 위한 UI 명시적 모드 토글은 별도 이슈로 등록하겠습니다."

→ Issue #763 등록. 그러나 컨트리뷰터 영역 영역 영역 PR #738 영역 영역 (Issue #144 영역 영역 4월 14일 영역 영역 OPEN, 더 큰 본질) 영역 영역 영역 작업 영역 영역 — Issue #763 영역 영역 의 의도 영역 영역 본 PR 영역 영역 정합 영역 영역.

## 3. 결함 본질 (Issue #144)

### 3.1 목표
- 수식 편집 대화상자 영역 영역 한컴/LaTeX 듀얼 입력 모드 지원
- 명령어 자동완성 + 템플릿 확장 + 기호 검색 추가

### 3.2 채택 접근
PR #143 (LaTeX 파서 확장) 영역 영역 시너지 영역 영역 — Rust 수식 파서 영역 영역 이미 양 구문 처리 영역 영역 백엔드 변경 부재.

## 4. PR 의 정정 — 2 영역

### 4.1 `rhwp-studio/src/ui/equation-editor-dialog.ts` (+447/-91)

**1. HWP ↔ LaTeX 듀얼 입력 모드**:
```typescript
type InputMode = 'hwp' | 'latex';
private mode: InputMode = 'hwp';

private setMode(m: InputMode): void { ... }
private toggleMode(): void {
    this.setMode(this.mode === 'hwp' ? 'latex' : 'hwp');
}
```
- 타이틀바 영역 영역 모드 토글 버튼 (HWP/LaTeX)
- 모드별 템플릿 스크립트 전환
- 기존 스크립트 영역 영역 `\명령어` 존재 시 LaTeX 모드 영역 영역 자동 시작

**2. 탭 기반 확장 템플릿** (30 → 130+ 항목):
- 7개 탭: 구조, 그리스, 연산자, 화살표, 함수, 장식, 특수
- `symbols.rs` 명령어 체계 정합 (PR #729 영역 영역 LaTeX 호환 영역 영역 정합)

```typescript
const TEMPLATE_GROUPS: TemplateGroup[] = [
    { id: 'struct', name: '구조', items: [
        { label: '분수', hwp: '{} over {}', latex: '\\frac{}{}' },
        { label: '√', hwp: 'sqrt {}', latex: '\\sqrt{}' },
        ... // 19+ items
    ] },
    ... // 6 more groups
];
```

**3. 명령어 자동완성**:
- 2자 이상 입력 시 드롭다운 활성화
- ↑↓ 키 영역 영역 선택 / Tab/Enter 영역 영역 확정 / Esc 영역 영역 닫기
- 기호 미리보기 (α) + 명령어 이름 (alpha) + 그룹명 (그리스) 동시 표시

**4. 기호 검색**:
- 이름 또는 유니코드 문자 영역 영역 전체 명령어 검색
- 검색 결과 클릭 시 커서 위치 영역 영역 삽입

**5. LaTeX 전환 힌트**:
- HWP 모드 영역 영역 `\` + 영문자 입력 감지 영역 영역 LaTeX 전환 안내 배너

### 4.2 `rhwp-studio/src/styles/dialogs.css` (+154/-7)

수식 편집 대화상자 영역 영역 신규 UI 요소 영역 영역 CSS 영역 — 모드 토글 버튼 / 탭 / 자동완성 드롭다운 / 검색 / LaTeX 힌트 배너.

### 4.3 Copilot 리뷰 반영 (commit `1c45a840`)
- LaTeX 모드 자동완성 정정 — 모드별 insert 분리
- 빈 name 방지
- 백슬래시 중복 제거

## 5. 의존성 점검

| 의존성 | 상태 |
|--------|------|
| PR #143 (LaTeX 파서) | ✅ PR #729 (closes #143) 머지 commit `eb3f9fd4` |
| Rust 수식 파서 영역 영역 양 구문 처리 | ✅ (PR #729 영역 영역 LaTeX 호환 41 테스트 PASS) |
| 백엔드 변경 부재 | ✅ TypeScript + CSS 만 변경 |

## 6. 본 환경 점검

- merge-base: `c9dd6f9c` (5/9 영역 영역 가까움)
- merge-tree 충돌: **0건** ✓
- 변경 영역 영역 격리: rhwp-studio frontend 영역 영역 만 — Rust / 다른 layout/render 경로 영역 영역 무관
- WASM 변경 부재 (백엔드 미수정 영역 영역 의 결과)

## 7. 영향 범위

### 7.1 변경 영역
- rhwp-studio 수식 편집 대화상자 영역 영역 UX 전면 개선
  - 듀얼 모드 토글
  - 자동완성 드롭다운
  - 탭 기반 130+ 템플릿
  - 기호 검색
  - LaTeX 전환 힌트

### 7.2 무변경 영역
- Rust 수식 파서 (PR #729 영역 영역 영역 호환)
- WASM 빌드 영역 영역 영역 변경 부재
- HWP3/HWPX 변환본 영역 영역 시각 정합

### 7.3 위험 영역
- TypeScript 변경 영역 영역 격리 — 다른 rhwp-studio 모듈 영역 영역 영향 부재
- PR #765 영역 영역 supersede 결정 영역 영역 작업지시자 권장

## 8. 충돌 / mergeable

- `git merge-tree --write-tree`: **CONFLICT 0건** ✓
- BEHIND — devel 영역 영역 5/10 사이클 영역 영역 진전, 본 PR 영역 영역 rhwp-studio 영역 영역 격리 영역 영역 충돌 부재

## 9. 처리 옵션

### 옵션 A — 2 commits cherry-pick + no-ff merge (추천)

```bash
git checkout -b local/task144 c578ed20
git cherry-pick b25a9640 1c45a840
git checkout local/devel
git merge --no-ff local/task144
```

→ **옵션 A 추천**. 이후 PR #765 close + Issue #763 close (PR #738 영역 영역 통합 영역 영역 명시).

## 10. 검증 게이트 (구현 단계 DoD)

### 자기 검증
- [ ] `cargo build --release` 통과 (Rust 변경 부재 영역 영역 영향 없음 보장)
- [ ] `cargo test --release` — 전체 ALL GREEN
- [ ] `cd rhwp-studio && npx tsc --noEmit` 통과
- [ ] `cd rhwp-studio && npm run build` 통과 (선택)
- [ ] 광범위 sweep — 7 fixture / 170 페이지 회귀 0 (TypeScript 영역 영역 SVG 무영향 보장)

### 시각 판정 게이트 — **WASM 빌드 + 작업지시자 인터랙션 검증 권장**

본 PR 영역 영역 의 본질 영역 영역 **rhwp-studio editor 영역 영역 사용자 인터랙션 (모드 토글 + 자동완성 + 탭 템플릿)**:
- WASM 빌드 후 dev server 영역 영역 수식 편집 대화상자 영역 영역 점검
- HWP/LaTeX 모드 전환 + 자동완성 + 탭 템플릿 + 기호 검색 + LaTeX 전환 힌트
- E2E 자동 테스트 신규 부재 → 작업지시자 직접 인터랙션 검증 권장

## 11. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure 20+ 사이클 (5/10 사이클 영역 영역 PR #728/#729/#730/#734/#735/#737/#738 영역 7번째) |
| `feedback_pr_supersede_chain` | **(a) 패턴 권위 사례** — PR #738 영역 영역 PR #765 (closes #763) 영역 영역 supersede (close+통합 머지). PR #649 → #650 동일 패턴 정합. |
| `feedback_image_renderer_paths_separate` | rhwp-studio TypeScript 영역 영역 격리 — Rust 렌더링 경로 영역 영역 무영향 |
| `feedback_process_must_follow` | 백엔드 변경 부재 영역 영역 위험 좁힘 + Rust 수식 파서 영역 영역 양 구문 처리 영역 영역 의 활용 (인프라 재사용) |
| `feedback_visual_judgment_authority` | rhwp-studio editor 인터랙션 영역 영역 — Canvas visual diff CI + 작업지시자 인터랙션 검증 |

## 12. 처리 순서 (승인 후)

1. `local/devel` 영역 영역 2 commits cherry-pick (옵션 A)
2. 자기 검증 (cargo test + tsc + 광범위 sweep)
3. WASM 빌드 + 작업지시자 인터랙션 검증 (dev server 영역 영역 수식 편집 대화상자 영역 영역 모드 토글 / 자동완성 / 탭 / 기호 검색 / LaTeX 힌트)
4. 인터랙션 검증 통과 → no-ff merge + push + archives 이동 + 5/10 orders 갱신
5. PR #738 close (closes #144 자동 정합)
6. **PR #765 close** + **Issue #763 close** (PR #738 영역 영역 supersede 명시) — 작업지시자 결정 영역 영역 (a) 패턴 처리

---

작성: 2026-05-10
