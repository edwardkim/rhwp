# 09 — 두부: PUA · U+FFFD · □

한컴 전용 글리프와 공개 글꼴의 `.notdef` 가 섞이면 시트가 두부로
보인다. 이 장은 **문서 결함인지 하네스 오염인지** 를 가른다.

정본: README 의 `#3385` 실측, `svg_glyph_risks` 주석 (U+F02FB,
U+F02B1–F02C4).

## 세 종류의 네모

| 보이는 것 | 코드포인트 | 원장 | 보통 의미 |
| --- | --- | --- | --- |
| □ (U+25A1) | 텍스트층에 네모 문자 | text-report 치환 | 대체 문자로 이미 들어감 |
| 빈 상자 `.notdef` | 글리프가 없음 | 시트에만, 텍스트는 살아 있음 | 글꼴 매칭/EBDT |
| 원문자 자리 깨짐 | PUA U+E000–F8FF, U+F0000+ | `svg-glyph-risk-report.tsv` | 한컴 전용 glyph |
| `` | U+FFFD | 같은 원장 | 디코더가 이미 잃음 |

한 시트에 여러 종류가 같이 있을 수 있다. 하나로 뭉개지 말 것.

## svg-glyph-risk-report.tsv

```
page	risk_count	glyphs	note
1	0	-	-
12	14	U+F02B1×7,U+F02C4×7	raw PUA 또는 U+FFFD — 공개 글꼴에서 두부 후보
```

이 원장은 PDF 텍스트 추출과 **독립** 이다. 한컴 PDF 가 전용 글꼴을
추출하지 못해도, rhwp SVG 에 raw PUA 가 있으면 후보가 남는다.
#2007 의 U+F02FB 총알이 이 경로다.

`risk_count=0` 인데 시트가 온통 네모면 **PUA 가 아니라 매칭 실패**
다. F14 로 간다.

## #3385 유형 (승격 후보)

CharOverlap 문맥의 원문자 U+F02B1~F02C4 가 공개 글꼴에서 두부가 된다.
text-report 는 `reference_only` 에 ①류, `svg_only` 에 □ 가 같이 나와
치환 후보가 된다. 시트에서 그 자리만 깨져 있으면 문서/렌더 후보다.
유지자가 이슈로 승격한다. 에이전트는 원장+시트를 첨부만 한다.

## 하네스 오염 유형 (승격 금지)

- 본문 한글 전부가 네모
- glyph-risk 가 비어 있음
- `RHWP_FONT_PATH_DIR` 없이 CI 러너에서 돌림
- HMKMM 을 Chrome 이 골라 `.notdef`

처방: 글꼴 경로를 넣고 같은 범위를 다시 돈다. 17장.

## 레시피

```bash
# 1) 리스크 원장
column -t -s $'\t' /tmp/rhwp-fidelity-plan/svg-glyph-risk-report.tsv | head
# 2) 치환 쪽
awk -F'\t' 'NR>1 && $2+0>0 && $3+0>0 {print}' \
  /tmp/rhwp-fidelity-plan/text-report.tsv
# 3) 시트 한 장
# cmp-p012.png 를 열어 본문 vs 원문자만 구분
```

Windows:

```powershell
Import-Csv -Delimiter "`t" $env:TEMP\rhwp-fidelity-plan\svg-glyph-risk-report.tsv |
  Where-Object { $_.risk_count -ne '0' -and $_.risk_count -ne '-' }
```

## 에이전트가 이슈 본문에 적을 것

```
페이지: p12
원장: svg-glyph-risk-report 14, text-report ref_only=6 svg_only=6
시트: cmp-p012.png (본문은 살아 있고 원문자만 □)
글꼴: RHWP_FONT_PATH_DIR=... (재실행 후에도 동일)
provenance: 한글 2022 / 파일→PDF / pdf/....-2022.pdf
판정 요청: 유지자. 자동 승격 아님.
```

본문이 전부 네모면 이 템플릿을 쓰지 말고 F14 를 쓴다.
