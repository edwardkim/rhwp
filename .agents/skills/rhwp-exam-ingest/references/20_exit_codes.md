# 20 — 종료 코드

helper 와 `rhwp` 의 코드를 섞어 읽지 않는다.

## rhwp (#2707 정본)

`mydocs/manual/cli_commands.md` 종료 코드 표.

| exit | 의미 |
| --- | --- |
| 0 | 성공 |
| 1 | 런타임 (파일 없음, 파싱 실패) |
| 2 | 사용법 (`-o` 누락 등) |
| 3 | `--verify` IR 차이 — **build-from-ingest 에는 보통 없음** |

`build-from-ingest` 는 생성 명령이다. `--verify` 플래그를 발명하지 않는다.
검증은 `export-text` / `dump` 별도 호출.

## pdf_to_pngs.sh

0 성공 · 1 인자/파일 · 2 도구 없음 · 4 DPI 계약.

## crop_image.sh

0 성공 · 1 인자/파일 · 2 ImageMagick 없음 · 3 출력 없음 · 4 bbox 계약.

## extract_docx.py

0 성공(fallback 포함) · 1 인자/파일 · 2 알 수 없는 플래그.

## check_deps.sh

0 필수 충족 (선택 누락 허용) · 1 필수 누락 · 2 사용법.

## 게이트에서

```
set -e 로 helper 를 묶지 마라.
각 호출의 exit 를 읽고 code 봉투를 본다.
exit 4 를 "없는 파일" 로 오해하지 마라. 계약 위반이다.
```

픽스처: `fixtures/matrices/exit_codes.json`.
