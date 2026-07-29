---
kind: investigation
status: active
canonical: mydocs/manual/codex/docs_and_git_workflow.md
last_verified: 2026-07-29
---

# Task #3486 Stage 9 — Studio CanvasKit 옛한글 shaping과 PDF glyph 차이

- 이슈: [#3486](https://github.com/edwardkim/rhwp/issues/3486)
- 브랜치: `task_m100_3486_hwp3_render_fidelity_v2`
- 선행 커밋: `2a50b8b4a` (Stage 8: HWP3 PageNumberPos marker 제거)
- 기준 오라클: `pdf/HWP3-password-123456.pdf` p1 및 동일 문서 전수 PDF

## Stage 8 뒤에도 남은 공통 title 차이

Stage 8은 HWP3 title 앞의 잘못된 `U+FFFC` marker를 제거했다. SVG raster의 title 시작 위치는
HWP5와 같은 x=251.9px 계열로 복구됐지만, Studio 화면에서 HWP3·HWP5·HWPX 모두 `ᄒᆞᆫ글`의
첫 음절이 PDF의 `한글` glyph와 다르게 보인다. 따라서 이것은 HWP3 PageNumberPos parser 결함이
아니며 포맷 공통 paint/shaping 문제다.

동일 문서의 HWP5 Studio title crop에서도 `ᄒᆞᆫ`은 분리된 ㅎ·아래아·종성 형태로 보인다. 한컴
PDF는 제품명 `한글 97 안내문`을 현대 `한`에 가까운 하나의 glyph로 보인다. 원문 text를 임의로
현대 음절로 바꾸면 실제 옛한글 의미를 잃을 수 있으므로, 먼저 Studio가 선택한 typeface와 GSUB
shaping 결과를 직접 확인한다.

## 현재 구현이 보장하는 것과 아직 증명하지 못한 것

이미 구현된 경로:

1. `split_into_clusters()`와 `build_cluster_len()`은 `ᄒ(U+1112) + ᆞ(U+119E) + ᆫ(U+11AB)`을
   한 cluster로 만들고, layout advance를 한 번만 준다.
2. Studio `CanvasKitLayerRenderer`는 옛자모가 든 text run에
   `Source Han Serif K Old Hangul` font manager를 선택해 `ParagraphBuilder` shaping 경로로
   paint한다.
3. `rhwp-studio/public/fonts/SourceHanSerifK-OldHangul-subset.woff2`(234KB)는 runtime fetch
   대상이며, `font-loader.ts`에는 같은 family와 옛자모 unicode range가 등록돼 있다.
4. PUA인 경우에는 이미 `pua_oldhangul.rs`가 KTUG/hypua2jamo 표를 사용해 표준 자모열로
   display conversion한다. 이번 title은 이미 표준 자모열이므로 그 표의 미적용 문제가 아니다.

아직 증명하지 못한 것:

- 실제 Studio 세션에서 font fetch와 `FontMgr.FromData()`가 성공해 old-Hangul font manager가
  `renderShapedScriptText()`에 전달되는지
- 전달되더라도 subset의 GSUB(아래아 조합)가 PDF와 같은 glyph·advance를 만드는지
- font manager 부재 시 현재 code가 fallback failure를 가시적으로 기록하고 있는지

## 판별 방법과 구현 경계

다음 구현 전에는 local Studio에서 이 fixture를 연 실제 paint contract를 확인한다. title의
`displayText`, `displayPositions`, 선택 font family, old-Hangul font manager의 준비 여부를
비밀정보 없이 기록하고, PDF p1 crop과 직접 대조한다.

그 결과에 따라 처리 범위를 한 가지만 택한다.

1. **font manager 미준비**: font fetch/등록 순서 또는 prepared typeface 조회를 고치고, 실제
   `ᄒᆞᆫ` cluster가 shaping 경로로 가는 회귀를 추가한다.
2. **manager는 준비됐지만 glyph가 PDF와 다름**: 공개 font의 glyph 자체 차이인지, font family의
   반환 이름/GSUB 적용 방식이 잘못됐는지 분리한다. 공개 font와 PDF가 본질적으로 다르면 임의
   현대화 치환을 하지 않고 그 한계를 명시한다.
3. **IR text/CharShape 해석 차이**: HWP3뿐 아니라 HWP5/HWPX에서도 같은 title을 재현하는
   공통 display projection 규칙을, 여러 문서의 oracle로 증명한 뒤에만 도입한다.

이 문서는 새 분석 단계의 시작 기록이다. 문서만 따로 commit하지 않고, 다음 보정·focused/e2e
회귀·실제 Studio/PDF visual evidence를 같은 commit으로 남긴다.

## 실제 CanvasKit font manager·shape 판정

동일한 `canvaskit-wasm` 번들로 runtime WOFF2를 직접 열어 확인했다. `FontMgr.FromData()`는 성공했고
내부 family 명은 등록 별칭이 아니라 `Source Han Serif K`였다. renderer는 `create()`에서 이 실제 family
명을 읽어 `renderShapedScriptText()`의 ParagraphStyle에 넘기므로, 별칭 차이로 font manager가 빠지는
경로는 아니었다.

| 확인 | 결과 | 의미 |
| --- | --- | --- |
| `U+1112`, `U+119E`, `U+11AB` glyph ID | 모두 0이 아님 | subset에 옛자모가 실제 포함됨 |
| `ParagraphBuilder.Make(..., oldHangulFontManager)` | 성공 | Studio와 같은 shaping manager 계약이 유효함 |
| `ᄒᆞᆫ` range 0..3 tight rect | 1개 | 자모가 분리된 advance 세 개가 아니라 하나의 cluster로 조판됨 |
| `글(U+AE00)` glyph ID | 0 | subset은 일반 완성형 한글용이 아니라 옛자모 구간 전용임 |

실제 CanvasKit shape와 Hancom PDF p1 title crop은
[`hwp3_password_pdf_p001_title_font_evidence.png`](../report/assets/task_m100_3486/stage9/hwp3_password_pdf_p001_title_font_evidence.png)에
나란히 남겼다. 왼쪽의 `ᄒᆞᆫ`은 한 cluster로 합성되지만 PDF의 제품명 `한` glyph와 모양이 다르다.
PDF는 Microsoft Print To PDF가 만든 CID subset(`CIDFont+F1` 등)이라 원래 한컴/Hanyang face 이름을
추출할 수 없었다.

따라서 이 차이는 font manager 미준비나 CanvasKit glyph fallback 실패가 아니라, 공개 Source Han
옛한글 glyph와 PDF에 인쇄된 한컴 전용 glyph의 차이다. 현재 오라클은 이 `한글 97 안내문` 한 계열뿐이며,
표준 자모열 `ᄒᆞᆫ`을 전역으로 `한`으로 바꾸면 실제 옛한글 내용을 훼손한다. 이 Stage에서는 렌더 문자열을
임의 치환하지 않는다.

## 이 Stage의 회귀 보강과 잔여 경계

`rhwp-studio/e2e/canvaskit-font-coverage.test.mjs`에 실제 subset의 세 자모 glyph, font manager,
Paragraph cluster를 함께 검증하는 회귀를 추가한다. 이 회귀는 전용 font manager가 사라져 tofu나
분리 advance로 후퇴하는 것을 막는다. PDF와 같은 Hanyang glyph 공급 또는 문서별 전용 PUA/glyph 계약은
별도 근거 없이는 이 테스트나 renderer에 추가하지 않는다.

다음 Stage는 이 glyph 차이와 별개로 p3의 표 셀 내용이 검게 채워지는 구조·paint 차이를 분석한다. 그
Stage에서도 먼저 새 분석 문서를 만들고, 원인이 확정될 때만 코드·focused test·page evidence를 같은
commit으로 남긴다.
