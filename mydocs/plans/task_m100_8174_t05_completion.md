# #8174 T05 완료 범위와 종료 조건

2026-10-28. 기준은 T04 완료 커밋 `f5098288552a7b8afba6b74dcf1df27ba92a68f1`이다.
작업지시자의 「T05 역시 나누지 않고 한 단위로 작업 범위를 잡고 진행해주세요」를 계획·
구현·검증·완료 보고·로컬 커밋까지의 승인으로 기록한다. 아래 D01–D06을 모두 끝내며
중간 절편마다 승인을 다시 요청하지 않는다.

## 실제 수명과 종결 범위

S36의 Native 제한, S37의 `PdfImportRequest`와 기존 Worker·Studio 보존 계약을 출발점으로 한다.
PDF 입력의 JS 옵션/입력 사본 → WASM 가져오기 → inspect/render/export 응답 → 편집 후보의
HWPX 재열기 → Studio의 원자적 교체까지가 범위다. 성공 후 일반 편집 수명은 독립적으로 유지한다.

| ID | 이번에 모두 처리할 내용 | 완료 근거 |
| --- | --- | --- |
| D01 | 옵션·입력의 실제 사본, 선택 view와 caller 소유권, WASM 최초 복사 전 예약/실패 전파 | 거대한/미지 옵션의 복사 전 거절, 실제 입력 한도·작은 decoded 예산·복구, 원본 buffer 유지 |
| D02 | inspect/render/export의 문자열·binary backing과 전송 경계, 응답 schema·한도·fatal 폐기 | 실제 제품 한도에 따른 거절/정상 출력, 불필요한 전체 사본 방지, 실패 Worker 재사용 거절·새 요청 복구 |
| D03 | 생성한 편집 HWPX의 재열기 admission·입력/압축 해제 사본·Core 예약과 deadline 검사 | 실제 남은 예산을 재열기까지 전달하고 성장 전 검사, 실패 시 후보 폐기, 다음 정상 요청과 일반 편집/저장 보존 |
| D04 | 입력 읽기·변환·재열기·교체를 잇는 Studio 취소/deadline·generation·후보 소유 | 대화상자를 교체 완료까지 유지, await 뒤와 교체 직전 재검사, 대기 중 취소/다른 문서 열기의 후보 역전 방지, 기존 문서/draft/recents/저장 핸들 보존 |
| D05 | S36 Native와 현재 Worker·Studio의 실제 정상/자원/종료·복구 검증 | 실행한 OS/권한·경계별 결과, 실제 Worker 종료/재생성·WASM trap, UI 실패/취소/복구·편집·양쪽 저장 |
| D06 | 현행 capability/사용 가이드·감사표·출력·비용·source/제품 증적 확정 | 적용한 경계와 host 책임 일치, 최적화 fresh WASM·영향 전쪽/쪽수/직접 판독·필수 lint/test, 필수 잔여 0건 |

Worker는 소유자가 종료해 실제 실행을 중단한다. main thread의 동기 WASM 호출은 같은
스레드의 timer/AbortSignal로 선점할 수 없다는 기존 계약을 유지한다. 재열기의 실제 예약/작업
경계와 교체 전 취소·deadline을 검사하고 이벤트 루프에 양보한 뒤 최종 교체를 판단한다.
이 구분을 숨기거나 Worker timer를 main thread 선점·전체 브라우저 RSS 상한으로 표시하지 않는다.
카운터는 실제 소유 payload의 논리 작업량이며 `parserAllocationBoundsProven=false`를 유지한다.

## 실행과 검증

1. 현재 caller/producer·기존 해결 커밋·공개 검사와 T04 증적을 고정한다. 현재 실패를 먼저 재현한다.
2. D01–D04의 공통 경로를 구현한다. 조판 규칙·기존 회귀 기대값·한도/baseline을 완화하지 않는다.
3. 정식 자동 수집 경로의 Rust 공개 검사·공통 Worker Node·Studio TypeScript/npm·실제 브라우저를
   실행한다. Native 접근 제한과 실행 중단·실패 후 정상 요청도 실제 제품에서 확인한다.
4. Rust/WASM 변경의 필수 lint·전체 회귀·Native Skia·최적화 fresh WASM을 완료한다. 기존
   21개·87쪽 및 바뀐 Studio 재열기의 같은 원본/독립 PDF를 대조해 쪽수·내용·출력을 확인한다.
   영향 페이지의 현재 compare/overlay/review를 직접 판독한다.
5. 입력/출력 사본·WASM 선형 메모리·변환/재열기 시간과 제품 크기를 구분해 기록한다. 실제
   프로세스/브라우저 RSS 보증으로 바꾸지 않는다. 증적은 ignored
   `output/pr-review/issue8174/t05-completion-20261028/`에 보존한다.
6. D01–D06의 잔여가 0건일 때만 T05와 TODO를 완료로 갱신하고 결과·코드·가이드를 커밋한다.

T06–T09·PR 게시·registry 업로드·태그는 이번 완료에 포함하지 않는다. T05 필수 항목을
그쪽으로 옮겨 완료를 대신하지 않는다. 공용 `target/pr-review`의 Cargo 작업은 순차 실행한다.

## 완료 판정

2026-10-28. D01–D06을 한 단위로 구현·실제 검증·보고했으며 필수 잔여는 0건이다.
[T05 완료 보고](../working/task_m100_8174_t05_completion.md)에 정확한 명령·source/제품·전체 회귀·
실제 Worker/Studio·Native 격리·전쪽 TSV/직접 판독·비용과 실패 보정 이력을 연결했다.
실제 최종 교체의 빈 화면을 해소하기 위한 Canvas context 수명 보정도 D04에 포함했다.
T06–T09·원격 PR/registry/태그를 이번 완료로 표시하지 않는다.
