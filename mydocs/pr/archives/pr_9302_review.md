# PR #9302 Studio 편집·저장 개선 5건 통합 검토

`postmelee`의 PR 5건에서 커밋 9개를 `d7b5286cbf` 기준으로 체리픽했다. 각 커밋에 원 작성자와 원본 SHA를 보존했다. 코드 head `5d89da436be52d5e2696849653037bff79dd8afa`의 GitHub Full CI와 Build & Test가 통과했다. 문서 기록을 포함한 head의 CI와 병합 가능 상태를 최종 확인한 뒤 일반 병합하는 것으로 판정했다. [통합 PR](https://github.com/edwardkim/rhwp/pull/9302) · [코드 CI](https://github.com/edwardkim/rhwp/actions/runs/44939210312) · [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/44939210312/job/136960674256)

## 원 PR별 검토

| 원 PR | 원 커밋 → 통합 커밋 (적용 순서) | 확인한 동작과 판정 |
| --- | --- | --- |
| [#9255](https://github.com/edwardkim/rhwp/pull/9255) · #5987 | `336c7c4451 → 5d1f4c4ee4`, `ea5bfa7efc → ef6402faf5` | 암호 내보내기와 호스트의 영구 저장 확인을 export ID, 문서 세대, 편집 순번, 형식으로 연결한다. Chrome에서 오래된 확인 거절, HWP·HWPX 암호 저장·재열기, HWP3 변환 및 일반 임베드 전송을 확인했다. [원 PR CI](https://github.com/edwardkim/rhwp/actions/runs/44762701899/job/136396424478) |
| [#9256](https://github.com/edwardkim/rhwp/pull/9256) · #9011 | `97202f4df6 → 4575dd8711`, `8b303317d6 → dd605758b6`, `1dc2b31db5 → 6597a9622f`, `31cc3560b2 → 7a2edde980` | 메모 생성·수정·삭제를 기존 snapshot/history에 연결한다. 본문·두 표 셀의 내용과 대상은 HWP와 HWPX 각각 두 차례 재열기 후에도 일치했다. 지원하지 않는 중첩 대상은 거절했다. #9259~#9262에서 단어 범위·이동·여백 표시·댓글을 다룬다. [원 PR CI](https://github.com/edwardkim/rhwp/actions/runs/44762076445/job/136393653151) |
| [#9257](https://github.com/edwardkim/rhwp/pull/9257) · #9174 | `06169a5653 → 3360f5fedf` | 동일 문서·공급자 세대에서 이미 공급한 글꼴 등록을 재사용한다. 통합본에서 8.43 MB 서체로 셀에 20회 입력했을 때 글리프 범위와 윤곽 재등록은 각각 0회·0바이트였고 두 번째 프레임까지의 중앙값은 33.1 ms였다. Canvas2D·CanvasKit의 공급자 교체와 해제도 통과했다. [원 PR CI](https://github.com/edwardkim/rhwp/actions/runs/44757185237/job/136378570717) |
| [#9266](https://github.com/edwardkim/rhwp/pull/9266) · #6821 | `7999e65461 → 253c6ae334` | 뷰포트의 스크롤 프레임에서 두 축의 눈금자를 다시 그린다. 원 PR의 3 CSS px 초과 세로 지연은 101/129에서 0/129, 가로 지연 프레임 개선은 약 8~13%였다. 통합본에서 첫 실행 안내의 ‘대체 글꼴로 보기’를 선택한 뒤 A4 눈금자가 795 px에서 1,123 px로 바뀌는 것을 확인했다. **부분 수용**하며 가로 목표 25%와 실물 트랙패드 관성 확인은 #6821에서 계속 추적한다. [원 PR CI](https://github.com/edwardkim/rhwp/actions/runs/44768024068/job/136415322327) |
| [#9267](https://github.com/edwardkim/rhwp/pull/9267) · #8197 | `ae85b0f7de → 5d89da436b` | 한 단의 일반 본문에서 기존 삽입·분할 명령을 한 번의 배치로 처리한다. 원 PR의 400문단 동기 처리는 1,825.9 ms에서 28.9 ms로 줄었다. 통합본에서 문단 순서, 캐럿·편집 알림 1회, HWP/HWPX 재열기, 실행 취소·다시 실행, 배치 실패 복구를 확인했다. [원 PR CI](https://github.com/edwardkim/rhwp/actions/runs/44773200839/job/136433188649) |

## 통합 검증과 충돌 처리

- `wasm-bridge.ts` 첫 줄에 #9255와 #9256이 추가한 HWP3 식별·메모 번역 import를 모두 보존했다. 이 두 원 PR을 이미 체리픽한 통합본에 **다시** 병합하는 모의 검사만 같은 곳에서 충돌했다. 작업지시자는 두 기능과 실제 devel 병합 경로를 검증하는 조건으로 해당 중복 검사에 예외를 승인했다. 원 PR 세 건, 별도 작업 #9300, 제출 시점의 devel `5665e53829`와 통합본의 모의 병합은 통과했다.
- Chrome과 Studio 배포 산출물에서 읽은 최적화 WASM은 SHA-256 `fe79de6dc6907e485288eb46a35ccd9d043a0ea330c4087225ef72cfd8633d98`로 일치했다. SDK 38건·바인딩/임베드 3건·E2E 목록 199/199·TypeScript·배포 빌드 통과. Studio 전체는 2,832건 통과, 별도 Node용 WASM이 필요한 기존 서식 검사 2건 생략이다.
- Rust fmt, Native·WASM32·workspace Clippy, workspace 빌드와 suite manifest가 통과했다. 기본 기능 전체 회귀에서 13,864건 중 13,855건이 첫 실행에 통과했다. 합성 글꼴 2건은 테스트가 생성한 서체 경로를 명시했고, CLI 7건은 같은 소스의 기본 기능 바이너리를 고정하여 재실행했다. 9건 모두 통과했다. 스위트 설정상 별도 제외는 45건이다.
- Native Skia 라이브러리 3,852건 통과·12건 제외, 그림 2건과 직접 PDF 36건 통과. 시험 중 생성되는 합성 서체를 명시했고 격리 프로세스에 노출된 설치 글꼴 237개는 모두 macOS 시스템 경로였다. 이번 Rust 변경은 WASM 대상 읽기 전용 글꼴 선언 조회이며 집중 검증은 실제 Studio 글꼴 소비 경로에서 수행했다.

병합 뒤 원 PR마다 통합 커밋과 검증 결과를 연결하고, #5987·#9011·#9174·#8197의 완료 조건을 근거와 함께 기록한다. #6821은 열린 상태로 유지한다.
