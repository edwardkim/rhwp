# Task #68: SVG export 폰트 서브셋 임베딩 — 구현 계획서

## 현재 아키텍처

- SVG에 `<style>` 섹션 없음, `<defs>`는 gradient/clip-path만 포함
- 폰트 이름은 `<text>` 요소의 font-family 속성에 inline
- 폰트 서브셋/임베딩 crate 미사용
- PDF 렌더러에서 `usvg::fontdb`로 시스템 폰트 로드하는 경로 존재

## CLI 옵션 설계

```bash
# 기본: 폰트 임베딩 없음 (CSS font-family 체인만)
rhwp export-svg sample.hwp

# 폰트 스타일만 삽입 (@font-face + local() 참조, 데이터 미포함)
rhwp export-svg sample.hwp --font-style

# 폰트 서브셋 임베딩 (사용 글자만 추출 + base64)
rhwp export-svg sample.hwp --embed-fonts

# 폰트 전체 임베딩 (서브셋 없이 전체 폰트 base64)
rhwp export-svg sample.hwp --embed-fonts=full

# 폰트 파일 탐색 경로 지정 (여러 번 지정 가능, 누적)
rhwp export-svg sample.hwp --embed-fonts --font-path /path/to/fonts
rhwp export-svg sample.hwp --embed-fonts --font-path ~/myfonts --font-path /usr/share/fonts
```

### 옵션 비교

| 옵션 | SVG 크기 | 오프라인 | 폰트 데이터 |
|------|---------|---------|------------|
| (없음) | 최소 | ❌ | 없음 |
| `--font-style` | +수 KB | ❌ (로컬 참조) | `@font-face { src: local("폰트명") }` |
| `--embed-fonts` | +수십~수백 KB | ✅ | 사용 글자 서브셋 base64 |
| `--embed-fonts=full` | +수 MB | ✅ | 전체 폰트 base64 |

### `--font-style` 동작

```svg
<style>
@font-face { font-family: "함초롬바탕"; src: local("함초롬바탕"), local("HCR Batang"); }
@font-face { font-family: "맑은 고딕"; src: local("Malgun Gothic"), local("맑은 고딕"); }
</style>
```
- 데이터 없이 `local()` 참조만 — 시스템에 폰트 있으면 사용
- 파일 크기 증가 미미, 정확한 폰트 매칭 향상

## 의존성 추가

- `subsetter = "0.2"` — OpenType 폰트 서브셋 (네이티브 전용, `cfg(not(wasm32))`)
- 기존 `base64 = "0.22"` 활용
- SVG에서는 base64 OTF/TTF도 브라우저가 지원하므로 woff2 변환 불필요

---

## 구현 단계 (4단계)

### 1단계: CLI 옵션 파싱 + SvgRenderer 폰트 수집

**대상 파일:** `src/main.rs`, `src/renderer/svg.rs`

**작업 내용:**
1. `src/main.rs` export-svg CLI에 옵션 추가:
   - `--font-style` — 로컬 참조 스타일만
   - `--embed-fonts[=full]` — 서브셋(기본) 또는 전체 임베딩
   - `--font-path <경로>` — 폰트 탐색 경로 (여러 번 지정 가능)
2. `SvgRenderer` 구조체에 추가:
   - `font_codepoints: HashMap<String, HashSet<char>>` — 폰트별 사용 글자 수집
   - `font_mode: FontEmbedMode` — None / Style / Subset / Full
   - `font_paths: Vec<PathBuf>` — 추가 폰트 탐색 경로
3. `render_node()`에서 TextRun 렌더링 시 `font_family`와 텍스트 char 수집

---

### 2단계: `--font-style` 구현 (local() 참조)

**대상 파일:** `src/renderer/svg.rs`

**작업 내용:**
1. 수집된 폰트 목록에서 `@font-face { src: local("...") }` CSS 생성
2. 폰트명 → local() 별칭 매핑 (한글명 + 영문명)
3. SVG finalize 시 `<svg>` 태그 직후에 `<style>` 삽입

---

### 3단계: `--embed-fonts` 구현 (서브셋/전체 임베딩)

**대상 파일:** `src/renderer/svg.rs`, `Cargo.toml`

**작업 내용:**
1. `Cargo.toml`에 `subsetter = "0.2"` 추가 (`cfg(not(wasm32))`)
2. 폰트 파일 탐색 함수:
   - 기본 경로: `ttfs/hwp/`, `ttfs/windows/`, `ttfs/`, 시스템 경로
   - `--font-path` 지정 경로를 최우선 탐색
3. 서브셋 모드: `subsetter::subset()` → 사용 글자만 추출 → base64
4. 전체 모드: TTF 파일 전체 → base64
5. `@font-face { font-family: "..."; src: url(data:font/opentype;base64,...); }` CSS 생성
6. `<style>` 삽입 (`defs_insert_pos` 활용)

---

### 4단계: 검증 + 회귀 테스트

**작업 내용:**
1. 각 옵션별 SVG 내보내기 및 결과 확인
   - 기본: `<style>` 없음
   - `--font-style`: `<style>` + `local()` 참조
   - `--embed-fonts`: `<style>` + base64 서브셋
   - `--embed-fonts=full`: `<style>` + base64 전체
2. 오프라인 브라우저에서 `--embed-fonts` SVG 한글 확인
3. `cargo test` 전체 통과
4. 파일 크기 비교

## 폰트 파일 탐색 우선순위

```
1. --font-path 지정 경로 (최우선)
2. ttfs/hwp/      — 프로젝트 HWP 전용 폰트
3. ttfs/windows/   — 프로젝트 Windows 폰트
4. ttfs/           — 프로젝트 일반 폰트
5. 시스템 폰트 경로:
   - macOS: /Library/Fonts, /System/Library/Fonts
   - Linux: /usr/share/fonts, /usr/local/share/fonts
   - Windows: C:\Windows\Fonts
```

## 폰트명 → 파일명 매핑

기존 `font_metrics_data.rs`의 `FONT_METRIC_ALIASES`와 PDF 렌더러의 fontdb 매핑 참조.
매핑 실패 시 해당 폰트는 임베딩 건너뜀 (CSS 체인 폴백에 의존).

## 리스크

| 리스크 | 대응 |
|--------|------|
| 서브셋 실패 (복잡한 OTF) | try/catch, 실패 시 해당 폰트 건너뜀 |
| 파일 크기 증가 | 한글 서브셋은 사용 글자 수에 비례, 보통 수십~수백 KB |
| 시스템에 폰트 파일 없음 | 임베딩 건너뛰고 CSS 폴백 체인 유지 |
| WASM 빌드에 subsetter 포함 | `cfg(not(wasm32))` 조건부 컴파일 |
| `--font-path` 경로에 폰트 없음 | 다음 탐색 경로로 폴백 |
