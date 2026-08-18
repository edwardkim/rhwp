# 06 — text-report.tsv 문자 멀티셋 (소실/과잉/치환)

`text-report.tsv` 는 기준 PDF 텍스트층과 SVG `<text>` 를 쪽별로
**NFC 정규화한 문자 멀티셋** 으로 비교한다. 공백과 순서는 무시한다.

헤더:

```
page	reference_only	svg_only	reference_only_chars	svg_only_chars	note
```

| 열 | 의미 | 후보 이름 |
| --- | --- | --- |
| `reference_only` | PDF 에만 있는 문자 수 | 소실 |
| `svg_only` | SVG 에만 있는 문자 수 | 과잉 |
| 둘 다 > 0 | 같은 쪽에서 빠지고 들어옴 | 치환 |
| `*_chars` | 코드포인트 요약 | 사람이 볼 표본 |
| `note` | 추출 실패 등 | `-` 이면 정상 비교 |

## 무엇을 알고 무엇을 모르는가

안다:

- 이 쪽에 PDF 엔 있는데 SVG 엔 없는 글자 (각주 누락, caption 누락)
- 이 쪽에 SVG 엔 있는데 PDF 엔 없는 글자 (이른 owner, 중복 paint)
- PUA 가 SVG 에 raw 로 있는지 (`svg-glyph-risk-report.tsv` 가 별도)

모른다:

- 같은 글자가 좌표만 옮긴 것
- 줄바꿈 위치
- 문자 순서 (URL 이 잘린 것은 sequence ledger 가 보완)
- PDF 가 path 로만 그린 장식 글자
- 숨김 텍스트

그래서 멀티셋도 **후보** 다. 최종 시각 판정을 대신하지 않는다.

## --text-only

Chrome·PNG·시트를 만들지 않는다. `pypdf` 만 있으면 된다. 긴 문서의
1차 전수에 맞다.

```bash
RHWP_BIN=target/release-test/rhwp \
venv/bin/python tools/fidelity_compare/fidelity_compare.py 0 214 \
  --source samples/input.hwp \
  --reference-pdf pdf/oracle-2020.pdf \
  --label issue-3738-hwp \
  --reference-grade '한컴 2020 기준 PDF' \
  --text-only --export-all-svg --layout-ledger \
  --out-dir /tmp/rhwp-fidelity-issue-3738
```

`--export-all-svg` 는 쪽마다 rhwp 를 재기동하지 않게 SVG 캐시를 한 번에
채운다. `--layout-ledger` 는 render tree 원장을 같이 남긴다.

## 인접 쪽 보완 원장

멀티셋이 쪽 사이에서 상쇄되면 한 쪽 TSV 는 조용하다. 하네스가 추가로
쓴다.

| 파일 | 잡는 것 |
| --- | --- |
| `text-owner-shift-candidates.tsv` | pN SVG-only ≈ pN+1 PDF-only |
| `text-owner-sequence-candidates.tsv` | 16자+ 순서 보존 문자열이 옆 쪽으로 |
| `page-boundary-fidelity-candidates.tsv` | 위 둘 + 표 fragment 를 한 큐로 |
| `visible-text-excess-candidates.tsv` | clip 안 보이는 SVG 과잉 48자+ |

방향 열 `rhwp_earlier_than_reference` / `rhwp_later_than_reference` 는
단서다. PDF 시각 owner 대조 전에는 결함이 아니다.

## 치환 vs 소실+과잉

```
12	6	6	①②③	□□□	substitution-candidate
18	24	0	각주본문누락		loss-candidate
19	0	24		각주본문과잉	excess-candidate
```

p12 는 원문자가 두부로 바뀐 치환 후보다 (#3385 유형).
p18/p19 쌍은 owner 이동 후보다. shift ledger 를 같이 연다.

## 정규화

`normalized_characters` 는 NFC 와 공백 제거를 한다. 호환 원문자
(①) 와 PUA 원문자는 다른 코드포인트로 남는다. "모양이 같으니 같다"고
합치지 않는다. 그게 tofu 검출의 핵심이다.

## 레시피 — 소실 상위

```bash
# reference_only 내림차순 (헤더 제외)
awk -F'\t' 'NR>1 {print $2+0, $0}' /tmp/rhwp-fidelity-issue-3738/text-report.tsv \
  | sort -nr | head
```

상위 쪽을 시트 모드 범위로 다시 돌린다. 숫자만으로 이슈를 열지 않는다.

## 에이전트 금지

- `reference_only=0, svg_only=0` 을 "시각적으로 동일"로 발표
- 순서 모르는 멀티셋으로 줄바꿈 버그를 확정
- PDF 추출기가 빈 쪽을 "전량 소실"로 이슈화 (note 와 추출 품질을 먼저)
- 원장 파일을 새로 합치는 스크립트를 이 스킬에 추가 (이미 boundary ledger 가 있음)
