# rhwp export-png 명령 매뉴얼

## 개요

HWP 문서를 PNG 이미지로 내보내는 native Skia raster backend 도구. PR #599 에서 도입된 `PageLayerTree` 기반 raster output. SVG 와 달리 픽셀 단위 결정적 raster 출력으로 인쇄용/이미지 검증용/공공 자산 파이프라인에서 사용.

## 사전 조건

- **`native-skia` feature 빌드 필수** (기본 빌드에는 미포함)
- 빌드: `cargo build --release --features native-skia`

## 사용법

```bash
rhwp export-png <파일.hwp> [옵션]
```

### 옵션

| 옵션 | 단축 | 설명 |
|------|------|------|
| `--output <폴더>` | `-o` | 출력 폴더 (기본: `output/`) |
| `--page <번호>` | `-p` | 특정 페이지만 내보내기 (0부터 시작) |
| `--font-path <경로>` | | 폰트 파일 탐색 경로 (여러 번 지정 가능) |
| `--scale <배율>` | | 렌더링 배율 (기본: 1.0) |
| `--max-dimension <픽셀>` | | 한 변 최대 픽셀 (longest edge). 자동 scale 계산 |
| `--vlm-target <프리셋>` | | VLM (Vision-Language Model) 입력 프리셋 |
| `--dpi <값>` | | DPI 메타데이터 (PNG `pHYs` chunk). 인쇄 워크플로우 크기 힌트 |

### VLM 프리셋

AI 파이프라인 + VLM 연동 사용 사례. `--vlm-target` 으로 입력 사양 자동 조정.

| 프리셋 | 한 변 최대 | 픽셀 한도 | 비고 |
|---|---|---|---|
| `claude` | 1568 px | 1.15 MP | Claude Vision (Anthropic) |
| `gpt4v-low` | 512 px | 262K px | GPT-4V low detail (OpenAI) |
| `gpt4v-high` | 2000 px | 1.54 MP | GPT-4V high detail (별칭: `gpt4v`) |
| `gemini` | 3072 px | 9.44 MP | Google Gemini |
| `qwen-vl` | 2240 px | 5.02 MP | Qwen-VL 28×28 patch (별칭: `qwen`) |
| `llava` | 672 px | 452K px | LLaVA / OSS CLIP backbone |

프리셋 이름은 하이픈/밑줄 모두 허용 (`gpt4v-low` = `gpt4v_low`). 대소문자 무시.

### 옵션 우선순위

scale 결정 순서 (실제 픽셀 수 영역 영역 결정):

1. **`--scale <배율>`** — 명시 지정 시 최우선
2. **`--max-dimension` / `--vlm-target`** — 명시 시 한도 안에 들어가도록 자동 scale 계산 (가장 작은 결과 선택)
3. **`--dpi <값>`** — 위 모두 미지정 시 `scale = dpi / 96.0` 자동 계산 (#614)
4. (옵션 없음) — 기본 scale 1.0

`--dpi` 영역 영역 메타데이터 영역 영역 위 결정 영역 영역 무관 영역 영역 항상 적용 — 실제 픽셀 수 영역 영역 영향 부재 (`pHYs` chunk 만 삽입).

`--vlm-target` + `--dpi` 동시 사용 가능 — 픽셀 한도 (`--vlm-target`) + 메타데이터 (`--dpi`) 영역 영역 독립 동작.

### DPI 메타데이터 (`--dpi`)

PNG `pHYs` chunk 에 DPI 메타데이터를 삽입하여 인쇄 시점 크기 힌트로 사용한다. **메타데이터 전용** — 실제 래스터 픽셀 수에 영향 부재.

- `--dpi 96` — 1.0× 동등 메타데이터 (기본 SVG 출력과 정합)
- `--dpi 300` — 인쇄 표준 DPI (`--scale` 미지정 시 `scale=3.125` 자동)
- `--scale 2 --dpi 150` — `--scale` 명시 + DPI 메타데이터만 별도 (auto-scale 미적용)
- `--vlm-target gpt4v --dpi 200` — VLM 픽셀 한도 + DPI 메타데이터 동시 적용

**`--dpi` 미지정 시** `pHYs` chunk 미삽입 (opt-in, 기본 PNG 동작 100% 보존).

`pHYs` chunk 의 PPM (pixels per meter) 변환: `ppm = round(dpi / 0.0254)`. PNG spec §11.3.5.3 정합.

### 사용 예시

```bash
# 전체 페이지 PNG 내보내기 (기본)
rhwp export-png samples/exam_kor.hwp

# 특정 페이지 (page 17 = index 16) 만 내보내기
rhwp export-png samples/exam_kor.hwp -p 16

# 출력 폴더 지정
rhwp export-png samples/exam_kor.hwp -o my_output/

# 한컴 전용 폰트 (HY견명조 등) 가 시스템에 없을 때 ttfs 디렉토리 지정
rhwp export-png samples/exam_kor.hwp --font-path /home/edward/mygithub/ttfs

# 여러 폰트 디렉토리 지정
rhwp export-png samples/exam_kor.hwp \
  --font-path /home/edward/mygithub/ttfs \
  --font-path /usr/share/fonts/truetype/nanum

# 고해상도 (2배 배율, 인쇄용)
rhwp export-png samples/exam_kor.hwp --scale 2.0

# 한 변 1024 픽셀 한도 (LLaVA-style)
rhwp export-png samples/exam_kor.hwp --max-dimension 1024

# Claude Vision 입력 (1568 px / 1.15 MP 자동 조정)
rhwp export-png samples/exam_kor.hwp --vlm-target claude

# Google Gemini 입력 (3072 px / 9.44 MP 한도)
rhwp export-png samples/exam_kor.hwp --vlm-target gemini

# GPT-4V high detail (별칭 gpt4v 도 동등)
rhwp export-png samples/exam_kor.hwp --vlm-target gpt4v-high
rhwp export-png samples/exam_kor.hwp --vlm-target gpt4v

# Qwen-VL (별칭 qwen 도 동등)
rhwp export-png samples/exam_kor.hwp --vlm-target qwen

# LLaVA / OSS CLIP (672 px 한도)
rhwp export-png samples/exam_kor.hwp --vlm-target llava

# AI 파이프라인 통합 (Claude + 한컴 폰트)
rhwp export-png samples/exam_kor.hwp \
  --vlm-target claude \
  --font-path /home/edward/mygithub/ttfs \
  -o output/claude_input/

# 인쇄용 300 DPI (auto-scale 3.125)
rhwp export-png samples/exam_kor.hwp --dpi 300

# scale 명시 + DPI 메타데이터만 별도 (auto-scale 미적용)
rhwp export-png samples/exam_kor.hwp --scale 2 --dpi 150

# VLM 픽셀 한도 + DPI 메타데이터 동시 적용
rhwp export-png samples/exam_kor.hwp --vlm-target gpt4v --dpi 200
```

### 출력 dimension 예시 (exam_kor page 17 기준, native 1123 × 1588)

| 옵션 | 출력 dimension | pixel count |
|---|---|---|
| (기본) | 1123 × 1588 | 1.78 MP |
| `--scale 2.0` | 2246 × 3175 | 7.13 MP |
| `--scale 0.5` | 562 × 794 | 0.45 MP |
| `--max-dimension 1024` | 725 × 1024 | 0.74 MP |
| `--vlm-target claude` | 898 × 1269 | 1.14 MP (≤1.15 MP) |
| `--vlm-target gpt4v-low` | 362 × 512 | 0.19 MP (≤512 px / 262K) |
| `--vlm-target gpt4v-high` | 1414 × 2000 | 2.83 MP → 한도 cap (≤1.54 MP) |
| `--vlm-target gemini` | 2173 × 3072 | 6.68 MP (≤3072 px / 9.44 MP) |
| `--vlm-target qwen-vl` | 1583 × 2240 | 3.55 MP (≤2240 px / 5.02 MP) |
| `--vlm-target llava` | 475 × 672 | 0.32 MP (≤672 px / 452K) |
| `--dpi 300` (auto-scale 3.125) | 3509 × 4961 | 17.4 MP |

VLM 프리셋 영역 영역 dimension 영역 영역 페이지 native 비율 + 한 변/픽셀 한도 영역 영역 의 가장 작은 결과 영역 영역.

## 출력 파일명 규칙

- 단일 페이지 (`-p` 지정): `{파일명}.png`
- 전체 페이지: `{파일명}_001.png`, `{파일명}_002.png`, ...

페이지 번호는 1부터 시작 (사용자 친화), 내부 인덱스는 0부터 시작 (`-p` 옵션).

## 폰트 fallback 동작

본 도구는 다음 순서로 폰트를 검색:

### 1. 사용자 지정 (`--font-path`) — 최우선

`--font-path` 로 지정한 디렉토리의 모든 TTF/OTF/TTC 파일을 메모리에 로드. CharShape.font_family (예: "HY견명조") 와 일치하는 typeface 가 있으면 우선 사용.

본 환경 권장 경로: `/home/edward/mygithub/ttfs` (한컴 전용 폰트 다수 보유)

### 2. 시스템 FontMgr — 한글 fallback chain

CharShape.font_family 가 시스템에 없는 경우 다음 순서로 fallback:

```
[CharShape.font_family,]
Noto Sans KR,
Noto Serif KR,
Noto Sans CJK KR,
Noto Serif CJK KR,
Nanum Gothic,
Nanum Myeongjo,
Malgun Gothic,
맑은 고딕,
Batang,
바탕,
Apple SD Gothic Neo,
AppleMyungjo,
DejaVu Sans,
Arial,
sans-serif
```

### 3. Skia legacy typeface — 마지막 fallback

위 모두 실패 시 Skia 의 `legacy_make_typeface` 호출. 시스템에 한글 글리프 보유 폰트가 전혀 없는 경우 사각형(豆腐) 표시.

## 한컴 전용 폰트 지원

한컴 워드프로세서가 사용하는 한컴 전용 폰트 (HY견명조, HY헤드라인M, HY견고딕 등) 는 시스템 fontconfig 에 자동 등록되지 않을 수 있다. 이 경우 PNG 에서 정확한 한컴 시각을 재현하려면:

```bash
rhwp export-png input.hwp --font-path /path/to/ttfs
```

ttfs 디렉토리에 한컴 전용 폰트가 있는 경우 자동 매칭.

## 출력 형식

각 PNG 는:
- **포맷**: PNG (RGBA, 8-bit)
- **DPI**: `--dpi` 옵션 지정 시에만 메타데이터 영향 (미지정 시 `pHYs` chunk 부재 — 디코더는 96 DPI 가정)
- **렌더링**: native Skia (skia-safe 0.x)
- **크기**: 페이지 크기 × scale (기본 1.0)
- **메타데이터** (`--dpi` 지정 시): `pHYs` chunk (4-byte X ppm + 4-byte Y ppm + 1-byte unit=meter)

## 빌드 가이드

```bash
# 디버그 빌드
cargo build --features native-skia

# 릴리즈 빌드 (권장)
cargo build --release --features native-skia

# native-skia 테스트
cargo test --features native-skia skia --lib
```

## 비목표 (현재 단계)

다음은 현재 단계에서 미지원 — 향후 task 후보:

- 복잡한 텍스트 shaping (kerning / GSUB / GPOS)
- 완전한 수식 (Equation) native replay (현재 placeholder/fallback)
- ~~raw-svg native replay~~ (PR #720, P6 단계 완료)
- ~~다른 VLM 프리셋 (GPT-4V / Gemini / Qwen-VL / LLaVA)~~ (PR #735, Task #613 완료)
- form object native replay (현재 placeholder)
- CanvasKit (browser/WASM) PNG export
- Skia visual regression fixture pipeline
- PNG 비-pHYs 메타데이터 chunks (gAMA, sRGB, iCCP 등)

## 단위 환산 참고

| 변환 | 공식 |
|------|------|
| HWPUNIT → mm | `hu × 25.4 / 7200` |
| HWPUNIT → px (96DPI) | `hu × 96 / 7200` |

## 트러블슈팅

### 한글이 사각형(豆腐)으로 보임

시스템에 한글 폰트가 없거나 CharShape.font_family 가 시스템 fontconfig 에 등록 안 됨.

**해결:**
1. `--font-path` 로 한글 폰트 디렉토리 지정 (권장)
2. 시스템에 Noto Sans KR / Nanum / Apple SD Gothic Neo 등 한글 폰트 설치
3. `fc-list :lang=ko` 로 시스템 한글 폰트 확인

### export-png 명령이 인식 안 됨

```
오류: export-png 명령은 native-skia feature 가 활성화되어야 합니다.
```

**해결:** `cargo build --release --features native-skia` 로 재빌드.

### LAYOUT_OVERFLOW 경고

```
LAYOUT_OVERFLOW: page=N, col=M, para=K, ...
```

페이지 본문 영역 초과 경고. PNG 출력 자체에는 영향 없으나 레이아웃 정합 검증용 표시. 본 환경 baseline 영역 (현재 환경의 알려진 영역) 인 경우 무시 가능.

## 관련 명령

- `rhwp export-svg` — SVG 출력 (CSS font chain, 시스템 폰트 fallback)
- `rhwp dump` — 조판부호 구조 덤프
- `rhwp dump-pages` — 페이지네이션 결과 덤프

## 참고

- 본 도구의 본질 영역: PR #599 (refs #536 — 멀티 렌더러 지원 트래킹 이슈)
- 후속 진전: PR #626 (P5 equation replay) → PR #720 (P6 raw SVG replay) → PR #734 (DPI 메타데이터, Task #614) → PR #735 (VLM 프리셋 6종 확장, Task #613)
- DTP 엔진 정체성 (`project_dtp_identity`) — 다층 레이어 / WebGPU / 마스터 페이지 인프라 토대
- `feedback_image_renderer_paths_separate` — SVG (`svg.rs`) / Canvas (`web_canvas.rs`) / Skia (`skia/renderer.rs`) 별도 image 함수, 시각 결함 정정 시 모든 경로 점검
