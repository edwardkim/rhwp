---
kind: guide
status: active
canonical: mydocs/manual/installer_channels_guide.md
last_verified: 2026-08-09
---

# 설치 채널 가이드 2부 — binstall·deb·rpm·MSI·스크립트·AUR·server.json (#4375)

1부(scoop·homebrew·winget — `channel_manifests_guide.md`,
[PR #4339](https://github.com/edwardkim/rhwp/pull/4339) 리뷰 중이라 상대 링크는
착지 후 잇는다)의 후속 일괄이다. 채널마다 **활성 조건**과 **검증 상태**를 정직하게 갈라
적는다 — "커밋됨"과 "동작 검증됨"은 다른 등급이다.

## 채널 매트릭스 (활성 조건·검증 상태)

| 채널 | 사용자 명령 | 활성 조건 | 검증 상태 |
|---|---|---|---|
| cargo-binstall | `cargo binstall rhwp` | **머지 즉시**(기존 릴리스 자산 URL 매핑) | 매핑이 실자산 이름과 일치함을 수동 대조 — binstall 실행은 미검증(로컬 미설치) |
| deb | 릴리스에서 `.deb` 받아 `dpkg -i` | 다음 `v*` 태그(시크릿 불요) | 워크플로 잡 — CI 실행으로 검증 예정(`workflow_dispatch` 권장) |
| rpm | `.rpm` → `rpm -i` | 동상 | 동상 |
| **MSI** | 릴리스에서 `.msi` 실행 | 동상 | wxs 는 WiX v4 스키마로 작성 — 로컬 WiX 부재로 미빌드, dispatch 1회로 검증 권장 |
| install.sh | `curl … install.sh \| bash` | 머지 즉시 | **로컬 미검증**(이 PC curl TLS 차단) — 구조는 ps1 과 동형 |
| install.ps1 | `irm … install.ps1 \| iex` | 머지 즉시 | **실전 E2E 검증됨**(v0.8.2 실다운로드→SHA-256 일치→실행, PR 본문 실측) |
| AUR (rhwp-bin) | `yay -S rhwp-bin` | AUR 계정 제출(메인테이너/위임) | PKGBUILD 실해시 반영 — 제출 전 `makepkg` 검증 필요(리눅스) |
| crates.io | `cargo install rhwp` | `CRATES_IO_TOKEN` + **패키징 include 정비(선행)** | 이중 게이트(시크릿→`publish --dry-run`) — dry-run 실패 시 게시하지 않고 경고만 |
| server.json | MCP 공식 레지스트리 제출 | 참조 패키지 게시 후(#4337 pypi/npm·#4372 oci) → #4343 실행 | 스키마 2025-12-11 실검증 후 작성 — 제출은 게시 뒤 |

## crates.io — 왜 바로 안 켜는가 (정직 기록)

`cargo package --list` 실측 13,975 파일. 본체는 `include_str!`/`include_bytes!`
50건(레시피·llms.txt·지식 지도 등 `mydocs/` 하위 포함)을 임베드하므로,
crates 패키징은 **include 목록 수술**(소스+임베드 파일 전수, 10MB 상한 확인)이
선행이다. 그때까지 잡은 dry-run 게이트가 게시를 막고 사유를 로그로 남긴다 —
체크박스를 위해 깨진 패키지를 올리지 않는다.

## 버전·해시 갱신

scoop·brew·winget 은 `tools/update_channel_manifests.py`(#4338)가 담당한다.
AUR PKGBUILD 의 pkgver·sha256 갱신을 같은 스크립트로 확장하는 것은 후속(이슈로).

## MSI 설계 메모

- `UpgradeCode` 고정이 판올림 교체의 열쇠 — 변경 금지.
- PATH 는 시스템 환경변수에 `Part="last"` 로 덧붙인다(제거 시 자동 원복,
  `Permanent="no"`).
- 서명(코드사이닝)은 version_policy.md §8 의 서명 도구 결정과 합류 — 미서명
  MSI 는 SmartScreen 경고가 뜬다는 사실을 릴리스 노트에 명시할 것.
