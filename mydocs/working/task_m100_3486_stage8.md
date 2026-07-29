---
kind: investigation
status: active
canonical: mydocs/manual/codex/docs_and_git_workflow.md
last_verified: 2026-07-29
---

# Task #3486 Stage 8 — 전수 PDF 대조 뒤 공통 옛한글 조판 원인 분리

- 이슈: [#3486](https://github.com/edwardkim/rhwp/issues/3486)
- 브랜치: `task_m100_3486_hwp3_render_fidelity_v2`
- 선행 커밋: `a82b82ffb` (Stage 7: HWP3 hyperlink 포함 줄 좌표 reset의 페이지 경계 보존)
- 기준 오라클: `pdf/HWP3-password-123456.pdf` (한컴 PDF, 24쪽)
- 직접 입력: `samples/HWP3-password-123456.hwp` (암호 HWP3, 24쪽)

## 시작 근거: 24쪽 1:1 대조는 했지만 수용 상태는 아니다

Stage 7에서 암호 HWP3 원본을 재직렬화하지 않고 같은 144 DPI로 한컴 PDF와 24쪽 전부 1:1
raster 대조했다. Stage 7의 hyperlink 줄 경계 보정 뒤 17·18쪽은 PDF의 내용 흐름에 맞게
복구됐지만, 문서 전체는 아직 1:1 수용 상태가 아니다.

특히 1쪽 title, 3쪽 표, 10쪽 legacy glyph/목록, 24쪽 목록·hyperlink의 차이가 남는다. 전수
contact sheet와 page 17·18 actual review PNG는 Stage 7 커밋의 아래 파일에 **실제 이미지로**
보존되어 있다.

- [24쪽 review contact sheet](../report/assets/task_m100_3486/stage7/hwp3_password_pdf_all_pages_review.png)
- [24쪽 overlay contact sheet](../report/assets/task_m100_3486/stage7/hwp3_password_pdf_all_pages_overlay.png)
- [17쪽 review](../report/assets/task_m100_3486/stage7/hwp3_password_pdf_p017_review.png)
- [18쪽 review](../report/assets/task_m100_3486/stage7/hwp3_password_pdf_p018_review.png)

전체 픽셀 일치율은 백지 영역의 영향을 크게 받으므로 수용 근거가 아니다. Stage 7의 잉크 영역
proxy도 1쪽 16.04026%, 3쪽 6.82176%, 10쪽 6.50127%, 24쪽 8.83464%로, 페이지별 실제
대조가 계속 필요함을 보여 준다.

## 같은 논리 문서 네 형식의 공통성

다음 네 입력은 같은 `한글 97 안내문`을 포맷·암호화만 달리 저장한 계열이다.

| 입력 | 화면/IR에서 확인한 1쪽 제목 계열 | 페이지 수 | 이 Stage에서의 의미 |
| --- | --- | ---: | --- |
| `HWP3-password-123456.hwp` | `ᄒᆞᆫ글 97 안내문` | 24 | PDF 직접 오라클 입력 |
| `HWP5-nopassword-123456.hwp` | `ᄒᆞᆫ글 97 안내문` | 24 | HWP3 page flow 비교 입력 |
| `HWP5-nopassword-123456.hwpx` | `ᄒᆞᆫ글 97 안내문` | 23 | 동일 문서의 평문 XML 계열 |
| `HWP5-password-123456.hwpx` | `ᄒᆞᆫ글 97 안내문` | 23 | 동일 문서의 암호 XML 계열 |

따라서 title의 `ᄒᆞᆫ`을 HWP3 parser에서만 현대 `한`으로 치환하는 것은 올바른 해법이 아니다.
원문 IR을 바꾸면 실제 옛한글 문서의 의미를 훼손할 수 있으며, 네 형식 공통의 renderer·font
계층 차이도 숨긴다.

## `hypua2jamo` 범위와 현재 경로

프로젝트의 `src/renderer/pua_oldhangul.rs`는 이미 KTUG Hanyang PUA 표(공개 도메인)를
사용하며, 그 원본으로 `mete0r/hypua2jamo`의 `hypua2jamocomposed.txt`를 명시한다. 즉 한컴
PUA(U+E000–U+F8FF)에서 KS X 1026-1 자모열로의 일반 변환은 이 프로젝트에 이미 존재한다.

그러나 title의 `ᄒᆞᆫ`은 이미 표준 옛한글 자모(U+1100–U+11FF) 조합이다. 이 경우
`hypua2jamo`에는 역으로 현대 음절 `한`으로 바꿀 정보도, 한컴 PDF와 동일한 glyph/advance를
주는 글꼴도 없다. 따라서 PUA 매핑을 문서별로 추가하거나 `ᄒᆞᆫ → 한`을 전역 치환하지 않는다.

현재 renderer는 표준 옛한글 자모가 있는 cluster에 `Source Han Serif K Old Hangul`을 우선
선택한다(`contains_old_hangul_jamo()` 및 SVG/Canvas font chain). 이는 tofu를 피하는 필요한
fallback이지만, 다음을 별도로 검증해야 한다.

1. Studio CanvasKit에 실제 해당 typeface가 등록되어 있는지와 SVG/Canvas의 fallback 순서가 같은지
2. title의 CharShape가 요구한 원래 face·weight·장평·자간과 fallback의 advance가 어떤 차이를 내는지
3. 조합 자모 cluster의 shaping/baseline이 PDF의 한컴 legacy font와 달라지는지

## HWP3 1쪽의 독립 결함: 쪽번호 위치 control이 title 앞에 남긴 가시 marker

같은 1쪽 title 문단(구역 0, 문단 1)을 parser dump로 대조했다.

| 입력 | 본문 text | control | line segment |
| --- | --- | --- | --- |
| HWP3 | `U+FFFC + ᄒᆞᆫ글 97 안내문` | `PageNumberPos`, `Header` | `ts=0, vpos=2168` |
| HWP5 | `ᄒᆞᆫ글 + U+2007 + 97 안내문` | `PageNumberPos`, `Header` | `ts=0, vpos=2168` |

두 문단 모두 같은 쪽번호 위치·머리말 control을 가지는데, HWP3만 title 앞에 object-replacement
marker가 남아 있다. 이 marker는 title의 논리 문자가 아니므로 Studio 화면의 title 앞 세로선/공백과
HWP3만 더 심한 첫 글자 차이의 독립 원인이다.

원인은 `parse_field_control_char()`의 HWP3 control 18–21 공용 처리다. `PageNumberPos`(20)는
HWP5와 달리 가시 inline object가 아닌 document-level 설정인데도, 현재 코드는 `U+FFFC` 하나와
1 UTF-16 advance를 추가한다. `Header`(16)는 별도 parser 경로에서 이미 marker를 만들지 않는다.

따라서 첫 보정은 **HWP3 control 20에만** text·`char_offsets`·UTF-16 advance를 만들지 않는 것이다.
새 actual fixture 회귀는 다음을 보장해야 한다.

1. HWP3 title 문단은 `PageNumberPos` control을 계속 보존한다.
2. title의 첫 가시 문자는 `ᄒ`이며 `U+FFFC`가 앞에 남지 않는다.
3. HWP3 p1 title run의 첫 x와 line segment의 `ts=0` 계약은 유지한다.
4. page-number-position control 외 18/19/21의 기존 placeholder·offset 해석은 바뀌지 않는다.

이 parser 보정은 HWP3 전용 marker 결함만 다룬다. `ᄒᆞᆫ`의 공통 font/shaping 차이와 표·목록 차이는
여전히 다음 Stage 대상으로 남긴다.

## 병행 변경과의 경계

현재 worktree에는 Stage 8 소유가 아닌 HWP3 drawing/reference-position 및 SVG·Skia·Canvas
background display 변경이 있다. Stage 7 재대조에서 1쪽 raster가 달라진 것은 이 병행 변경의
영향일 수 있으며, Stage 7의 typeset 변경은 17·18쪽 page item만 바꿨다.

따라서 이 Stage는 다음을 하지 않는다.

- 병행 변경을 되돌리거나 이 Stage 커밋에 섞지 않는다.
- p1 전체 proxy 변화를 Stage 7 또는 Stage 8의 glyph 결론으로 귀속하지 않는다.
- HWP3 한정 parser 치환으로 네 형식 공통 title 차이를 감추지 않는다.

## 다음 구현 전 판별 계약

다음 코드 변경 전에 title glyph의 CharShape/font-face와 SVG·Studio CanvasKit의 실제 선택 face를
동일 입력에서 기록한다. 그 결과에 따라 아래 중 하나만 택한다.

1. **font asset/등록 결함**이면 해당 face의 로딩·unicode-range·fallback 우선순위를 공통 renderer
   경로에서 최소 범위로 보정하고, HWP3/HWP5/HWPX의 title을 함께 회귀로 고정한다.
2. **shaping/metric 결함**이면 원문 `run.text`는 보존한 채 paint/display 경로의 cluster shaping과
   advance만 보정한다.
3. PDF가 한컴 독점 legacy glyph를 쓰고 공개 font로 동등 재현할 방법이 없으면, 그 차이를 명시적으로
   남기고 table/list/page-flow처럼 독립적으로 검증 가능한 결함부터 다음 Stage에서 처리한다.

어느 경우에도 이 분석 문서만 별도 커밋하지 않는다. 위 판별 뒤의 코드·focused 회귀·실제 PNG
대조 결과를 이 문서와 같은 커밋에 남긴다. 그 커밋 후 새 잔여 결함이 남으면 다음 Stage 분석 문서를
먼저 만들고 같은 순서를 반복한다.

## 구현·회귀·24쪽 재대조 결과

`parse_field_control_char()`에서 control 20(`PageNumberPos`)만 본문 text, `char_offsets`, UTF-16
advance를 만들지 않게 했다. control 객체와 그 값 자체는 그대로 보존한다. control 18/19/21의 기존
공백·marker·offset 처리는 변경하지 않았다.

독립 actual fixture 회귀:

```bash
CARGO_TARGET_DIR=target/task_3486_render_v2 CARGO_INCREMENTAL=0 \
  cargo test --profile release-test --test hwp3_page_number_pos_fixture
# 1 passed; 0 failed
```

같은 암호 HWP3 원본과 한컴 PDF를 144 DPI, pixel-diff threshold 32로 다시 24쪽 모두 대조했다.
legacy PDF text layer 때문에 `pdftotext -bbox-layout`은 여전히 abort했으나, SVG export, PDF raster,
compare, overlay, review contact sheet는 24쪽 모두 생성했다. 다음 PNG는 링크만 남긴 것이 아니라
저장소에 실제로 포함한다.

- [24쪽 review contact sheet](../report/assets/task_m100_3486/stage8/hwp3_password_pdf_all_pages_review.png)
- [24쪽 overlay contact sheet](../report/assets/task_m100_3486/stage8/hwp3_password_pdf_all_pages_overlay.png)
- [1쪽 review](../report/assets/task_m100_3486/stage8/hwp3_password_pdf_p001_review.png)

| 항목 | Stage 7 | Stage 8 | 판정 |
| --- | ---: | ---: | --- |
| p1 내용 픽셀 proxy | 16.04026% | 16.57286% | title 앞 marker 제거로 개선 |
| p1 전체 픽셀 일치율 | — | 86.92644% | 백지 영향 때문에 수용 근거 아님 |
| 전체 평균 내용 픽셀 proxy | 10.75278% | 10.77497% | 미세 개선 |
| 최저 내용 픽셀 proxy | 6.50127% | 6.50127% | p10 등 잔여 결함은 그대로 |
| Stage 7 대비 rhwp raster hash | — | p1만 변경 | 2–24쪽은 byte-identical |

현재 structural heuristic은 모든 페이지에서 flag를 내지 않았지만, 이는 font/glyph·표 내용의 실제
차이를 충분히 검출하지 못하므로 1:1 수용 근거가 아니다. p1 review에서 marker는 사라졌고 title의
시작 x도 HWP5와 같은 251.9px 계열로 복구됐다. 그러나 PDF와 rhwp 사이의 title glyph·본문 metric,
삽화/TOC geometry, p3 표, p10·p24 legacy text/목록 차이는 남아 있다.

따라서 이 커밋의 완료 범위는 **HWP3 PageNumberPos의 잘못된 가시 marker 제거**다. 전체 24쪽 PDF
1:1 수용은 아직 완료되지 않았으며, 다음 Stage에서 공통 old-Hangul shaping/폰트와 표·목록을 이
HWP3 parser 결함과 분리해 계속 다룬다.
