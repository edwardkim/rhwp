# #8174 v2-S36 — Native PDF 작업자 접근 제한

Issue: #8174. 2026-10-26, T05/B07의 Native 절편이다.
사용자 「확인했습니다. 계속 진행해주세요.」와 기존 T01–T09 순차 진행 승인을 따른다.
기준 로컬 커밋은 `c1982b6d24`이며 원격 변경이나 rebase는 수행하지 않았다.

## 변경 경계

`import-pdf`와 게시 패키지의 Rust host 예제가 같은
[`confine_pdf_worker`](../../src/resource_supervisor/pdf_confinement.rs)를 소비한다. Linux
x86_64/aarch64에서 새 단일 스레드 작업자는 파이프인 표준 입출력만 유지하고 기존 상속 fd를
닫는다. 입력 PDF를 읽기 전용 핸들로 연 뒤 capability를 비우고 부모 종료 신호를 복원하며,
Landlock ABI 3 이상의 읽기 규칙과 seccomp 허용 목록을 적용한다. 입력 bytes를 해석하거나
계산 스레드를 만들기 전에 제한을 설치한다. 글꼴 자원 외 파일 내용 읽기·쓰기, 소켓, 외부
실행과 별도 프로세스 생성은 허용하지 않는다. 계산 스레드는 제한을 상속한다.

처음에는 제한 후 입력 경로를 다시 열었으나 `/mnt/e/tmp/21a.pdf`에서 EACCES를 재현했다.
이 실패를 보존하고, 허용한 입력의 읽기 전용 핸들을 반환해 사용하도록 보정했다. WSL
마운트의 입력을 처리하기 위해 파일 제한을 제거하거나 해당 마운트 전체를 허용하지 않았다.
지원 커널 기능이나 신뢰하는 font 설정이 잘못되면 `PDF_PROCESS_SANDBOX_UNAVAILABLE`로
실패한다. 다른 OS/아키텍처의 실행 한도와 실제 접근 제한을 혼동하지 않는다.

부모는 성공한 작업자의 제한 보고를 확인한 뒤 `filesystemNetworkSandbox=true`를 표시한다.
일반 `resource_supervisor::run`은 이 값을 자동으로 올리지 않는다. 기존 AS/CPU/wall/출력
감독과 함께 사용하지만 프로세스 전체 RSS·부모·동시 요청·커널 자원의 완전한 제한을
보증하지 않는다. 파일 메타데이터 은닉이나 컨테이너 전체 격리도 주장하지 않는다.
`info`·`export-svg`·`convert`의 공통 로더와 직접 라이브러리 호출은 별도 경계로 유지한다.

게시 패키지에 포함할 [`pdf_isolation_host`](../../packaging/crates-io/examples/pdf_isolation_host.rs)는
요청마다 새 자식을 만들고 Session 가져오기·첫 저장을 수행한다. 부모의 합성 비공개 파일과
네트워크가 정상 접근되는 대조군을 먼저 확인한다. 자식의 차단 검사는 연결 거절 같은
우연한 네트워크 실패가 아니라 EPERM을 확인하며 실제 입력 읽기와 계산 스레드도 검사한다.

## 실행 결과

- 기능용 Native: `release-test`, root opt-level 1·codegen-units 64. 최종 성능 근거가 아니다.
  마지막 제품 빌드는 `native-build-r9.log`이며 root library는 r8과 같다. 변경 source 전체와
  바이너리 hash는 아래 `native-producer.json`·`frozen-source.json`으로 고정했다.
- 실제 host 예제: 일반 Linux 사용자·사용자 네임스페이스 root·WSL 마운트 입력에서 모두 통과.
  허용 입력/계산 스레드, 상속 fd 닫기, 무관한 파일 읽기/쓰기·TCP/UDP·exec/fork·chmod 차단,
  capability 0, 부모 종료 신호 유지, 외부 prlimit 차단을 확인했다. 잘못된 표준 입출력도 거절했다.
- 공개 API·실제 CLI 계약: 신규 confinement **2 PASS**, 기존 자원 정책 **6 PASS**.
  신규 검사는 `tests/cases/`에만 추가했다. 기존 검사의 Linux 접근 제한 기대값을 실제 적용에
  맞췄으며 출력 보존·정상 후속 요청 조건은 유지했다.
- 기존 검사 실행 중 S34–S35에서도 남아 있던 출력 한도 오류 코드 불일치를 발견했다.
  과거 로그의 `SERIALIZE_RESOURCE_LIMIT: output byte limit`와 진단 JSON 한도 오류를 보존했다.
  bounded producer가 transport 이전에 거절해도 약속한 `PDF_PROCESS_OUTPUT_LIMIT`를
  반환하게 보정했다. 다른 decoded 한도 실패는 기존 오류와 상세 원인을 유지한다.
- S35 대비 **34개 저장·98쪽 재열기**, 상세 진단 3건, 선택 7·9쪽의 양쪽 형식 **4개 왕복**이
  같다. 저장 파일 34개와 상세 보고용 3개도 bytes가 같다. 비교에서 자원 사용량만 제외했으며
  진단 의미·정책·재열기 렌더 트리를 완화하지 않았다.
- HWP/HWPX의 출력·진단·잔여 decoded 거절과 기존 출력/빈 stdout 보존·새 정상 요청 복구
  **6건**을 통과했다.
- 최신 Native CFF **100%**, Type3 11쪽 **99.97900%**. 현재 제품의 compare·overlay·review를
  산출해 문자·간격·표 외곽과 그림/캡션 소속을 직접 대조했다. 새 렌더링 회귀를 추가하거나
  기존 baseline을 변경하지 않았다.
- Native library/CLI/host 예제 Clippy(`-D warnings`), feature-off library·WASM library 컴파일을
  통과했다. 컴파일 검사는 최신 최적화 WASM 제품 실행을 뜻하지 않는다.
- 자동 수집 스크립트 **23 PASS**, S35 대비 unit-tier 검사 통과. 브랜치 전체의 이전 source
  unit 증가 차단은 해결된 것으로 표시하지 않는다.

원시 명령·로그·source/binary hash·TSV·PNG는 ignored
`output/pr-review/issue8174/v2-s36-20261026/`에 보존한다. 직접 rustc로 연결한 공개 integration
원본의 실행은 집중 검사이며, 최종 자동 suite 실행은 T08에서 별도로 수행한다. 초기 컴파일의
libc 상수 오류와 직접 연결 검사에서의 serde artifact 선택 오류도 보존했고, 실제 Cargo
fingerprint에 연결된 dependency artifact로 바로잡았다.

## 남은 범위

T04의 Core/나머지 serializer 할당과 T05의 WASM/Studio 복사·직렬화·동기 재열기 경계는
남아 있다. 이번 Linux x86_64/WSL2 실행을 aarch64·다른 OS 실행 완료나 비신뢰 업로드의
전체 지원 완료로 확대하지 않는다. host 예제의 실제 CI 실행 연결, 이전 절편 source unit의
공개 API 검사 이동, 최종 fresh WASM·Studio·전쪽 시각·성능·포장·소비도 남는다.
다음 절편을 다시 승인받기 위해 중단하지 않고 기존 승인 범위에서 계속한다.
