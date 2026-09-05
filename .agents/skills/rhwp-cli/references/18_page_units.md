# 페이지 번호와 HWPUNIT

페이지 번호는 **0부터**. 단위는 HWPUNIT.
이 두 가지를 틀리면 올바른 명령을 잘못된 쪽에 쏜다. 결함이 아닌데 결함처럼 보인다.

## 0 기준 축

| 표면 | 필드/플래그 | 기준 |
|---|---|---|
| export-svg/png/pdf/text/markdown | `-p`, pages[].page | 0 |
| dump-pages | `-p` | 0 (페이지) |
| dump | `-s` 구역, `-p` 문단 | 0 (페이지 아님) |
| export-render-tree | `-p` | 0 |
| ir-diff | `-s` 구역, `-p` 문단 | 0 (페이지 아님) |
| export-text --json | pages[].page | 0 |
| search --json | matches[].page | 0 |
| digest --pages a..b | 0, 양끝 포함, a<=b | 0 |

## 1 기준 예외 (이 스킬 기본 축 아님)

`extract-pages --from/--to` 는 **1부터**. `search` 가 `page: 1` 을 주면 여기서는 `--from 2 --to 2`.
차트 `--chart` 도 문서 순서 1부터. 표 `--table` 은 0부터.

## 환산

- 1인치 = 7200 HWPUNIT = 25.4mm = 96px (DPI 96)
- 1px = 75 HWPUNIT
- 1mm ≈ 283.46 HWPUNIT
- 페이지 번호는 0부터. PDF/한컴 표기는 1부터.
- extract-pages --from/--to 만 1 기준. -p 와 혼동하면 한 쪽 밀린다.

- 1인치 = 7200 HWPUNIT = 25.4mm = 96px
- 1px = 75 HWPUNIT
- 1mm ≈ 283.46 HWPUNIT

## 계산 카드

| 입력 | 연산 | 결과 |
|---|---|---|
| 한컴 1쪽 | 1-1 | `-p 0` |
| 한컴 4쪽 | 4-1 | `-p 3` |
| 10mm | 10 × 283.46 | 2834.6 HU |
| 96px (1인치) | 96 × 75 | 7200 HU |
| A4 가로 | 210mm × 283.46 | ≈ 59526 HU (덤프는 59528) |
| A4 세로 | 297mm × 283.46 | ≈ 84188 HU |

덤프 헤더 예: `용지: 210.0mm × 297.0mm (59528×84188 HU)`.
반올림이 있으니 HU 를 mm 로 역산한 뒤 다시 곱해 같기를 기대하지 말 것.

## 함정

- 사용자가 "페이지 0" 이라고 하면 이미 0 기준인지 한 번 확인한다.
- dump 의 `-p` 를 dump-pages 의 `-p` 로 읽으면 문단 3 을 페이지 3 으로 연다.
- PDF 뷰어 쪽번호는 1부터. export-pdf `-p 0` 은 PDF 의 첫 쪽.
- ir-diff `-p` 는 문단이다. 페이지를 좁히려면 먼저 dump-pages 로 문단 번호를 얻는다.
