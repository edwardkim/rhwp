# Task #902 v2 Stage 27 보고서 — WMF drop-shadow 패턴 진단 + textLength 재적용

**Stage**: 27 / 28 (v2 점진 포팅)
**상태**: 완료

## 1. 결정적 진단

작업지시자 challenge: "박스안에 문자가 정확하게 표시안되는게 무슨 폰트문제와 관련이 있음?"

WMF SVG 출력 분석 결과 발견:

```xml
<text fill="#FFFFFF" x="174" y="255">...주전산센타...</text>  ← shadow (백색)
<text fill="#000000" x="170" y="251">...주전산센타...</text>  ← main (흑색, 4px offset)
```

**WMF binary 가 각 텍스트를 2번 그림** — 약 4px 위치 차이로 백색 (shadow) + 흑색 (main).

이는 한컴 viewer 의 일반적 drop-shadow 패턴:
- 한컴 굴림체 (narrow glyph) → 두 텍스트가 충분히 겹쳐 single bold drop-shadow 시각
- NanumGothic / Apple SD Gothic Neo (wider glyph) → 두 텍스트가 덜 겹쳐 **stacking 처럼 보이는 artifact**

→ 사용자가 보는 "박스 안 문자 밀림" 본질: 폰트 metric 으로 인한 drop-shadow 패턴 visual artifact.

## 2. 해결 — textLength 재적용

Stage 26 에서 revert 했던 textLength 를 재적용. 이번엔 작동 원리 명확:

```xml
<tspan x="291" textLength="117" lengthAdjust="spacingAndGlyphs">전</tspan>
```

두 EXTTEXTOUT (shadow + main) 의 각 glyph 가 textLength=DX 로 강제 fit:
- 둘 다 같은 너비 (117) 로 렌더
- 정확히 4px offset 으로 겹침
- 한컴 처럼 drop-shadow 시각 효과

## 3. 검증

```
cargo build --release           — Finished
cargo test --release --all-targets — 1412 passed / 0 failed
wasm-pack build --release       — pkg/rhwp_bg.wasm 4.88 MB (May 16 00:27)
```

## 4. 한계

- glyph 미세 압축 effect (한컴 굴림체보다 너른 폰트가 narrowing) — 수용 가능
- 폰트 자체 quality 차이는 여전 (한컴 굴림체 vs NanumGothic 의 hinting / weight)

## 5. 다음 단계

Stage 28 (최종): 보고서 + PR
