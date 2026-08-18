# 예제 — RHWP_FONT_PATH_DIR

이슈 #5329. 실 에이전트 경로. gym 아님.

## POSIX

```bash
export RHWP_FONT_PATH_DIR="/opt/hancom/fonts:/usr/local/share/fonts"
venv/bin/python tools/fidelity_compare/fidelity_compare.py korexam 0 2 \
  --out-dir /tmp/korexam-hnc
```

Linux 면 `out-dir/_fontconfig/fonts.conf` 에 `<dir>` 가 생긴다.

## Windows

```powershell
$env:RHWP_FONT_PATH_DIR = "C:\Windows\Fonts;C:\Program Files (x86)\Hnc\Shared\Fonts"
```

`_fontconfig` 가 없는 것이 정상.

글꼴 파일을 커밋하지 않는다.

관련: `references/10_font_path_dir.md`.
정지 F04.
