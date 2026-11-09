# Studio 그림 이동 검사의 Vite 캐시 격리와 R41 검증

## 실제 실패 원인

첫 전체 Node 실행은 2753건 중 2752 PASS / 1 FAIL이었다. 실패는 `picture-relocation-owner-8557.test.ts`의 음성 대조에서 공유 `node_modules/.vite/deps`를 정리하던 `ENOTEMPTY`다. 정상 입력·기존 잘못된 소유 경로·잘못된 미리보기 좌표의 동작 기대값을 바꿀 근거로 사용하지 않았다.

테스트의 `createServer`가 앱 설정을 읽으면서 SSR 동작 검사와 관계없는 브라우저용 SDK 사전 묶음을 시작했다. 첫 캐시 분리 시도에서도 assertion은 통과했지만 서버 종료 뒤의 지연 쓰기가 임시 경로를 다시 만들었다. 같은 source의 단독 실행 3건은 통과했으며, 이 결과만으로 공유 캐시 문제를 해결했다고 쓰지 않았다.

## 보정 범위

- 각 `withHandler` 호출에 ignored `output/studio-test-cache` 아래 고유한 캐시 경로를 만든다.
- 테스트의 resolved config에서 클라이언트 의존성 자동 탐색·사전 묶음을 비활성화한다. 실제 SSR 모듈·앱 alias·동작 함수와 두 음성 대조는 그대로 실행한다.
- 서버 종료 뒤 `finally`에서 이 호출이 만든 경로만 정리한다. 앱·사용자 개발 서버의 캐시나 서버는 변경하지 않는다.
- `exercise`와 세 test 정의·assertion 본문은 수정 전후 바이트 단위로 같다.
- 원본 전체 source SHA-256: `4b6cfeb880a155ff817cf237a8a48fe32f15e1c3e5dce9c6ade7c0090fae93da`
- 보정 전체 source SHA-256: `cdd6a13fd972a3515f71380db3ec2492a5db4b70378ebaaf82a8ebf12c1c4ab1`
- 변경하지 않은 assertion·test 정의 본문 SHA-256: `2bbac9057a799dda0e2848dfabe2a48a0645166801a9022a1f12cd71c426a575`

## 실제 검증

동시에 실행한 두 원본 검사 프로세스는 각각 3 PASS로 총 6 PASS였다. 둘 다 종료한 뒤 새 임시 캐시 잔여는 0개였다. 초기 진단에서 남긴 정확한 8개 경로는 실제 사용 참조가 없음을 확인한 뒤 정리했다. 설치 글꼴·한컴 출력·실물 페이지의 고정 좌표에 의존하는 검사가 아니다.

최종 병렬 Node 전체 실행도 **2753 PASS / 0 FAIL**이다. 이전 실패 로그와 단독·동시 재실행 로그를 보존했고, 최종 통과로 이전 명령의 종료 코드를 바꾸지 않았다.

## 같은 실제 최적화 WASM의 Studio 소비

- 제품·WASM 검증 source: `851e4074514935e4c6a7377f71a97f0cca454191`
- WASM SHA-256: `8a0cbdfe787c97273ec47922daa7d12d10b158bc471e6cfd29559bb96a2c8723`
- WASM glue SHA-256: `0099b8a6e00f750e1dbddb3a89bf42874f24effcf8bf4dc5eb328d69e691606b`
- 아래 12개 실제 단계가 모두 exit 0이다. 검증 로그는 ignored output에 보존하며 이 문서에는 커밋하지 않는다.

| 단계 | 실제 결과 |
| --- | --- |
| `cff-ttf-browser` | PASS, exit 0 |
| `canvas-font-browser` | PASS, exit 0 |
| `studio-node` | PASS, exit 0 |
| `production-build` | PASS, exit 0 |
| `frontend-public-api` | PASS, exit 0 |
| `canvaskit-browser` | PASS, exit 0 |
| `rebase-picture-public-api` | PASS, exit 0 |
| `rebase-picture-e2e` | PASS, exit 0 |
| `fontname-api` | PASS, exit 0 |
| `fontname-slot-limit` | PASS, exit 0 |
| `fontname-saved-references` | PASS, exit 0 |
| `marker-source-contract` | PASS, exit 0 |

실제 CanvasKit 수식·그림 공개 API·드래그 E2E·글꼴 이름 등록·슬롯 한도·HWP/HWPX 재열기 참조와 원 PUA 마커 소유도 같은 fresh package에서 확인했다. 이 Studio 완료 결과를 Native/fresh WASM 전쪽·기준 대비 하락 비교·최종 Rust/lint의 완료로 대신하지 않는다.

## 의도한 예외 로그와 실제 실패의 구분

AI 명령 복구·잘못된 셀 편집 대상·웹 글꼴 실패/시간 초과 검사는 모의 객체로 예외를 만들어 거절·복구 상태를 확인한다. 해당 검사는 최종 전체 실행에서 모두 PASS다. CDN 실패 로그는 가짜 `fetch`·`FontFace`·가상 타이머의 출력이며 실제 CDN 장애 증거가 아니다. 공유 Vite 캐시의 `ENOTEMPTY`는 이 예상된 예외와 달리 첫 실행의 실제 실패였으며 위 보정과 재실행으로 해결했다.
