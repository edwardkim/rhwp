---
kind: memory
status: historical
canonical: mydocs/manual/memory/MEMORY.md
last_verified: 2026-07-26
name: project-wasm-docker-build
description: wasm 빌드는 Docker 경유가 프로젝트 표준 — docker compose --env-file .env.docker run --rm wasm. 네이티브는 로컬 cargo
metadata: 
  node_type: memory
  type: project
---

rhwp 의 WASM 빌드 표준 경로 (2026-07-19 작업지시자 교정으로 확정):

```
docker compose --env-file .env.docker run --rm wasm
```

- Docker 는 **WASM 빌드 전용**, 네이티브 빌드/테스트/분석은 항상 로컬 cargo
  (cli_commands.md:20, rhwp_cli_skill_guide.md 명시)
- `--env-file .env.docker` 필수 — 프로젝트 루트의 `.env` 는 사용자의 로컬
  메모 파일이라 compose 자동 로드가 파싱 실패함 (건드리지 말 것)
- 컨테이너에 wasm-pack 0.15.0 고정 (Dockerfile) — 재현성 목적. 호스트
  wasm-pack/npx 직접 실행은 표준 아님
- wasm_api.rs 에 export 를 추가하는 PR 검증 시: **wasm 재빌드 없이는 studio
  가 스테일 pkg 로 돌아감** — 실기동/시각 확인 전 Docker 빌드 선행
- dev_environment_guide.md(WASM canonical)에 Docker 경로 누락 — 문서 불일치
  보수 대상 (2026-07-19 발견)

관련: [[reference-docker-wsl-recovery]]
