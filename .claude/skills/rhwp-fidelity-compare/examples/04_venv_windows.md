# 예제 — Windows venv\Scripts\python.exe

이슈 #5329. 실 에이전트 경로. gym 아님.

## 명령

```powershell
py -3.12 -m venv venv
venv\Scripts\python.exe -m pip install pypdf pypdfium2 pillow
venv\Scripts\python.exe tools\fidelity_compare\fidelity_compare.py plan 0 9 `
  --out-dir $env:TEMP\rhwp-fidelity-plan
```

## 하지 말 것

```powershell
# 금지
venv\bin\python tools\fidelity_compare\fidelity_compare.py plan 0 1
python -m pip install --break-system-packages pypdf
```

관련: `references/03_windows.md`.
전사: `fixtures/transcripts/windows_venv.txt`.
정지 F15.
