---
kind: guide
status: active
canonical: mydocs/manual/verification/hft_original_font_supply.md
last_verified: 2026-10-16
---

# 독립 전체 한글 표본으로 원본 HFT 글꼴 공급 준비

원본이 HFT `한양신명조`를 요청하지만 공급한 TTF가 실제 HFT 윤곽과 다를 때 사용하는
선택 절차다. `scripts/prepare_hft_hangul_font.py`가 **별도의 전체 한글 알파벳 표본**을
작성하고, 그 표본의 한컴 Print Type3 윤곽을 로컬 CFF 글꼴로 옮긴다.
HFT 전체 형식이나 모든 HFT face를 지원하는 변환기가 아니다.

이 도구는 대상 문서나 대상 정답 PDF에서 글자를 가져오지 않는다. 표본은 U+AC00–U+D7A3
11,172자를 순서대로 모두 담아야 한다. `restore`는 PDF에서 추출한 문자가 이 전체 순서와
다르면 거부하므로 일반 문서의 정답 PDF를 원본 글꼴 자료로 사용할 수 없다.

## 준비와 독립 표본 출력

Python 환경에는 `fonttools`와 `PyMuPDF`가 필요하다. 저장소 루트에서 실행한다.

```bash
venv/bin/python scripts/prepare_hft_hangul_font.py --help

venv/bin/python scripts/prepare_hft_hangul_font.py create-specimen \
  --template samples/hwpx/blank_hwpx.hwpx \
  --out output/pr-review/<id>/hft-alphabet.hwpx
```

기존 문서의 저장 줄·본문을 제거하고, 원본 HFT face와 전체 한글만 넣는다.
`*.provenance.json`에 template/input SHA-256·요청 face·문자 수를 기록한다.

이 **별도 표본**을 해당 한컴에서 Print 출력한다.
[MCP 변환 사용법](../mcp_hwp2024Convert_usage.md)의 `start → status → download`와
Print 실행 계약을 따른다. 원본 HFT face가 실제 공급되었는지 확인하고 engine·제품/빌드·
PrintMethod·출력 모드·job·입력/PDF SHA·쪽수를 기록한다.
예제 표본은 `engine: 2020`으로 출력하며 이름은 `hft-alphabet-2020.pdf`로 정한다.
PDF는 로컬 `pdf/` 아래에 보존하고 Git 밖의 비공개 경로나 `.git/info/exclude`로 공개 제출에서 제외한다.

## 원본 윤곽과 전진폭 보존

```bash
venv/bin/python scripts/prepare_hft_hangul_font.py restore \
  --specimen-pdf pdf/<local-only>/hft-alphabet-2020.pdf \
  --out output/pr-review/<id>/original-hft-font/HGSMJ-original.otf
```

전체 표본의 Type3만 읽는다. 원본 `FontMatrix=0.001`·전진폭과 move/line/cubic/close 경로를
같은1000em CFF 좌표로 옮긴다. 이동·맞춤·획 변경·점수에 따른 보정은 하지 않는다.
지원하지 않는 행렬·연산, 충돌하는 중복 glyph program, 누락된 한글은 실패다.
각 원본 glyph program과 PDF/생성 글꼴 SHA는 출력의 `*.provenance.json`에 보존한다.

CFF의 실제 왼쪽 끝 `xMin`은 `hmtx`의 왼쪽 side bearing에도 반올림해 기록한다.
모든 bearing을0으로 넣으면 이 테이블을 읽는 소비자와 CFF에서 직접 경계를 읽는 소비자의
계약이 달라진다. 원본 윤곽은 이동하지 않는다.
[OpenType hmtx 사양](https://learn.microsoft.com/en-us/typography/opentype/spec/hmtx)의
CFF 윤곽과 hmtx bearing 일치 조건을 따른다.

범위는 한양신명조 정규체의 현대 한글이다. ASCII·기호·PUA·독립 bold/italic face는 만들지 않으며
그 글자들은 기존 실제 공급 사다리를 사용한다. 생성 글꼴을 이름만 같은 임의 TTF와 동등하다고
추정하지 않는다. 준비 뒤 실제 원본 HFT 윤곽과 같은지, 외곽·전진폭·기준선과 대표 PNG를 대조한다.
원본 윤곽·매핑을 확인하지 못한 범위는 미검증이다.

## Visual Sweep에 공급

준비한 글꼴을 시스템에 설치하지 않고 `--font-path`로 명시한다.

```bash
venv/bin/python scripts/visual_sweep.py \
  --rhwp-bin target/pr-review/debug/rhwp --rasterizer png \
  --embed-fonts full --font-path output/pr-review/<id>/original-hft-font \
  --compat 2022 --silhouette-only --page-workers 4 \
  --out output/pr-review/<id>/native \
  --file-target sample samples/<source>.hwpx pdf/<independent-print>-2020.pdf
```

fresh WASM은 같은 원본·PDF·글꼴·DPI·compat로 위 명령에 `--wasm-pkg <fresh-package>`를
추가하고 다른 output에 산출한다. 원본 HFT 공급을 마친 뒤에는 그 파일 바이트를 고정한다.
동일한 글꼴을 쓰는 검증이 실행 중일 때 다시 생성하거나 덮어쓰지 않는다.

**기존 대체 TTF 결과와 원본 HFT 결과를 섞지 않는다.** 이전 결과·차이 원인을 보존하고
같은 원본 HFT를 공급한 고정 devel/head의 Native/fresh WASM 전쪽 TSV를 모두 새로 산출한다.
원본 입력·독립 정답 PDF·DPI·비교 마스크·허용치는 유지한다. 글꼴 공급 보완을 하락의 면제나
기준값 완화로 사용하지 않는다. 전쪽90%·devel 대비0.1%p·완료 쪽98%·쪽수/내용·확대 판독은
[시각 검증 거버넌스](visual_verification_governance.md)를 그대로 적용한다.

글꼴과 표본 PDF는 로컬에만 보관한다. 공개 증적에는 생성 절차·입력/글꼴/Print 해시,
실제 선택 face·전쪽 TSV와 대표 PNG를 연결한다. 이 절차를 쓰지 않는 실행과 일반 Studio의
기존 fallback 환경은 별도 검증 범위로 기록한다.
