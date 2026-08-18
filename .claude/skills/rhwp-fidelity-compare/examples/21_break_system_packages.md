# 예제 — --break-system-packages 거절

이슈 #5329. 실 에이전트 경로. gym 아님.

## 상황

PEP 668 로 시스템 pip 가 막히자 사용자가
`pip install --break-system-packages pypdf` 를 요청한다.

## 문장

"저장소 `venv/` 만 사용합니다. `--break-system-packages` 는
이 스킬에서 거절합니다 (F15). POSIX 는 `python3.12 -m venv venv`,
Windows 는 `py -3.12 -m venv venv` 입니다."

관련: `references/02_setup_venv.md`, `03_windows.md`, `14_missing_venv.md`.
정지 F15.
