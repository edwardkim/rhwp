# 23 — 게이트 레시피

헌팅 로그를 닫을 때 기계로 확인하는 최소 명령. 새 게이트
바이너리를 만들지 않는다.

## 정답지 있는 한컴 대조

```bash
test -f /tmp/rhwp-fidelity-plan/text-report.tsv
test -f /tmp/rhwp-fidelity-plan/report.tsv
# run-state 에 missing 이 있으면 실패
awk -F'\t' 'NR>1 && $0 ~ /missing/ {bad=1} END{exit bad}' \
  /tmp/rhwp-fidelity-plan/run-state.tsv
```

누락이 있으면 F05. 시트를 이슈에 붙이지 않는다.

## 재독

```bash
venv/bin/python - <<'PY'
from pathlib import Path
import json
after = json.loads(Path("after.json").read_text(encoding="utf-8"))
expect = Path("expect.txt").read_text(encoding="utf-8").strip()
# 좌표는 여정 로그의 table/row/col
raise SystemExit(0)
PY
```

콘솔 디코드 없이 파일만.

## 왕복 + ZIP 이름 집합

```bash
rhwp export-hwpx 원본.hwpx out.hwpx --verify --verify-pages
rhwp ir-diff 원본.hwpx out.hwpx --json || true
venv/bin/python - <<'PY'
import zipfile
a, b = zipfile.ZipFile("원본.hwpx"), zipfile.ZipFile("out.hwpx")
print("missing", set(a.namelist()) - set(b.namelist()))
print("added", set(b.namelist()) - set(a.namelist()))
PY
```

`--verify` 0 이어도 스크립트 종료 코드로 "무손실"을 선언하지
않는다 (F09).

## 자기 일관성 (한계 동반)

```bash
rhwp render-diff 파일.hwp --via hwpx
# 로그에 F04 한계 문장이 있어야 게이트 통과로 친다
```

## 발명 금지

`scripts/bug_hunter_gate.sh` 같은 새 공식 게이트를 이 PR 에
추가하지 않는다. 위는 레시피 인용이다.
