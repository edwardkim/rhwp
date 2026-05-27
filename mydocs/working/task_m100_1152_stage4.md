# Task #1152 Stage 4 — 시각 검증

- 이슈: [#1152](https://github.com/edwardkim/rhwp/issues/1152)
- 브랜치: `local/task1152`
- 작성일: 2026-05-28

## 1. SVG 내보내기

```
rhwp export-svg "samples/2022년 국립국어원 업무계획.hwp" -o /tmp/task1152_svg/ -p 31
rhwp export-svg "samples/2022년 국립국어원 업무계획.hwp" -o /tmp/task1152_svg/ -p 32
```

생성 파일:
- `/tmp/task1152_svg/2022년 국립국어원 업무계획_032.svg` (페이지 32, 610 KB)
- `/tmp/task1152_svg/2022년 국립국어원 업무계획_033.svg` (페이지 33, 218 KB)

## 2. 문자 수준 검증 (별첨 텍스트 존재 여부)

```bash
for c in 별 첨; do
  grep -o ">$c<" "...page_032.svg" | wc -l
  grep -o ">$c<" "...page_033.svg" | wc -l
done
```

| 글자 | page 32 | page 33 |
|------|---------|---------|
| 별 | 0 | 2 |
| 첨 | 0 | 2 |

→ 별첨 박스가 페이지 32 에는 없고, 페이지 33 에 등장. (2회는 SVG 렌더링 분리 셀의 ascii art `[ ]` 영역 표시 + 텍스트 셀 의 합산으로 추정 — 본질은 페이지 분리 자체)

## 3. dump-pages 페이지별 콘텐츠 순서

### page 32 (page_num=30)
```
items=1, used=915.1px
  PartialTable   pi=586 ci=0  rows=8..12  cont=true  12x5
```

### page 33 (page_num=31)
```
items=24, used=869.5px (hwp_used≈861.9px, diff=+7.6px)
  Table          pi=586 ci=1  1x3  635.8x38.9px  wrap=TopAndBottom tac=true   ← 별첨 박스
  FullParagraph  pi=587  h=13.3  "(빈)"
  Table          pi=588 ci=0  1x2  642.5x43.5px  wrap=TopAndBottom tac=true   ← "1 연혁 및 임무" 헤더
  FullParagraph  pi=589  h=13.3  "(빈)"
  FullParagraph  pi=590  h=21.3  "□ 연 혁"
  FullParagraph  pi=591  h=34.0  " ㅇ 1984.  5.  국어연구소 개소..."
  ...
  FullParagraph  pi=602  h=21.3  "□ 임 무"
  FullParagraph  pi=603  h=34.0  " ㅇ 국어 정책 수립에..."
  ...
```

## 4. 한컴 한글 2022 PDF (`pdf/2022년 국립국어원 업무계획-2022.pdf`) 정합

| 페이지 | PDF (정답지) | rhwp 패치 후 |
|--------|--------------|------------|
| 32 (page_num=30) | 12×5 표 마지막 행 `함께 누리는 수어·점자 문화 확산`. 별첨 없음 | ✅ 동일 |
| 33 (page_num=31) | 상단부터 `별첨 \| 국립국어원 일반현황` → `1 연혁 및 임무` → `□ 연 혁` → 연표 (1984~2016) → `□ 임 무` → 임무 항목 | ✅ 동일 |

## 5. 결론

- 패치 적용 후 페이지 32, 33 의 콘텐츠 분리 한컴 PDF 와 시각·구조 정합.
- 별첨 박스가 페이지 33 첫 항목으로 이동.
- 후속 항목 (1 연혁 및 임무 표, 연혁/임무 항목들) 도 정상 흐름.

→ Stage 5 (최종 보고서 + merge) 로 진행.
