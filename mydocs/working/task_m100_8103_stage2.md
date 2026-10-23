# #8103 재검토와 비동기 경계 보강

Issue: #8103

## 검토 기준과 구현 전 분석

- 검토 head: `cebd9040d28f95d5f950b3b06258f4c47da60553`.
  최신 devel은 계속 `def947c2149df5bc812d801e6e79be19a403b4d1`이다.
  이슈 본문·댓글·중복 PR을 재조회했으며 #8103 PR은 아직 없다.
- 사용자 요청에 따라 코드와 검증 주장을 다시 대조한다. 기존 98개 Chromium E2E는
  기본 Canvas2D만 사용했다. auto mutation guard는 단위 대역만 실행했으므로 실제 auto
  renderer의 quiet 재선택·지연 mutation·문서 전환과 두 쪽 보기의 증거가 부족하다.
- 검토 중 발견한 경쟁 가설: 이전 mutation의 300ms auto 재선택 타이머가 새 mutation의
  metrics await 중 발화할 수 있다. `selectNextDocumentRevision`이 새 revision을 무효화하면
  올바른 fencing도 새 mutation의 layout 완료를 폐기한다. 선택 재투영 완료가 누락되는지
  실제 타이머와 제품 메서드, 실제 브라우저에서 수정 전에 확인한다.
- 별도 가설: auto의 후속 backend/metrics 변경에서 `refreshPages`를 다시 실행하더라도
  현재 선택을 새 좌표로 재투영할 완료 알림이 없는지 확인한다.
- 기존 정답은 유지한다. 현재 유효 선택·확정 페이지 원점 관계, 새 선택 대조군,
  스크롤·anchor/focus·내용 순서와 실제 삭제 범위를 검사한다. 고정 픽셀·skip·허용치
  완화로 검증을 맞추지 않는다. 문서 조판·#8102·다른 PR의 구현은 변경하지 않는다.

## 검증 계획

실제 auto/CanvasKit 경로와 기존 Canvas2D, mutation 직전 이전 quiet 타이머가 남은 경계,
새 revision·문서 교체·dispose와 async 역순 완료를 점검한다. 확정 배치 완료 이후에만
mutation count를 증가시켜 다른 source의 이벤트가 검사 대기를 거짓으로 만족하지 않게 한다.
보정이 생기면 새 source commit의 fresh WASM·TypeScript·전체 unit·production build·관련
E2E·문서 링크·manifest·공백 검사를 재실행한다. 기존 실패와 보정 결과를 구분해 기록한다.

## 수정 전 재현과 보정

1. 이전 quiet 타이머가 새 mutation의 metrics await 중 실행되면 최신 epoch가 폐기된다.
   실제 `RendererSession`과 `CanvasView` 메서드를 사용하는 타이머 검사에서 이전 소스는
   2 PASS / 1 FAIL이었다. 실제 auto 브라우저에서도 450ms 지연 후 완료 대기가
   타임아웃됐다. 새 mutation 시작 시 이전 예약을 취소하고, 최신 mutation 완료 뒤
   새 quiet 예약을 유지한다.
2. quiet 재선택이 backend 또는 metrics를 바꾸면 배치는 다시 계산하지만 선택 완료
   이벤트가 없었다. 완료 직전 current/dispose를 검사하고 실제 배치 갱신 뒤에만
   `renderer-reselection` source로 알린다. 변경 없음·stale·dispose는 알리지 않는다.
3. focus 캐럿 DOM도 삭제 전 한 쪽 원점에 남았다. 기존 소스에서 마지막 쪽 캐럿 top은
   772.9, 확정 배치 관계의 기대값은 10975.4였다. 기존 `CaretRenderer.updatePosition`으로
   현재 배율·쪽 원점에 재투영한 후 두 값이 일치했다. 이 수치는 진단 관측값이며 회귀의
   고정 px 정답으로 쓰지 않는다. 자동 스크롤과 논리 cursor·anchor/focus는 변경하지 않는다.

제품 메서드를 사용하는 단위 경계 검사는 보정 후 4 PASS / 0 FAIL / 0 SKIP이다.
확장 Chromium E2E의 선행 실행은 Canvas2D·auto 합계 278 PASS / 0 FAIL이며,
원본 글꼴 공급 검사를 더한 실행은 280 PASS / 0 FAIL이다. 이 수치를 새 커밋의 최종
재실행 결과와 혼동하지 않는다. 최종 실행 결과는 결과보고에 추가한다.

## 실제 글꼴과 CanvasKit 추가 진단의 실패

- 클라우드 Chromium에서 CDN 직접 요청과 proxy 요청은 각각 fetch 실패와
  `ERR_CERT_AUTHORITY_INVALID`였다. TLS 검증을 해제하지 않았다.
- 기존 proxy/CA를 사용하는 curl로 원본 `HANBatang.woff`를 내려받았다. SHA-256은
  `b3192018e3bd4da76b29b0530f71c79fb6f465a2b7cd41308bc38f9a4ae09a6c`다.
  선택 E2E의 선택 사항 `--font-cache`는 정확히 해당 공개 URL에 같은 바이트만 공급한다.
  renderer·metrics·선택 rect는 대역하지 않으며, 공급 횟수와 실제 backend를 검사한다.
- 실제 CanvasKit 강제 실행은 전체 삭제·Undo·Redo·스크롤·줌의 앞선 검사를 통과했지만
  지연 metrics 중 글꼴 family 미준비 오류가 발생하고 선택 해제 단계에서 타임아웃됐다.
  전체 실행 결과는 **실패(exit 1)**다. Canvas2D fallback을 CanvasKit 성공으로 세지 않았다.
- Vite 메모리 내 transform으로 두 제품 파일만 검토 전 `cebd9040` 소스로 공급한 독립
  대조 실행에서도 같은 글꼴 오류와 같은 단계의 타임아웃을 재현했다. 작업 트리를
  되돌리지 않았으며 같은 fresh WASM·원본 글꼴·확장 E2E를 사용했다. 이전 소스에서도
  존재한다는 증거이지 현재 실패가 해결됐다는 증거는 아니다. CanvasKit 강제 모드의
  지연 경계 완료를 성공 또는 skip으로 표시하지 않는다. 렌더 자원 수명·호스트 복구는
  #8103의 선택 오버레이 수정에 섞지 않았다.

## 제출 상태

기존 본인 PR #8178은 다른 세션의 작업이다. 이번 검토 중 head가 변경되어 CI가 다시
실행 중인 것을 확인했다. 해당 PR을 수정하지 않았다. 새 PR 생성 전 기존 PR 상태를
다시 확인하고 저장소 우선순위 절차를 적용한다. 사용자 개인 브라우저의 직접 확인은
자동 Chromium 검증과 별개로 남는다.
