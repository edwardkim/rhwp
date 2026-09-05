# 예외 경로 — 바이너리 없음 (`missing_binary`)

증상: 닥터 종료 코드 **3**, 리포트 `binary.found==false`,
`exceptions[].kind=="missing_binary"`.

이것은 건강 실패(1)가 아니다. **아직 빌드하지 않았다**는 신호다.

## 즉시 할 일

```bash
cargo build --release --bin rhwp
python tools/agent_onboarding/rhwp_doctor.py --json
```

닥터에게 빌드를 맡기지 않는다. 릴리스 빌드는 수 분이 걸리고,
온보딩 명령이 거기에 매달리면 에이전트 세션이 죽는다.

## 원인 표

| 원인 | 확인 | 처방 |
|---|---|---|
| 한 번도 빌드하지 않음 | `target/release/` 없음 | 위 빌드 명령 |
| 다른 워크트리의 target | `--repo-root` 가 이 트리인가 | 경로 교정 |
| PATH 에 다른 이름 | `Get-Command rhwp` | 별칭 또는 `--rhwp` |
| `--rhwp` 오타 | 파일 존재 | 절대 경로 |
| sparse checkout 이 target 을 지움 | `git sparse-checkout` | target 은 생성물, 다시 빌드 |
| cargo 미설치 | `cargo --version` | rustup. 네트워크 필요 시 오프라인 안내 |

## 리포트에서 볼 칸

- `binary.source` 가 `(미발견)` 또는 `--rhwp(미발견)`.
- `binaryInventory[]` 의 모든 `exists==false`.
- `buildCommand` 가 항상 `cargo build --release --bin rhwp`.
- `mcpJson` 은 그래도 방출된다 (`command: "rhwp"`). 붙이면 호스트가
  실행에 실패한다. 먼저 빌드한다.

## 하지 말 것

- `cargo install` 로 crates.io 의 다른 크레이트를 받지 않는다.
- Docker 로 네이티브 바이너리를 만들려 하지 않는다.
- gym 러너 바이너리를 rhwp 대신 쓰지 않는다.
- `target/debug` 를 숨기지 않는다. 닥터가 fallback 으로 쓸 수 있다.

## 점검 단계 01 — cargo 존재

`cargo --version`. 없으면 rustup.


## 점검 단계 02 — 워크스페이스

저장소 루트에 `Cargo.toml` 과 `src/main.rs`.


## 점검 단계 03 — bin 이름

`--bin rhwp`. 다른 패키지 bin 을 만들지 않는다.


## 점검 단계 04 — release 산출

빌드 후 파일이 생겼는지 `binaryInventory` 로 확인.


## 점검 단계 05 — 실행

`target/release/rhwp --version` 또는 `rhwp.exe`.


## 점검 단계 06 — 닥터 재실행

같은 `--repo-root`.


## 점검 단계 07 — PATH 등록 (선택)

호스트가 짧은 이름을 원하면 PATH 에 release 를 넣는다.


## 점검 단계 08 — RHWP_BIN (선택)

PATH 를 더럽히기 싫으면 절대 경로.


## 점검 단계 09 — MCP command

재실행 리포트의 `mcpJson` 을 붙여넣는다.


## 점검 단계 10 — 오프라인 빌드

캐시 없으면 네트워크 예외 문서로.


## 점검 단계 11 — 권한

Unix 에서 `+x`.


## 점검 단계 12 — 안티바이러스

Windows 가 exe 를 격리하면 예외 경로에 추가.


## 점검 단계 13 — 디스크

target/ 는 크다. 공간 부족이면 빌드가 중간에 죽는다.


## 점검 단계 14 — 툴체인

`rust-toolchain.toml` 이 지정한 채널.


## 점검 단계 15 — 재확인

exit 3 이 0/1 로 바뀌었는지 `--json` 의 `exitCode`.


## 성공

`binary.found==true` 이고 `checks` 의 `version` 이 PASS.
그다음 샘플 자가검증으로 넘어간다.
