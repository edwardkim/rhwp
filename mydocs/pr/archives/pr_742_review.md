---
PR: #742
제목: Task #420 — 문서 전환 시 글꼴 드롭다운 초기화 (이전 문서 폰트 잔존 정정)
컨트리뷰터: @oksure (Hyunwoo Park) — 20+ 사이클 핵심 컨트리뷰터 (5/10 사이클 11번째 PR)
base / head: devel / contrib/font-dropdown-reset
mergeStateStatus: BEHIND
mergeable: MERGEABLE — `git merge-tree` 충돌 0건
CI: ALL SUCCESS (Build & Test / Canvas visual diff / CodeQL / Analyze rust+js+py / WASM SKIPPED)
변경 규모: +28 / -1, 2 files
검토일: 2026-05-10
---

# PR #742 검토

## 1. 메타

| 항목 | 값 |
|------|-----|
| PR 번호 | #742 |
| 제목 | Task #420 — 문서 전환 시 글꼴 드롭다운 초기화 |
| 컨트리뷰터 | @oksure — 20+ 사이클 (5/10 사이클 11번째 PR) |
| base / head | devel / contrib/font-dropdown-reset |
| mergeStateStatus | BEHIND, mergeable: MERGEABLE — 충돌 0건 |
| CI | ALL SUCCESS (Canvas visual diff 포함) |
| 변경 규모 | +28 / -1, 2 files |
| 커밋 수 | 1 (`f5d6dce2`) |
| closes | #420 |

## 2. 결함 본질 (Issue #420)

### 2.1 결함 영역
HY헤드라인M 폰트 사용 문서를 열고 텍스트 복사 후 새 문서 생성 시, 이전 문서의 폰트(HY헤드라인M)가 새 문서의 글꼴 드롭다운에 잔존. 스타일 드롭다운은 `initStyleDropdown()`으로 문서 전환 시 초기화되지만, 글꼴 드롭다운에는 대응 메서드가 없어 누적.

### 2.2 채택 접근
`initStyleDropdown()` 패턴 정합 — `initFontDropdown(docFonts?: string[])` 신규 추가 + `replaceChildren()` 통한 완전 초기화 + 기본 7폰트 + 문서 고유 폰트 + 대표/로컬 글꼴 복원.

## 3. PR 정정 — 2 files, +28/-1

### 3.1 `rhwp-studio/src/ui/toolbar.ts` (+26)

```typescript
/** 문서 로드 시 글꼴 드롭다운을 초기화한다 (기본 글꼴 + 문서 글꼴 + 대표/로컬) */
initFontDropdown(docFonts?: string[]): void {
    const BASE_FONTS = ['함초롬바탕', '함초롬돋움', '맑은 고딕', '나눔고딕', '바탕', '돋움', '궁서'];
    this.fontName.replaceChildren();
    for (const name of BASE_FONTS) {
        const opt = document.createElement('option');
        opt.value = name;
        opt.textContent = name;
        this.fontName.appendChild(opt);
    }
    if (docFonts?.length) {
        const seen = new Set(BASE_FONTS);
        for (const name of docFonts) {
            if (!seen.has(name)) {
                const opt = document.createElement('option');
                opt.value = name;
                opt.textContent = name;
                this.fontName.appendChild(opt);
                seen.add(name);
            }
        }
    }
    this.populateFontSetOptions();
    this.populateLocalFontOptions();
}
```

처리 단계:
1. `replaceChildren()` — 드롭다운 완전 초기화
2. BASE_FONTS 7개 재등록 (함초롬바탕/함초롬돋움/맑은 고딕/나눔고딕/바탕/돋움/궁서)
3. `docFonts` (현재 문서 `fontsUsed`) 중복 없이 추가
4. `populateFontSetOptions()` + `populateLocalFontOptions()` 재호출 — 대표/로컬 글꼴 복원

### 3.2 `rhwp-studio/src/main.ts` (+2/-1)

```typescript
console.log('[initDoc] 6. toolbar initFontDropdown + initStyleDropdown');
toolbar?.initFontDropdown(docInfo.fontsUsed);
toolbar?.initStyleDropdown();
```

`initializeDocument()` 영역 영역 `initStyleDropdown()` 직전 영역 영역 `initFontDropdown(docInfo.fontsUsed)` 호출 추가.

## 4. 인프라 재사용

| 인프라 | 활용 |
|--------|------|
| `initStyleDropdown()` 패턴 | 본 PR `initFontDropdown()` 정합 |
| `populateFontSetOptions()` / `populateLocalFontOptions()` | 기존 메서드 재호출 — 대표/로컬 폰트 복원 |
| `DocumentInfo.fontsUsed` | 기존 IR 활용 — 문서 고유 폰트 목록 |

→ `feedback_process_must_follow` 정합 — 신규 인프라 도입 부재.

## 5. 영향 범위

### 5.1 변경 영역
- rhwp-studio editor 의 글꼴 드롭다운 영역 영역 문서 전환 시 초기화

### 5.2 무변경 영역
- 스타일 드롭다운 (이미 정합)
- WASM 코어 (Rust) — 변경 부재
- HWP3/HWPX 변환본 시각 정합

### 5.3 위험 영역
- TypeScript 단일 영역 변경 — Rust 렌더링 경로 무영향
- `replaceChildren()` 표준 DOM API — 호환성 정합

## 6. 충돌 / mergeable

- `git merge-tree --write-tree`: **CONFLICT 0건** ✓
- BEHIND — devel 영역 영역 5/10 사이클 진전, 본 PR 단일 메서드 추가 영역 영역 충돌 부재

## 7. 처리 옵션

### 옵션 A — 1 commit cherry-pick + no-ff merge (추천)

```bash
git checkout -b local/task420 1c7a3043
git cherry-pick f5d6dce2
git checkout local/devel
git merge --no-ff local/task420
```

→ **옵션 A 추천**.

## 8. 검증 게이트 (구현 단계 DoD)

### 자기 검증
- [ ] `cargo build --release` 통과 (Rust 변경 부재 영역 영역 영향 없음)
- [ ] `cargo test --release` ALL GREEN
- [ ] `cd rhwp-studio && npx tsc --noEmit` 통과
- [ ] 광범위 sweep — 7 fixture / 170 페이지 회귀 0 (TypeScript 영역 영역 SVG 무영향)

### 시각 판정 게이트 — **WASM 빌드 + 작업지시자 인터랙션 검증 권장**

본 PR 본질 영역 영역 **rhwp-studio editor 사용자 인터랙션**:
- WASM 빌드 후 dev server 영역 영역 점검
  - HY헤드라인M 문서 열기 → 텍스트 복사 → 새 문서 → 글꼴 드롭다운 영역 영역 HY헤드라인M 부재 점검
  - 새 문서 영역 영역 BASE_FONTS 7개 + 대표/로컬 글꼴 정합 표시
- E2E 자동 테스트 신규 부재 → 작업지시자 직접 인터랙션 검증 권장

## 9. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure 20+ 사이클 (5/10 사이클 영역 영역 PR #728/#729/#730/#734/#735/#737/#738/#739/#740/#742 영역 11번째) |
| `feedback_image_renderer_paths_separate` | TypeScript 단일 영역 변경 — Rust 렌더링 경로 무영향 |
| `feedback_process_must_follow` | 인프라 재사용 (initStyleDropdown 패턴 + populateFontSetOptions/LocalFontOptions) — 위험 좁힘 |
| `feedback_visual_judgment_authority` | rhwp-studio editor 인터랙션 — 작업지시자 인터랙션 검증 권장 |

## 10. 처리 순서 (승인 후)

1. `local/devel` 영역 영역 1 commit cherry-pick (옵션 A)
2. 자기 검증 (cargo test + tsc + 광범위 sweep)
3. WASM 빌드 + 작업지시자 인터랙션 검증
4. 인터랙션 검증 통과 → no-ff merge + push + archives 이동 + 5/10 orders 갱신
5. PR #742 close (closes #420 자동 정합)

---

작성: 2026-05-10
