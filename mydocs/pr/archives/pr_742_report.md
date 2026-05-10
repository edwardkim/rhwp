---
PR: #742
제목: Task #420 — 문서 전환 시 글꼴 드롭다운 초기화 (이전 문서 폰트 잔존 정정)
컨트리뷰터: @oksure (Hyunwoo Park) — 20+ 사이클 핵심 컨트리뷰터 (5/10 사이클 11번째 PR)
처리: 옵션 A — 1 commit cherry-pick + no-ff merge
처리일: 2026-05-10
머지 commit: 0a7ea499
---

# PR #742 처리 보고서

## 1. 처리 결과

✅ **머지 완료** — 옵션 A (1 commit cherry-pick + no-ff merge `0a7ea499`)

| 항목 | 값 |
|------|-----|
| 머지 commit | `0a7ea499` (--no-ff merge) |
| Cherry-pick commit | `8927d561` |
| closes | #420 |
| 시각 판정 | ✅ 작업지시자 웹 에디터 인터랙션 검증 통과 |
| 자기 검증 | tsc + cargo test ALL GREEN + sweep 170/170 same + WASM 4.65 MB |

## 2. 정정 본질 — 2 files, +28/-1

### 2.1 `rhwp-studio/src/ui/toolbar.ts` (+26)

`initFontDropdown(docFonts?: string[])` 메서드 신규:

```typescript
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
2. BASE_FONTS 7개 재등록
3. `docFonts` 중복 없이 추가
4. 대표/로컬 글꼴 복원

### 2.2 `rhwp-studio/src/main.ts` (+2/-1)

`initializeDocument()` 영역 영역 `initStyleDropdown()` 직전 영역 영역 `initFontDropdown(docInfo.fontsUsed)` 호출 추가.

## 3. 결함 본질 (Issue #420)

HY헤드라인M 사용 문서 → 텍스트 복사 → 새 문서 → 글꼴 드롭다운 영역 영역 HY헤드라인M 잔존. 스타일 드롭다운(`initStyleDropdown` 정합) 영역 영역 대비 글꼴 드롭다운 영역 영역 대응 메서드 부재.

## 4. 인프라 재사용

| 인프라 | 활용 |
|--------|------|
| `initStyleDropdown()` 패턴 | `initFontDropdown()` 정합 |
| `populateFontSetOptions()` / `populateLocalFontOptions()` | 기존 메서드 재호출 |
| `DocumentInfo.fontsUsed` | 기존 IR 활용 |

→ `feedback_process_must_follow` 정합 — 신규 인프라 도입 부재 영역 영역 위험 좁힘.

## 5. 본 환경 cherry-pick + 검증

| 검증 | 결과 |
|------|------|
| `cherry-pick` (1 commit) | ✅ 충돌 0건 (auto-merge 정합) |
| `tsc --noEmit` (rhwp-studio) | ✅ 통과 |
| `cargo test --release` | ✅ ALL GREEN |
| 광범위 sweep (7 fixture / 170 페이지) | ✅ **170 same / 0 diff** |
| WASM 빌드 (강제 재빌드) | ✅ 4.65 MB |

## 6. 작업지시자 웹 에디터 인터랙션 검증 ✅ 통과

- HY헤드라인M 사용 문서 → 새 문서 → 글꼴 드롭다운 영역 영역 HY헤드라인M 부재 ✅
- BASE_FONTS 7개 + 대표/로컬 정합 표시 ✅

## 7. 영향 범위

### 7.1 변경 영역
- rhwp-studio editor 의 글꼴 드롭다운 문서 전환 시 초기화

### 7.2 무변경 영역
- 스타일 드롭다운 (이미 정합)
- WASM 코어 (Rust) — 변경 부재
- HWP3/HWPX 변환본 시각 정합 (sweep 170/170 same 입증)

## 8. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure **20+ 사이클** (5/10 사이클 PR #728/#729/#730/#734/#735/#737/#738/#739/#740/#742 영역 영역 11번째 PR) |
| `feedback_image_renderer_paths_separate` | TypeScript 단일 영역 변경 — Rust 렌더링 경로 무영향 (sweep 170/170 same 입증) |
| `feedback_process_must_follow` | 인프라 재사용 (initStyleDropdown 패턴 + populateFontSetOptions/LocalFontOptions) — 위험 좁힘 |
| `feedback_visual_judgment_authority` | 작업지시자 웹 에디터 인터랙션 검증 ✅ 통과 |

## 9. 잔존 후속

- 본 PR 본질 정정 영역 영역 잔존 결함 부재

---

작성: 2026-05-10
