---
name: ""
description: 개발환경에서 TTF 폰트 파일 위치 — 라이선스 이슈로 프로젝트 외부에 분리. 환경별 경로 상이
metadata: 
  node_type: memory
  type: reference
  originSessionId: d0e244f3-cdb5-4f3c-bc1d-5ee46e2d3ff2
---

폰트 파일은 프로젝트 외부로 분리되어 있다. 환경별 경로:

- **Linux (WSL2, 현재 환경)**: `/home/edward/mygithub/ttfs`
- **macOS (`ios/devel` 환경)**: `/Users/edwardkim/vspace/ttfs`

SVG 내보내기 시 `--font-path` 옵션으로 해당 환경의 경로를 지정한다.
