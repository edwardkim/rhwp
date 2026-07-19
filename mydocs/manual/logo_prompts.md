---
kind: snapshot
status: historical
canonical: mydocs/manual/branding_strategy.md
last_verified: 2026-07-17
---

# rhwp 로고 생성 AI 프롬프트

## 브랜드 핵심 키워드

- 알(egg) — 탄생, 시작, 가능성
- 깨어남 — 닫힌 포맷을 열다
- ㅎ(한글 초성) — 한글 문서
- 모두(All) — 모든 사람, 모든 AI, 모든 플랫폼
- 정밀함 — 1px 단위 조판 정확도
- Rust — 기술적 신뢰성

## 프롬프트

### 프롬프트 1: 알에서 깨어나는 ㅎ

```
A minimal, modern logo design for an open-source document viewer called "rhwp".

The logo depicts a smooth egg shape with a subtle crack line running diagonally across it. Through the crack, the Korean character "ㅎ" (Hangul consonant) is visible, glowing softly from inside. The egg shell is pearl white with a slight golden shimmer. The "ㅎ" inside is rendered in deep navy blue (#1B3A6B).

Clean flat design, no gradients except a subtle inner glow. Suitable for both light and dark backgrounds. Square format, works at 16x16 favicon size and 512x512. No text, just the symbol.

Style: geometric, minimal, professional. Similar quality to Mozilla, Rust Foundation, or JetBrains logos.
```

### 프롬프트 2: 알 + R 모노그램

```
A sophisticated monogram logo combining the letter "R" and an egg silhouette for an open-source project called "rhwp" (meaning "All HWP" — a Korean document format viewer).

The letter "R" is stylized so its curved stroke forms the outline of an egg. The counter space (hole) of the R contains a subtle Korean character "ㅎ". Color palette: navy blue (#1B3A6B) for the main form, warm orange (#E8731A) accent on the egg crack line, white negative space.

Flat vector design, no shadows, no 3D effects. Geometric and balanced. Professional tech company quality. Works as app icon, favicon, and print logo.
```

### 프롬프트 3: 열린 알 — 문서가 나오는 순간

```
A logo for "rhwp" — an open-source Korean document (HWP) viewer/editor.

Concept: An egg seen from above, cracked open in half. Inside the egg, instead of yolk, there is a small document page with Korean text lines visible. The two shell halves frame the document. The crack line forms a subtle "R" shape.

Colors: Shell in warm ivory/cream, document in clean white with light blue text lines, background transparent. Accent: thin rust-orange (#E8731A) line along the crack edge.

Ultra-minimal, flat, vector style. App icon friendly — must be recognizable at 32x32 pixels. No literal text in the logo, only implied text lines on the tiny document.
```

### 프롬프트 4: 순수 타이포그래피

```
A typographic logo for "rhwp" — an open-source document platform.

The four letters "rhwp" in a custom geometric sans-serif typeface. The "r" is slightly larger and styled as a standalone mark — its stem curves to suggest an egg shape. A hairline crack runs through the "r" from top-right to bottom-left.

Color: Single color, deep navy (#1B3A6B). The crack line in rust orange (#E8731A). Kerning is tight but not touching. Weight is medium — confident but not heavy.

Below the lettermark, in small caps with generous letter-spacing: "모두의 한글" in a clean Korean typeface.

Vector, flat, scalable. Corporate identity quality — suitable for GitHub README header, VS Code Marketplace banner, and business cards.
```

### 프롬프트 5: 추상적 — 열림의 순간

```
Abstract logo mark for "rhwp", an open-source project that opens closed document formats for everyone.

Two curved shapes facing each other, like an egg split in two, with a narrow gap of light between them. The gap forms the negative space of the Korean character "ㅎ". The overall silhouette is circular/oval.

Left half: navy blue (#1B3A6B). Right half: rust orange (#E8731A). Gap: white/transparent. The two halves don't touch — the opening between them is the core message.

Geometric precision, perfect symmetry, flat vector. No texture, no gradient. Think Paul Rand, Saul Bass — timeless simplicity. Must work as monochrome (single color) version too.
```

## 활용 가이드

| 용도 | 권장 프롬프트 | 크기 |
|------|-------------|------|
| VS Code 익스텐션 아이콘 | 1 또는 2 | 128×128 |
| GitHub 리포지토리 | 4 (타이포그래피) | 배너 1280×640 |
| favicon | 1 또는 5 | 32×32, 16×16 |
| Open VSX / Marketplace | 2 또는 3 | 256×256 |
| 소셜 미디어 | 5 (추상적) | 1:1 정사각 |
| 문서 헤더 | 4 + "모두의 한글" | 가로형 |

## AI 도구별 팁

| 도구 | 팁 |
|------|-----|
| Midjourney | `--style raw --ar 1:1 --v 6` 추가. 로고는 `--style raw`가 필수 |
| DALL-E 3 | 프롬프트 그대로 사용. "I NEED this exact design" 추가하면 지시 준수율 향상 |
| Stable Diffusion | 네거티브 프롬프트: "realistic, photo, 3d, gradient, shadow, text" |
| Ideogram | 텍스트 포함 로고에 강함. 프롬프트 4에 적합 |
| Adobe Firefly | 벡터 스타일 지정 가능. "vector flat logo" 키워드 추가 |

## 생성 후 후처리

1. AI 생성물을 **참고용**으로 사용하고, 최종 로고는 벡터로 재작업
2. SVG로 변환하여 모든 크기에서 선명하게
3. 단색 버전(흑백) 반드시 제작 — favicon, 인쇄용
4. 다크 모드용 반전 버전 제작
