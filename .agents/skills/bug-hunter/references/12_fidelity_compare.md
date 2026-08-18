# 12 — tools/fidelity_compare

정본: [`tools/fidelity_compare/README.md`](../../../../tools/fidelity_compare/README.md).
이 장은 헌팅 여정에서 빼먹지 말 사용법만 가리킨다. 플래그를
발명하지 않는다.

`fidelity_compare.py` 는 **저장소 도구**이지 `rhwp` 하위명령이
아니다. 비교 전용 하위명령을 rhwp 에 추가하지 않는다 (P15).

## 요구

```bash
python3.12 -m venv venv
venv/bin/python -m pip install pypdf pypdfium2 pillow
# Chrome/Chromium. --text-only 는 pypdf 만
```

Windows: `venv\Scripts\python.exe`. 시스템 Python 에 직접 설치하지
않는다.

실행 파일 탐색: `target/release-test/rhwp` → `target/release/rhwp`
→ PATH. 아니면 `RHWP_BIN` / `CHROME_BIN`.

코드 변경 직후 (이 스킬은 코드를 안 바꾸지만 재현 시):

```bash
cargo build --profile release-test --target-dir target/pr-review
RHWP_BIN=target/pr-review/release-test/rhwp \
  venv/bin/python tools/fidelity_compare/fidelity_compare.py plan 0 9 \
  --out-dir /tmp/rhwp-fidelity-plan
```

`--out-dir` 은 worktree 밖. 시트가 트리를 더럽히지 않게.

## 등록 키

```bash
venv/bin/python tools/fidelity_compare/fidelity_compare.py plan 0 34 \
  --out-dir /tmp/rhwp-fidelity-plan
venv/bin/python tools/fidelity_compare/fidelity_compare.py <키> <시작> <끝>
```

쪽은 0 기준, 끝 포함. 한컴/PDF 8쪽은 7.

## 임의 쌍

```bash
RHWP_BIN=target/release-test/rhwp \
venv/bin/python tools/fidelity_compare/fidelity_compare.py 0 214 \
  --source 'samples/입력.hwp' \
  --reference-pdf 'pdf/한컴-기준.pdf' \
  --label issue-XXXX \
  --reference-grade '한컴 2020 기준 PDF' \
  --text-only --export-all-svg --layout-ledger \
  --out-dir /tmp/rhwp-fidelity-issue-XXXX
```

positional 은 `<시작> <끝>` 뿐. provenance 키를 같은 `--out-dir`
옆에 남긴다.

## 헌팅에서 읽는 파일

1. `provenance.tsv` / 직접 기록한 provenance
2. `run-state.tsv` — 누락이 있으면 종료 코드도 0 이 아님 (F05)
3. `text-report.tsv` — 소실/과잉/치환
4. `report.tsv` — 픽셀 랭킹
5. `page-boundary-fidelity-candidates.tsv` — 사람 감사 큐
6. `--layout-ledger` 후보면 해당 TSV

자동 merge gate 로 쓰지 않는다. 후보 검출이다.

## 관련

- README 가 나열하는 TSV 이름을 여기서 재정의하지 않는다
- 예제: [02_hangul_pdf_compare.md](../examples/02_hangul_pdf_compare.md)
