---
kind: guide
status: active
canonical: mydocs/manual/channel_manifests_guide.md
last_verified: 2026-08-09
---

# 설치 채널 매니페스트 가이드 — scoop·homebrew·winget (R38, #4338)

`contrib/packaging/` 의 매니페스트 3종과 갱신 스크립트의 운영 문서다. 목표는
R38("설치 1줄")의 채널 축 — 릴리스 바이너리는 이미 존재하므로(#4327 §4), 남은
것은 각 설치 관리자에 그 바이너리를 잇는 매니페스트다.

**정직 고지**: 채널별 심사·CI 의 세부 요건은 **제출 시점에 해당 채널 문서로
검증**해야 한다(§5 미검증 목록). 이 저장소의 매니페스트는 제출 원본이며, 여기서
검증하는 것은 자산 URL·해시의 실물 일치와 갱신 멱등성까지다.

## 1. scoop (Windows) — 머지 즉시 사용 가능

scoop 은 버킷 등재 없이도 매니페스트 raw URL 로 바로 설치할 수 있다:

```
scoop install https://raw.githubusercontent.com/edwardkim/rhwp/devel/contrib/packaging/scoop/rhwp.json
```

- `extract_dir: rhwp` — 아카이브가 `rhwp/` 폴더를 담는 구조(release-binary.yml)
  를 반영한다.
- `autoupdate` 절이 있어 공식 버킷(main/extras) 제출 시 자동 갱신 대상이 된다.
  버킷 제출은 후속(§5).

## 2. winget (Windows) — microsoft/winget-pkgs 제출 원본

`contrib/packaging/winget/` 의 3종(version·installer·defaultLocale)이 제출
단위다. portable(zip) 형식으로 `rhwp\rhwp.exe` 를 `rhwp` 별칭으로 노출한다.

제출 절차(요지): winget-pkgs 포크 →
`manifests/e/Edwardkim/rhwp/<버전>/` 에 3파일 복사 → PR. `wingetcreate` 또는
저장소 검증 도구(`winget validate`)로 사전 검증을 권장한다. 스키마 버전
(1.6.0)·필드 요건은 제출 시점에 winget-pkgs 문서 기준으로 재확인한다.

## 3. homebrew (macOS·linux) — 탭 위치는 메인테이너 결정

`contrib/packaging/homebrew/rhwp.rb` 는 3자산(mac arm/intel·linux)을 덮는다.
brew 는 "탭(= Formula/ 를 담은 git 저장소)"이 필요하므로 두 경로 중 하나를
메인테이너가 고른다:

1. **전용 `homebrew-rhwp` 저장소 신설(권장)** — `brew tap edwardkim/rhwp` 가
   관례대로 동작. 이 파일을 그 저장소 `Formula/rhwp.rb` 로 복사.
2. 본 저장소 루트에 `Formula/` 추가 — 추가 저장소가 없지만
   `brew tap edwardkim/rhwp https://github.com/edwardkim/rhwp` 로 전체 저장소를
   클론하게 된다(무거움).

homebrew-core 제출은 인지도 요건이 있어 후속으로 둔다(§5).

## 4. 릴리스 시 갱신 — 1커맨드

```
gh release download vX.Y.Z -p SHA256SUMS.txt
python tools/update_channel_manifests.py X.Y.Z SHA256SUMS.txt
```

`--check` 는 변경 없이 멱등 검증만 한다(현재 커밋본이 해당 버전 기준 최신인지).
릴리스 워크플로에 이 갱신을 자동 커밋으로 얹을지는 후속 판단(자동 커밋은 보호
브랜치 정책과 얽힌다).

## 5. 미검증·후속 목록 (정직)

- winget-pkgs 심사 세부(스키마 최신판·모니커 충돌·설치 검증 VM) — 제출 시 확인.
- scoop 공식 버킷(main/extras) 등재 기준 — 제출 시 확인.
- homebrew-core 인지도 요건(스타·다운로드) — 탭 운영 후 재평가.
- 서명: 현재 무결성은 SHA256SUMS 기반. version_policy.md §8 의 서명 도구 결정
  후 매니페스트·검증 절차에 반영.
- linux 추가 채널(AUR·nix 등)은 수요 신호 확인 후.
