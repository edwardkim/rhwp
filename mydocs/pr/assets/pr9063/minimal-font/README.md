# PR #9063 최소 글꼴 실행 환경

`run.sh`는 Linux bwrap으로 시스템·사용자 설치 글꼴과 웹폰트 cache를 숨기고 저장소 소유 공개 글꼴 5종만 ambient 공급한다. 실제 목록은 [노출 목록](exposed-fonts.txt)과 같다. fixture는 기존 라이선스가 확인된 `ttfs/redistributable`을 명시 공급한다. 설치 여부에 따른 조용한 skip은 추가하지 않았다.

Node launcher에는 기존 NVM Node 24.15.0 runtime 디렉터리만 read-only로 연결했다. HOME의 사용자 글꼴을 노출하지 않는다. Cargo는 기본 작업공간의 절대 `target/pr-review` 하나를 사용하며 동시에 빌드하지 않는다.

WSL의 `/etc/resolv.conf`는 `/mnt/wsl/resolv.conf`를 가리키므로 그 대상 파일도 read-only로 연결했다. 이 연결이 없는 첫 Skia Clippy는 의존 도구 다운로드의 DNS 조회에서 중단됐다. 수정 후 같은 wrapper의 실제 DNS 조회를 확인하고 해당 검사만 재실행했다. 글꼴 격리·제품 소스·검사 기준은 바꾸지 않았다.

이번 실행은 기본 작업공간의 `output/pr-review/pr9063-correction/minimal-font/fonts.conf`에 이 디렉터리의 설정을 복사한 뒤 수행했다. 집중·전체 release-test·Native 공식 3종·lint의 실제 source·명령·exit·개수는 상위 [실행 원장](../evidence.md)을 따른다. 원시 로그는 최종 PR에 포함하지 않는다.
