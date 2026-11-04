# 글꼴 공급과 최소 환경

## 회귀 실행의 파일 시스템 격리

`bwrap`으로 `/usr/share/fonts`·`/usr/local/share/fonts`를 빈 tmpfs로 가리고 home은 작업 저장소·Cargo·Rustup만 bind한다. 외부 개인 글꼴 경로는 노출하지 않고 `RHWP_FONT_PATH`를 unset한다. Fontconfig는 저장소 `ttfs/opensource`만 탐색한다. Fontconfig에 실제 노출된 기본 목록은 아래 다섯 개다. 저장소에 이미 포함된 라이선스 확인 fixture·내장 fallback을 검사가 명시 공급하는 경로도 허용하며, 개인 글꼴 경로는 가려져 있다.

| 저장소 경로 | SHA-256 |
| --- | --- |
| `ttfs/opensource/NotoSansKR-ExtraLight.ttf` | `67b4003e2be99ea44a7c957e4b35acde1b5e6e82a54fe195c5598c3c617bc2e3` |
| `ttfs/opensource/NotoSansKR-Regular.ttf` | `6e06a7fe5d696ca719894a23f36bb2b1be8c816a5937cd4ad0f23ca67780dd74` |
| `ttfs/opensource/NotoSansSymbols2-Regular.ttf` | `876598db5f1e3bef2bc77c4d94c5b0d5d2c1620be4878a06f18fb5434e331688` |
| `ttfs/opensource/RHWPOldHangul-Regular.otf` | `0a32ff354d11c955f819d84d91e695927d41aca0fd74b4a286b6235345101af4` |
| `ttfs/opensource/RHWPPublicBatang-Regular.ttf` | `7e3aee5ee1d8f44cc37d35dbd07e5f66414fee5ce552ebda6b4347643d9d0b78` |

기존 저장소의 라이선스와 OFL 고지를 재사용한다. 개인 글꼴 설치 유무에 따라 회귀를 건너뛰지 않았다. 격리 명령은 [최종 검증](validation.md)에 기록한다.

## 독립 Print의 시각 대조

일반 공급 경로는 `/home/edward/mygithub/ttfs`다. form-01은 Native 카탈로그 256파일/64MiB 상한 전에 필요한 한양 바탕이 빠지지 않도록 `HBATANG.TTF` 하나를 명시 공급했다. SHA-256은 `35f84328500fc2c3ee0b148aa75de0ed384bf9eee8dee9148c01b0a11a27fe05`이며 base·후보·Native·WASM이 같은 프로그램을 사용한다.

form-01의 오래된 빈 2022 Print와 필요한 글꼴이 빠진 광범위 카탈로그의 초기 진단은 최종 판정에서 제외했다. 검증 기준은 `samples/hwpx/form-01-2024.pdf`다. 나머지 입력의 실제 카탈로그·공급 해시는 로컬 실행 결과에서 확인했고 글꼴 파일은 커밋하지 않았다. 회귀 최소 환경과 시각 대조용 공급을 구분한다.
