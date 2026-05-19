---
name: 로컬 폰트 경로
description: 개발환경에서 TTF 폰트 파일 위치 — 라이선스 이슈로 프로젝트 외부에 분리
type: reference
originSessionId: 4861649d-834a-43c6-a262-9f08333360e8
---
폰트 파일은 프로젝트 외부로 분리되어 있다. 환경별 경로:

- **macOS (현재 환경)**: `/Users/edwardkim/vspace/ttfs`
- **Linux (WSL2)**: `/home/edward/mygithub/ttfs`

SVG 내보내기 시 `--font-path` 옵션으로 해당 경로를 지정한다.
