# Task M100 #1189 Stage 1

## 목적

Task #1139 병합 이후 후속 보정으로, `3-09월_교육_통합_2023.hwp` 19쪽이 한컴/PDF 기준과 맞지 않는 문제를 분석한다.

## 시작 기준

- 이슈: [#1189](https://github.com/edwardkim/rhwp/issues/1189)
- 기준 브랜치: 최신 `upstream/devel`
- 작업 브랜치: `local/task_m100_1189`
- 대상 HWP: `samples/3-09월_교육_통합_2023.hwp`
- 권위 PDF: `pdf/3-09월_교육_통합_2023.pdf`
- 대상 페이지: 화면 기준 19쪽
  - `export-svg`/`dump-pages`: `-p 18` (0-index)
  - `pdftoppm`: `-f 19 -l 19` (1-index)

## 재현 및 분석 산출물

- 산출물 디렉터리: `output/task1189_stage1_page19_analysis/`
- 현재 SVG: `output/task1189_stage1_page19_analysis/3-09월_교육_통합_2023_019.svg`
- 현재 PNG: `output/task1189_stage1_page19_analysis/rhwp_page19.png`
- 디버그 PNG: `output/task1189_stage1_page19_analysis/rhwp_page19_debug.png`
- PDF 19쪽 PNG: `output/task1189_stage1_page19_analysis/pdf_page19-19.png`
- 좌우 비교 PNG: `output/task1189_stage1_page19_analysis/rhwp_vs_pdf_page19.png`
- 페이지 dump:
  - `output/task1189_stage1_page19_analysis/dump_page18.txt`
  - `output/task1189_stage1_page19_analysis/dump_page19.txt`
  - `output/task1189_stage1_page19_analysis/dump_page20.txt`

## 실행 명령

```bash
cargo run --bin rhwp -- export-svg samples/3-09월_교육_통합_2023.hwp -p 18 -o output/task1189_stage1_page19_analysis
cargo run --bin rhwp -- export-svg samples/3-09월_교육_통합_2023.hwp -p 18 -o output/task1189_stage1_page19_analysis/debug --debug-overlay
cargo run --bin rhwp -- dump-pages samples/3-09월_교육_통합_2023.hwp -p 18
pdftoppm -f 19 -l 19 -r 144 -png pdf/3-09월_교육_통합_2023.pdf output/task1189_stage1_page19_analysis/pdf_page19
rsvg-convert output/task1189_stage1_page19_analysis/3-09월_교육_통합_2023_019.svg -o output/task1189_stage1_page19_analysis/rhwp_page19.png
rsvg-convert output/task1189_stage1_page19_analysis/debug/3-09월_교육_통합_2023_019.svg -o output/task1189_stage1_page19_analysis/rhwp_page19_debug.png
```

## 관찰 결과

- 현재 rhwp 19쪽은 PDF 19쪽 대비 우측 단의 `문29)` 이하가 아래로 밀려 있다.
- PDF 기준으로는 19쪽 우측 단에 `문29)` 본문과 그림이 함께 들어오지만, 현재 rhwp는 `문29)`가 하단에 걸리고 이후 내용이 20쪽으로 넘어간다.
- `export-svg` 과정에서 19쪽 overflow가 직접 기록된다.
  - `pi=935 line=1`: 단0 하단 `4.1px` overflow
  - `pi=952 line=0`: 단1 하단 `17.6px` overflow
  - `pi=953 line=0`: 단1 하단 `41.1px` overflow
- `dump-pages -p 18` 기준 19쪽은 `pr.endnote_paragraphs`에 합쳐진 미주 흐름이다.
  - 19쪽 단0: `pi=921`부터 시작, `pi=935`가 단0/단1에 걸쳐 분할됨
  - 19쪽 단1: `pi=946`에서 `문29)` 시작, `pi=951` 그림, `pi=952`, `pi=953`이 페이지 하단을 넘김
  - 20쪽 단0: `pi=953 lines=1..2`부터 이어짐
- 따라서 이번 문제의 1차 범위는 일반 본문 조판이 아니라, Task #1139에서 본격 처리된 미주 흐름의 페이지/단 분할과 TAC 그림 주변 높이 산정이다.

## 잠정 판단

19쪽 불일치는 `문29)` 자체만의 문제가 아니라, 18~20쪽에 이어지는 미주 단 흐름에서 누적 높이가 PDF/한컴 기준보다 크게 잡혀 `문29)` 배치가 늦어지는 형태로 보인다. 특히 `pi=935` 분할 이후 19쪽 단1의 `pi=940`, `pi=951` TAC 그림과 주변 문단 높이/여백 산정이 우선 조사 대상이다.

## 다음 분석 항목

1. `pi=935`, `pi=940`, `pi=946`, `pi=951`, `pi=952`, `pi=953`의 문단 측정 높이와 그림 bbox를 분리해 확인한다.
2. 19쪽 단1에서 `문29)` 시작 y 좌표와 `pi=951` 그림 y 좌표를 PDF 기준과 정량 비교한다.
3. 미주 흐름에서 TAC 그림 주변 paragraph spacing, line segment vpos, partial paragraph 분할이 일반 본문과 다르게 처리되는지 확인한다.
4. 원인이 좁혀진 뒤 회귀 테스트를 추가하고, 소스 수정은 작업지시자 승인 후 진행한다.

## 현재 상태

- 2026-05-31: 이슈 #1189를 확인하고 최신 `upstream/devel` 기준 `local/task_m100_1189` 브랜치를 생성했다.
- 2026-05-31: 19쪽 SVG/PDF 비교 산출물과 18~20쪽 `dump-pages` 산출물을 생성했다.
- 2026-05-31: 19쪽 문제는 미주 흐름의 누적 높이/단 분할 불일치로 1차 분류했다.
