# 예제 — venv 없음

이슈 #5329. 실 에이전트 경로. gym 아님.

## 증상

```
pypdf가 필요합니다: python -m pip install pypdf
$ echo $?
2
```

## 하지 말 것

```bash
python3 -m pip install --break-system-packages pypdf
```

## 처방

저장소 `venv` 를 만들고 `venv/bin/python` 또는
`venv\Scripts\python.exe` 로 다시 친다.

관련: `references/14_missing_venv.md`, `02_setup_venv.md`.
전사: `fixtures/transcripts/missing_venv.txt`.
정지 F09.
