# 작업 2473 단계 2 - Safari 빌드 실패 전파

## 범위

- 출력량을 `tail`로 줄이는 Safari macOS `xcodebuild` 호출에서도 종료 상태를 보존한다.

## 발견

- `rhwp-safari/build.sh`는 `set -e`를 사용했지만, 서명 인증서가 없어 로컬 빌드가 실패할 때
  `xcodebuild ... | tail -3`가 `tail`의 성공 상태를 반환했다.
- `CODE_SIGNING_ALLOWED=NO`를 지정한 직접 `xcodebuild`는 Safari 앱을 성공적으로 컴파일했다.
  즉 source package는 빌드 가능하지만 wrapper script가 서명 빌드 실패를 거짓 성공으로 보고했다.

## 검증 계획

1. shell script 문법을 확인한다.
2. `pipefail`이 실패한 producer로 인해 pipeline을 실패시키는지 확인한다.
3. 로컬 서명이 불가능한 환경에서 Safari build wrapper를 실행해 nonzero 상태가 전파되는지
   검증한다.
