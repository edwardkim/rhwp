# Task #4089 Stage 1 — Windows Docker WASM 산출물 격리

- Issue: [#4089](https://github.com/edwardkim/rhwp/issues/4089)
- Base: `upstream/devel` `c121f6185`
- Branch: `codex/issue-4089-wasm-docker`
- 측정일: 2026-08-14 KST

## 구현

`wasm` compose 서비스는 이제 `/build-target` named volume을 `CARGO_TARGET_DIR`로 사용한다.
따라서 Windows Docker Desktop의 `/app/target` bind mount hard-link 제약과 분리되며, volume은
컨테이너 실행 사이에 Cargo 산출물을 보존한다.

서비스는 root로 named volume을 쓸 수 있게 실행하되, EXIT trap으로 `pkg/` 전체를
`HOST_UID`/`HOST_GID`로 되돌린다. 이 trap은 wasm-pack 성공·실패·중단 시에도 실행된다. 빌드
직전에는 이전 중단이 남긴 `pkg/*-opt.wasm`만 제거한다. 소스·정상 WASM 산출물·CI workflow는
삭제하거나 변경하지 않는다.

개발 가이드는 Docker를 표준 경로로 승격하고, Windows 네이티브 실행은 `--no-opt` 진단 전용으로
제한했다. 이를 통해 stale `*-opt.wasm`을 다시 입력으로 집는 루프를 피하면서도 Docker가 없는
호스트에서 Rust→WASM 컴파일과 wasm-opt를 분리할 수 있다.

## 로컬 검증

- `python scripts\\tests\\test_docker_wasm_compose.py` — 4/4 통과. named target volume,
  stale `*-opt.wasm` 정리 순서, exit 시 ownership 복원, Docker 우선/`--no-opt` 진단 문서의
  정적 계약을 확인했다.
- `git diff --check` — 통과.
- `wasm-pack --version` — 이 Windows host는 `0.14.0`이고 저장소 고정판은 `0.15.0`이다.
  따라서 네이티브 wasm-pack 실행 결과를 릴리스 검증으로 기록하지 않았다.
- `docker`/`docker compose` — 실행 파일이 없고 `.env.docker`도 없어서 컨테이너 실행은
  수행하지 못했다. `.env.docker`의 값을 읽거나 새 파일을 만들지 않았다.

## 후속 검증 경계

PR CI 또는 Docker Desktop이 있는 Windows host에서 아래 표준 명령을 두 번 순차 실행해 named
target cache 재사용과 stale file 정리를 실제로 확인해야 한다.

```powershell
docker compose --env-file .env.docker run --rm wasm
```

이 Stage는 #4089만 다룬다. renderer·parser·Hancom 기준 이슈는 닫지 않으며, 이 이슈도 merge와
실행 검증 전에는 종료하지 않는다.
