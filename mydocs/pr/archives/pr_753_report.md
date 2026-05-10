---
PR: #753
제목: Task #741 — hwp3-sample10.hwp HWP3 native 렌더링 8 단계 정정 + Git LFS 구축
컨트리뷰터: @jangster77 (Taesup Jang) — HWP3 핵심 컨트리뷰터 (16+ 사이클)
처리: 옵션 B — Task #741 commits 11개 개별 cherry-pick + 메인테이너 chore 2개 + no-ff merge
처리일: 2026-05-10
머지 commit: a90ecd09
---

# PR #753 처리 보고서

## 1. 처리 결과

✅ **머지 완료** — Task #741 commits 11개 + 메인테이너 chore 2개 (.gitattributes + CLAUDE.md)

| 항목 | 값 |
|------|-----|
| 머지 commit | `a90ecd09` (--no-ff merge) |
| Cherry-pick commits | 11개 (Stage 1~4/5/6/7/7후속/8 + 후속 4개 + CI fix) |
| 메인테이너 chore | `fe4676c3` (.gitattributes + pdf-large/README) + `80eccfef` (CLAUDE.md) |
| Skipped | `ab2fa527` (devel `25299a5b` 영역 영역 이미 적용) |
| closes | #741 |
| 시각 판정 | ✅ 작업지시자 웹 에디터 시각 판정 통과 |
| 자기 검증 | cargo build/test ✅ + tsc ✅ + sweep 170/170 same + WASM 4.68 MB + hwp3-sample10 svg export ✅ |

## 2. 정정 본질 (Issue #741)

`hwp3-sample10.hwp` (Oracle 기술 문서, 763 페이지) HWP3 native 렌더링 결함 8개 영역 정정 — 한컴 viewer + PDF (한글 2022) 영역 정합 도달.

### 2.1 Stage 1~8

| Stage | 영역 | commit |
|-------|------|--------|
| 1~4 | HWP3 외부 file path 그림 IR (`ImageAttr.external_path`) + image placeholder + TAC 그림 paragraph line_spacing 정합 (ls=600) + HWP5 변환본 paragraph 26 페이지 분할 정합 (vpos-reset 후속 가드) | `81c16cfa` |
| 5 | HWP3 사적 graphic char (0x0080~0x7FFF) cross-ref 매핑 — 표준 KSSM (0x8000+) 외 한컴 사적 인코딩. 상위 6값 = 98.5% coverage | `e184718b` |
| 6 | HWP3 ParaShape tabs[40] → Document IR TabDef 변환 + **`Hwp3TabDef` 필드 순서 bug 정정 (30+ 사이클 미발견 본질 결함)** | `1348d939` |
| 7 | HWP3 leader → HWP5 fill_type 매핑 (1→3 점선) + 제목차례 자동 장식 inject (한컴 사적 로직 cross-ref) | `d574cf13` + `d2ddb3bc` |
| 8 | HWP3 차례 inline page 번호 (ch=1) decode + display | `639088a9` |

### 2.2 후속 (외부 그림 영역)
- `02130377` rhwp-studio web canvas + dialog 정정
- `dd71507d` 외부 그림 fixture 3개 (oracle.gif / rdb02.gif / s1.jpg)
- `4bfcee05` native CLI 자동 load + WASM API + Vite middleware (PDF 96 MB 영역 영역 `pdf-large/` 영역 영역 LFS pointer 영역 영역 이동)
- `314bb121` CI fix: tests/issue_516.rs ImageAttr external_path

### 2.3 핵심 본질 발견

**Stage 5 — HWP3 사적 인코딩**:
HWP3 hchar 0x0080~0x7FFF 영역 영역 표준 KSSM 조합형 외 한컴 사적 인코딩. johab decoder '?' 반환 → 가로선/▷/■ 누락. 한컴 변환본 cross-ref 영역 매핑 도출.

**Stage 6 — Hwp3TabDef 필드 순서 bug**:
기존 `(position:u16, type:u8, leader:u8)` — 실제 byte stream 어긋남. `HWP3_DIAG_TABS` 진단으로 default tab pattern 검증 → `(tab_type:u8, leader:u8, position:u16 LE)` 로 정정. **30+ 사이클 미발견 본질 결함**.

**Stage 7 — 한컴 viewer 자동 장식**:
HWP3 paragraph 26 (cc=8 "￼￼ 제목차례 ") → 한컴 viewer "════════════════════■ 제목차례 ■══════════════════════" 자동 장식 inject. **HWP3 spec 외 한컴 사적 로직**. 한컴 변환본 cross-ref 영역 trigger 조건 (새번호 + 쪽번호위치 + visible text ≤ 6 chars) 도출.

**Stage 8 — 차례 inline page 번호 (ch=1)**:
HWP3 차례 entries 가 페이지 번호를 inline `ch=1` control 로 저장. 290 occurrences 모두 정합.

## 3. Git LFS 신규 구축 — pdf-large/ 폴더 한정 격리

PR 본문 영역 명시된 PDF 96 MB (GitHub 권장 50 MB 초과) 영역 영역 본 환경 영역 영역 Git LFS 신규 구축:

### 3.1 메인테이너 chore commits
- `fe4676c3` chore: Git LFS pdf-large/ 한정 추적 설정
  - `.gitattributes` 영역 영역 `pdf-large/**/*.pdf filter=lfs diff=lfs merge=lfs -text` 패턴
  - `pdf-large/README.md` 영역 영역 폴더 정책 문서
- `80eccfef` docs: CLAUDE.md 영역 pdf-large/ 폴더 정책 추가
  - "예제 폴더" + "PDF 권위 자료 명명 규약" 갱신

### 3.2 PDF 배치
- 본 PR 영역 영역 `pdf/hwp3-sample10-hwp5-2022.pdf` 진입 commit 영역 영역 PDF 사전 제외 후 cherry-pick (Stage 5)
- 후속 commit (`4bfcee05`) cherry-pick 시 modify/delete 충돌 → PDF 영역 영역 `pdf-large/` 영역 영역 LFS pointer 영역 영역 직접 이동
- LFS 업로드 96 MB 완료 (push 시)

### 3.3 정책
- 기존 `pdf/` / `pdf-2020/` / `pdf-2010/` 영역 영역 일반 git 영역 영역 보존 (< 50 MB)
- 신규 50 MB 초과 PDF 영역 영역 `pdf-large/` 영역 영역 직접 배치 (자동 LFS 변환)
- Clone / Fork 영역 영역 LFS 미설치 환경 영역 영역 placeholder 영역 영역 진입

## 4. 인프라 사용

| 인프라 | 활용 |
|--------|------|
| Git LFS (신규 구축) | pdf-large/ 폴더 한정 추적 |
| `git-lfs 3.5.1` | 사용자 영역 (~/.local/bin/) 영역 영역 설치 (root 미사용) |
| `find_initial_column_def` 등 (기존 IR) | Stage 1~4 image placeholder + Stage 6 TabDef 변환 |
| `executeOperation` (PR #728 인프라) | Stage 후속 영역 영역 외부 그림 dialog 영역 영역 활용 |

## 5. 본 환경 검증

| 검증 | 결과 |
|------|------|
| Cherry-pick 충돌 | ✅ 0건 (Stage 5 PDF 진입 영역 영역 사전 제외 + 4bfcee05 modify/delete 영역 영역 pdf-large/ 이동 처리) |
| `cargo build --release` | ✅ 통과 |
| `cargo test --release` | ✅ ALL GREEN |
| `tsc --noEmit` (rhwp-studio) | ✅ 통과 (WASM 빌드 후 — 신규 WASM API 영역) |
| 광범위 sweep (7 fixture / 170 페이지) | ✅ **170 same / 0 diff** |
| WASM 빌드 (Docker) | ✅ 4.68 MB (신규 WASM API 영역 영역 약간 증가) |
| `git lfs ls-files` | ✅ `pdf-large/hwp3-sample10-hwp5-2022.pdf` LFS pointer |
| LFS 업로드 (push 시) | ✅ 96 MB 완료 |
| hwp3-sample10 native svg export | ✅ 페이지 1/2 정합 (53/489 KB) |

## 6. 작업지시자 웹 에디터 시각 판정 ✅ 통과
- 페이지 1: image placeholder 표시
- 페이지 2: 제목차례 자동 장식 + ▷ markers + 점선 leader + 페이지 번호
- 페이지 28~339: 차례 inline page 번호 정합

## 7. 영향 범위

### 7.1 변경 영역
- `src/parser/hwp3/` — Stage 5/6/7/8 종합 (johab.rs / mod.rs / records.rs / drawing.rs)
- `src/model/image.rs` — Stage 1~4 (`ImageAttr.external_path`)
- `src/renderer/` — Stage 1~4 (image placeholder + vpos-reset 후속 가드)
- `src/wasm_api.rs` / `src/main.rs` / `src/document_core/commands/document.rs` / `src/model/document.rs` / `src/renderer/web_canvas.rs` — Stage 후속 영역 (외부 그림 native CLI + WASM API)
- `rhwp-studio/` — Stage 후속 (web canvas + dialog + Vite middleware)
- `samples/hwp3-sample10*` (3개) + `samples/oracle.gif` / `rdb02.gif` / `s1.jpg` (외부 그림 fixture)
- `pdf-large/hwp3-sample10-hwp5-2022.pdf` (96 MB, LFS)
- `.gitattributes` (신규)
- `pdf-large/README.md` (신규)
- `CLAUDE.md` (예제 폴더 + 명명 규약 갱신)

### 7.2 무변경 영역
- 광범위 sweep 170/170 same — HWP3 native 외 회귀 부재
- 기존 pdf/ / pdf-2020/ / pdf-2010/ 영역 영역 일반 git 영역 영역 보존

## 8. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @jangster77 16+ 사이클 (HWP3 핵심) |
| `feedback_image_renderer_paths_separate` | image placeholder 영역 영역 다중 경로 (svg.rs / picture_footnote / table_cell_content / web_canvas) 동기 정정 — **권위 사례** |
| `feedback_process_must_follow` | Task #741 단계별 분리 (Stage 1~8) — 위험 좁힘 + 단계별 self-review |
| `feedback_hancom_compat_specific_over_general` | Stage 5 사적 char 매핑 (상위 6값 = 98.5% coverage) + Stage 7 제목차례 trigger 조건 (visible text ≤ 6) — 일반화 영역 영역 회귀 위험 좁힘 |
| `feedback_pdf_not_authoritative` | 한글 2022 PDF 정답지 등급 ✅ 정합 |
| `reference_authoritative_hancom` | 한컴 viewer + PDF cross-ref 영역 영역 권위 자료 |
| `feedback_visual_judgment_authority` | 작업지시자 웹 에디터 시각 판정 ✅ 통과 |
| `feedback_visual_regression_grows` | 광범위 sweep 170/170 same 단계별 검증 |

## 9. Git LFS 정책 정합 (신규 룰 후보)

본 PR 영역 영역 도입된 Git LFS 격리 정책:
- **분리 폴더 한정**: `pdf-large/` 만 LFS 추적 (다른 폴더 영역 영역 일반 git 보존)
- **GitHub 한도 정합**: 무료 LFS 1 GB 영역 영역 효율적 활용 (96 MB = 9.6%)
- **컨트리뷰터 영역 영역 명시**: PR 본문 영역 영역 PDF 50 MB 초과 시 `pdf-large/` 배치 가이드
- **CLAUDE.md 영역 영역 명문화**: "예제 폴더" + "PDF 권위 자료 명명 규약" 갱신

## 10. 잔존 후속

- HWP3 leader 값 2+ (점선/파선) 등장 시 추가 매핑 (PR 본문 명시)
- PUA char (U+F080F, U+F0827) 폰트 fallback (별 task)
- 다른 HWP3 sample 의 "제목차례 type" 패턴 추가 발견 시 trigger 조건 확장
- LFS 사용량 모니터링 (1 GB 한도)

---

작성: 2026-05-10
