# 예제 — POSIX venv 설치

이슈 #5329. 실 에이전트 경로. gym 아님.

## 명령

```bash
python3.12 -m venv venv
venv/bin/python -m pip install pypdf pypdfium2 pillow
venv/bin/python -c "import pypdf, pypdfium2, PIL; print('ok')"
venv/bin/python tools/fidelity_compare/fidelity_compare.py plan 0 0 \
  --text-only --out-dir /tmp/rhwp-fidelity-smoke
```

## 읽는 법

시스템 `python3 tools/fidelity_compare/...` 가 아니다.
`--break-system-packages` 가 없다.

관련: `references/02_setup_venv.md`.
정지 F09 예방.
