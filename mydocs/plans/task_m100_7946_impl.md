# Task #7946 — Studio PDF 직접 열기 구현계획

- Issue: [#7946](https://github.com/edwardkim/rhwp/issues/7946)
- 기준 SHA: `4662c069e557e928ed7968d061121fde90b1f027`
- 상태: 구현 전, 계획 승인 대기
- 상위 계획: [수행계획](task_m100_7946.md)

## 1. 제품 연결 계약

PDF 지원 빌드에서는 파일 열기·fallback input·드롭으로 PDF 바이트를 기존 제품 loader에
전달한다. `HwpDocument` → 공통 IR → 기존 Canvas 렌더 경로를 사용하며 PDF→HWPX
임시 변환·재파싱이나 POC 의존을 추가하지 않는다.

빌드 capability는 실제 Rust feature와 일치하는 작은 WASM 조회 API로 제공한다.
기본 비활성인데 메서드는 존재하는 현재 계약을 고려해 `typeof importPdf`만으로 판정하지 않는다.
이전 WASM에 조회 API가 없으면 PDF 비활성으로 처리하고 HWP/HWPX/HML을 유지한다.
조회 API 이름과 배선은 구현 단계에서 기존 capability 명명 규약에 맞춘다.

현재 Studio는 문서 API를 동기 호출한다. 이번에는 그 문서 수명주기를 재사용한다.
진행 표시를 준비하되 동기 실행 도중 취소·deadline을 보장한다고 표시하지 않는다.
전체 편집기를 Worker RPC로 바꾸는 작업은 별도 범위이며 기존 공용 Worker 계약을 변경하지 않는다.

## 2. 변경 위치와 실제 소비 경로

| 위치 | 변경 | 확인할 소비 지점 |
| --- | --- | --- |
| `src/wasm_api.rs` 및 바인딩 | feature-on/off capability 조회 | 초기화 뒤 실제 WASM 응답, 이전 bundle fallback |
| `rhwp-studio/src/core/wasm-bridge.ts` | capability·PDF 진단 접근, 기존 원자적 load 재사용 | 후보 생성·convertToEditable 이후 commit/실패 복구 |
| `rhwp-studio/src/command/file-system-access.ts`, `file-open-picker.ts` | capability별 파일 형식·picker options | File System Access와 fallback input이 같은 목록 소비 |
| `rhwp-studio/index.html`, `src/main.ts` | input accept/안내·drop 판별·정식 load 연결 | `loadFile` → `loadBytes` → `initializeDocument` → Canvas |
| 형식·최근 문서·저장 target 관련 타입/소비자 | PDF 가져오기 출처와 HWP/HWPX 출력 형식 구분 | 원본 PDF handle에 저장하지 않으며 기존 dirty/최근 문서 계약 보존 |
| 기존 Studio/WASM 검사와 필요한 `tests/cases/` 원본 | capability·입력·오류·문서 교체 회귀 | 실제 호출 경로의 수정 전 FAIL/수정 후 PASS와 정상 대조군 |
| `mydocs/manual/dev_environment_guide.md`, Studio 안내 | PDF-on Docker 빌드·열기·기능 제한 | 실제 실행한 명령·산출물과 일치 |

기존 PDF parser/lowering과 pagination은 수정 대상으로 삼지 않는다. 직접 열기에서 기존
`convertToEditable`나 폰트 준비가 저장본 경로와 다른 결과를 만들면 생산 값부터 실제 Canvas
소비까지 추적해 원인을 기록한다. 근거 없이 저장 LineSeg를 지우거나 좌표를 보정하지 않는다.

PDF File System Access handle은 입력/최근 문서 정보로 취급하고 출력 handle로 재사용하지 않는다.
첫 저장은 기존 HWP/HWPX 출력 선택을 사용한다. 파일명만 바꾸거나 PDF를 HWP 출처로 가장하지 않는다.

## 3. 검증 순서

1. 작은 계약 검사: on/off·이전 WASM, picker/fallback/drop, 취소·잘못된 확장자,
   손상/한도 오류 뒤 현재 문서·다음 정상 입력, 원본 PDF 덮어쓰기 방지.
2. Docker `wasm` 서비스와 locked wrapper로 루트 web package를 PDF-on으로 생성한다.
   기존 공용 Docker target/cache를 재사용하고 pkg/public JS·WASM SHA를 확인한다.
   별도 검증 디렉터리에 default-off package도 확보해 실제 비활성 계약을 확인한다.
   Native 검사는 기본 작업공간 `/home/edward/mygithub/rhwp/target/pr-review`를 재사용한다.
3. 실제 CDP Chrome의 정식 UI로 원본 PDF를 직접 연다. 12쪽·스크롤·쪽 이동, 3쪽 crop과
   11쪽 그림 9/10·캡션 및 오류 없음, 실제 로드 WASM SHA를 기록한다.
   HWP/HWPX/HML 기존 공개 대조군의 열기·렌더·문서 교체도 확인한다.
4. 원본 PDF와 새 직접 입력 결과의 전쪽 Native/fresh WASM TSV를 먼저 산출한다.
   각 backend 최저 90% 이상·미달/누락 0·12쪽 일치 및 대표 영역 직접 판독을 수행한다.
   이전 저장 HWP/HWPX의 비교를 새 PDF 직접 입력 검증으로 대신하지 않는다.
   B10-1 한컴 재조판 미달은 분리해 유지한다.
5. [변경 범위별 검증](../manual/pr_review/local_validation.md#43-변경-범위별-기본-검증)에
   맞춰 Studio 타입/검사·바인딩 검사를 실행한다. Rust 변경에는 fmt·Native/WASM/workspace
   Clippy·workspace build와 base SHA 고정 정책 검사를 순차 적용한다.
   긴 전체 회귀·PR CI는 정본의 승인 절차를 따르고 이미 충분한 증거를 불필요하게 반복하지 않는다.

검출을 주장하는 새 검사는 수정 전 실제 원인으로 실패하는지 확인한다. 배치 회귀는 소유·
순서·포함·누락/중복 관계를 기본으로 하며 신규 fixture/golden은 시각 선행 조건을 따른다.
빌드/환경 실패와 제품 결함, 미실행 경로를 구분해 기록한다.

## 4. 증적·단계 경계

로그·중간 JSON·TSV·브라우저 캡처는 ignored `output/pr-review/issue7946/`에 둔다.
원본·기준 PDF·사용 글꼴·code SHA·실제 WASM 해시와 실행 명령을 단계 보고서에서 연결한다.
50 MiB 이상 PDF는 로컬에만 보존하며 기존 공개 12쪽 PDF를 우선 사용한다.

계획 승인 → 구현·집중 검사 → 단계 보고·커밋 → 실제 WASM·시각 검증 → 최종 보고·
승인된 원격 반영/PR 순서다. 지금은 계획 문서만 작성하며 제품 source·정식 UI는 변경하지 않는다.
