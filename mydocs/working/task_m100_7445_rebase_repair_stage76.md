# Stage76: CanvasKit 중복 import와 인쇄 종이 검사 복구

Issue: #7445. 기존 지원 대조군만 사용했고 새 HWP 보정 대상은 추가하지 않았다. 부모 HEAD는 `c33e8435365395ac72f04400fbb878543b34ba4d`, base는 `def947c2149df5bc812d801e6e79be19a403b4d1`이다.

## 원인과 최소 수정

`f45878c7caafb43ca1fd017ddd93484a0acc6ad2`의 상대 import와 이 브랜치 `f9ed38fd1cff4471bad5d6eaa8e3700416e01b5c`의 별칭 import가 같은 `parseCssFontFamilyList`를 중복 선언했다. 실제 Studio 동적 import가 HTTP 500을 반환하고 지원 CanvasKit 8건이 Canvas2D로 fallback했다. upstream 상대 import와 양쪽의 소비 구현은 유지하고 별칭 import 한 줄만 제거했다. backend guard나 준비 조건은 바꾸지 않았다.

Studio 단위 검사에서는 기존 크기 정규식 1건이 실패했다. 원 검사는 `e999f7a9fbf`, 후속 캐시 소유 보정은 `673fc6ef540`이며, 이 브랜치의 `0efd854b2644c2569b02b18f03dcf58ac674a50d`가 공통 Print 종이 크기 소비를 도입했다. 편집 pageInfo만 강제하는 정규식이 그 계약과 충돌했다. 기존 함수를 제품 메서드 실행으로 바꿔 같은 크기·더 큰 Print 종이·더 작은 Print 종이 및 소수 우·하단 보존을 검사한다. 편집 pageInfo가 원본 그대로 renderer로 전달되는 것과 비교 창 경계 검사도 유지했다. 단순한 변수명 교체나 검사 제거로 통과시키지 않았다.

## 실제 검증

- 부모 전체 nextest: **11,751 PASS / 0 FAIL / 45 skip**. Native Skia lib 3,859 PASS / 12 ignore 외 라이브러리 182 PASS, release lib root 3,802 PASS / 12 ignore 외 182 PASS다. Native/WASM 및 workspace/all-target/Skia Clippy, workspace/release build, manifest/unit tier, fmt, doc-tests도 완료했다. 원 게이트의 마지막 TypeScript 실패는 보존하고 이 단계에서 복구했다.
- 최초 중복 제거 뒤 TypeScript exit 0, Studio 2,408건 중 2,405 PASS / 1 FAIL / 2 skip였다. 종이 계약 검사 보정 뒤 같은 전체 **2,406 PASS / 0 FAIL / 2 skip**, 최종 17.069초다. 최종 TypeScript도 별도로 exit 0이다.
- 실행형 종이 계약 1 PASS. 과거 편집 종이 크기 소비를 되살린 진단에서는 Print 기대 451×601에 실제 151×301로 실패했다. 진단 변형을 과거 소스의 새 빌드로 오인하지 않는다.
- 기존 두 대조군 × compat/default × screen/print의 실제 CanvasKit **8건 모두 요청 backend, runtime 완료, fallback 0, exit 0**이다. 숨은 Canvas2D overlay 없이 직접 재생했다. Print raw는 low-slack 양 모드 99.97179%, 저장 줄 결측 양 모드 99.83083%다. 실제 제품 사각형과 독립 Print review/overlay를 직접 판독했다. Screen 고유 세 PNG도 내용 보존을 확인했으며 Print PDF와의 점수는 진단으로만 분리했다.
- 실제 초기화된 JS/WASM 응답이 frozen 산출물과 일치함을 확인했다. JS는 Vite URL 재작성과 inline source map의 원문을 대조했고, WASM은 원 바이트를 대조했다. 엔진 source는 기존 frozen producer `fb258e6a6d97e001b180c93e005139e73df6e85e`와 같다. Mac locked web/pkg/no-opt 로컬 대체 빌드이며 Docker 최적화로 보고하지 않는다.

## 자체 진단 정정과 잔여

처음 작성한 실행형 검사에서 설치된 TypeScript 7의 classic compiler API를 사용하려다 API 부재로 실패했다. 저장소의 기존 방식인 Node `stripTypeScriptTypes`로 정정했다. 음성 진단의 경로 및 failure recorder 누락도 정정하고, 최종 크기 assertion 실패와 정상 1 PASS만 근거로 사용했다. 초기 오류 로그는 ignored output에 남겼다.

Chrome이 이미 종료된 뒤에도 crash reporter·GoogleUpdater가 stdout/stderr를 물려받아 Puppeteer의 close가 대기했다. 프로세스·열린 파일·peer inode로 확인하고 종료된 Chrome의 소유 read pipe 두 개만 정리해 정상 exit 0을 확인했다. updater나 다른 사용자 프로세스를 종료하지 않았고 캡처를 중복 실행하지 않았다.

별도 실제 Canvas2D 판독에서는 로컬 글꼴 미감지 상태의 sample16 HWPX 45쪽이 raw 95.07810%여도 본문이 네모로 표시됐다. 제품의 로컬 글꼴 감지를 거친 같은 원본·코드에서 99.99941%와 실제 글리프 표시를 확인했다. 폰트 구현을 바꾼 결과가 아니며, 이 한 쪽을 전체 실제 소비 통과로 일반화하지 않는다. 정상 감지 조건의 Canvas2D 54건은 캡처와 exit 0을 확인했고, 독립 Print 비교와 직접 판독을 계속한다.

전쪽 base/head 52조건과 Native/fresh WASM 비교, 실제 출력의 남은 판독, 기존 C354 devel 대비 4.28268%p 하락 및 C356 진단 미달의 판정이 남아 있다. 이 단계는 로컬 보정 체크포인트이며 **PR 준비 완료가 아니다**. Rust 엔진·WASM·원본·PDF·폰트·정책의 변경이나 원격 조치는 없다.

[제출용 대표 이미지와 출처](../pr/assets/issue7445/completed-20261009/rebase-def947-20261023/canvaskit-import-preservation/README.md). 로그와 임시 산출물은 ignored `output/pr-review/7445-completed-20261009/rebase-8155-20261022/canvaskit-import-repair/`에만 보존했다.
