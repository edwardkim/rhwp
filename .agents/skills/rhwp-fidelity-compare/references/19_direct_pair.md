# 19 — direct pair (`--source` `--reference-pdf` `--label`)

REG 에 없는 HWP/PDF 쌍은 세 플래그를 **모두** 지정한다.
positional 은 `<시작쪽> <끝쪽>` 두 개만.

```bash
venv/bin/python tools/fidelity_compare/fidelity_compare.py 0 214 \
  --source 'samples/입력.hwp' \
  --reference-pdf 'pdf/한컴-기준.pdf' \
  --label issue-3738-hwp \
  --reference-grade '한컴 2020 기준 PDF' \
  --text-only --export-all-svg --layout-ledger \
  --out-dir /tmp/rhwp-fidelity-issue-3738
```

## 계약

| 조건 | 결과 |
| --- | --- |
| 세 플래그 중 일부만 | 사용법 오류: 모두 지정해야 합니다 |
| positional ≠ 2 | direct pair positional 은 두 개 |
| `--reference-grade` + 등록 키 | `--reference-grade` 는 direct pair 전용 |
| source 없음 | `source 파일을 찾을 수 없습니다` exit 2 |
| pdf 없음 | `reference PDF를 찾을 수 없습니다` exit 2 |
| label 비 ASCII | 가능하면 ASCII. 경로 구분자 금지 |

`label` 은 산출 폴더 기본 이름과 provenance 식별자다.
`issue-3738-hwp` 처럼 이슈+형식을 붙인다.

## 시작·끝

0-based, 끝 포함. 214 는 215번째 쪽이다. PDF 보다 크면 overflow
메시지 (15장). 전수를 모르면 먼저 `pypdf` 로 쪽수만 읽는다.

```bash
venv/bin/python - <<'PY'
from pypdf import PdfReader
print(len(PdfReader("pdf/oracle.pdf").pages))
PY
# N 이면 끝 쪽은 N-1
```

암호화면 F13.

## 긴 문서 1차

215쪽 시트를 한 번에 만들지 않는다.

1. `--text-only --export-all-svg --layout-ledger` 전수
2. text-report / boundary / page-count 로 창을 고른다
3. 그 창만 `--out-dir` 같게 시트

`export-all-svg` 캐시가 3 을 싸게 만든다. 바이너리를 바꿨으면
out-dir 을 새로 잡는다.

## 한글 경로

POSIX 는 따옴표. Windows 는 PowerShell 변수 (03장). 배경 cmd 에
한글을 직접 붙이지 않는다.

## 에이전트 금지

- `--source` 만 주고 키가 `0` 으로 오인 (0 은 시작 쪽이다)
- label 에 공백·슬래시
- 사용자 PDF 를 등급 문장 없이 `한컴 2022 기준 PDF` 로 적기
- 원본을 `--out-dir` 에 복사해 덮어쓸 여지를 만들기
