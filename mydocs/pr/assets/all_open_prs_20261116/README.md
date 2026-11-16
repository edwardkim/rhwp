# 열린 PR 7건 통합 검토 증적

브랜치 `review/integration-20261116-open-prs`, 검증 base
`53774cc1508591b6ac794950f0063452fbaf6e87`, 제품 소스
`167921b3c52d78d3da180b746b22d91af9b3d3cc`의 검토 기록이다.
2026-11-17 현재 필수 로컬 검증을 완료했다. [통합 PR #9143](https://github.com/edwardkim/rhwp/pull/9143)을 제출했고 code head CI를 완료했다. 병합·후속 처리는 아직 완료하지 않았다.

## 원 PR별 근거

| 원 PR | 검토 범위 | 개별 기록·증거 |
| --- | --- | --- |
| #9022 | 칸 안 연속 그림 호스트·그림 없는 뒤 호스트·사진 행 높이 | [리뷰](../../archives/pr_9022_review.md), [반례 원인](picture-host-cause.md), [제목/사진 행](title-photo-cause.md), [같은 쪽 이미지 신원](frame-boundary-validation/image-manifest.json) |
| #9105 | 가져온 번호·글머리표의 원문/모델 글자 모양 ID 소속 | [리뷰](../../archives/pr_9105_review.md), 최소 글꼴 집중 14 PASS와 전체 회귀 내 실제 가져오기·두 형식 반복 저장 |
| #9107 | HWPX 1.2/1.4 평문 FIXED/BETWEEN 줄간격 | [리뷰](../../archives/pr_9107_review.md), [공개 API 편집·두 형식 반복 저장 16회](public-api-save-validation/linespacing-repeat-save-result.json) |
| #9109 | 저장 줄 없는 글상자의 자리차지 표 점유 | [리뷰](../../archives/pr_9109_review.md), [주장 페이지 Native/WASM 이미지·판독](claim-page-evidence/index.json) |
| #9127 | 설치 Noto Sans CJK KR의 고딕 대체 체인 소비 | [리뷰](../../archives/pr_9127_review.md), [최소 글꼴 Mac의 실제 Native glyph 선택·raster](noto-cjk-native-consumer.json) |
| #9137 | 저장 줄 있는 글상자 표 높이·정렬과 제한/비제한 머리 경계 | [리뷰](../../archives/pr_9137_review.md), [경계 원인](header-front-cause.md), [직접 판독](claim-page-evidence/index.json) |
| #9141 | 한컴 저장 줄의 FIELD_END 뒤 개체 소속 | [리뷰](../../archives/pr_9141_review.md), [변경 없는 두 형식 반복 저장 12회와 별도 편집 진단](public-api-save-validation/field-save-scope-candidate.json) |

세부 적용 순서·원 head·충돌 해소 출처는 [선행 분석](intake-analysis.md)을 따른다.
원 contributor의 브랜치를 rewrite하지 않았다. 중복된 선행 6개 커밋을 제외하고 29개를 누적 적용했다.

## 완료한 검증

- [Rust 13단계](rust-gates-final.json): 최소 글꼴 macOS 전체 nextest
  **13,696 PASS·0 FAIL·45 skip**. 관계형 검사 보정 뒤 검증 head `3e9ce11f9c`에서 888.636초의 새 전체 실행과
  9단계(full nextest·fmt·Native/WASM/all-target Clippy·workspace build·suite 준비/정책·unit tier)를
  다시 완료했다. 제품·해당 검사 소스가 같은 Native Skia lib·Native 그림 placeholder·직접 PDF·문서
  테스트 4단계는 `21950b8ab8`의 exit 0 실행을 신원을 보존해 재사용했다.
  [전체 회귀 결과](full-nextest-final.json)의 source와 검증 head를 구분한다.
- 독립 데스크톱 한컴 2020/2024의 **Hancom PDF Print 66종·74쪽**:
  [Native](native-claims-final.json)와 [fresh WASM](wasm-claims-final.json) 모두 전쪽 90% 이상이다.
  [Print 영수증](mcp-print-references.json)에 입력/PDF 해시·실제 제품·출력 경로를 연결했다.
  웹한글기안기·Microsoft Print to PDF의 원 PR 자료를 새 공식 점수로 재사용하지 않았다.
- [고정 base 대비 fresh WASM](wasm-base-head-comparison.json): 원 PR 대상 66종·74쪽 중
  상승 59쪽·하락 0쪽이다. 정본 보호 검증은 별도 전쪽 실행으로 확인했다.
- [검증 입력 커밋 감사](committed-input-audit.json): 66개 원본과 대응 PDF는 실행 바이트·HEAD의
  바이트·영수증 SHA가 모두 같고 PDF는 모두 50MiB 미만이다. 새 글꼴 바이너리를 넣지 않았다.
- [Native 정본 보호](native-protected-final.json): **18문서·1,771쪽의 공식 두 자리 하락 0쪽**.
  [조건 재현 분석](protected-condition-analysis.md)은 입력·PDF·판정 기준을 바꾸지 않고
  실제 독립 Print의 글꼴 공급자와 기존 수용 브라우저 조건을 재현한 근거다.
- [같은 주장 페이지의 직접 review·overlay 판독](claim-page-evidence/index.json)과
  [Native 잔여 영역 판독](native-direct-reading.json)을 수치와 별도로 기록했다.

## 판정에서 구분하는 범위

기존 결재·사진 대조군의 93%대 잔여 차이는 base/후보 PNG가 같다. 새로운 개선으로 계산하지 않는다.
밀린 앵커 대조군의 A1/A2 한 줄 피치 잔여도 같은 쪽 이미지에 공개하며 100%로 표현하지 않는다.

#9141은 **한컴이 저장한 줄의 슬롯 소속 복구**를 검증한다. 값을 바꾼 뒤 저장 줄 없는 글상자의
TAC 표를 폭으로 다시 나누지 못하는 현상은 [고정 base](public-api-save-validation/field-save-scope-base.json)와
후보에 모두 있고 원 PR의 「남은 범위」에도 명시돼 있다. 이를 편집 통과로 기록하거나 다른 누름틀
문제를 완전 해결했다고 주장하지 않는다. 원 PR·연결 이슈의 종료 범위는 실제 해결 여부로 판단한다.

정본 보호값은 Native/WASM별 행을 따로 대조한다. 예를 들어 편람 130쪽의 공식 값은
Native 99.97%, WASM 99.96%이며 Native 행을 WASM 기준으로 대체하지 않는다.
보호 하한·점수·기준 PDF·2px 판정을 낮추거나 서로 다른 조건의 쪽별 최고값을 합치지 않는다.

## 최종 보호 판정과 실제 소비 경로

[Native/fresh WASM 정본 보호](protected-final.json)는 18문서씩 36조건·3,542행에서
**공식 두 자리 하락 0쪽·상승 0쪽**이다. 보호 하한을 변경하지 않았다.
[sample10 원본 공급 재현](sample10-original-supply-validation.json)은 base/후보 Native/WASM을
각각 전 763쪽 측정해 최저 99.57%·보호 하락 0쪽을 확인했다. 원본 66개 프로그램은 동일 SHA로
확보했고 미확보 14개 Windows 프로그램은 전쪽 실제 WASM 요청에 없음을 구분했다.
Native는 원본 프로그램 선택과 독립 Print의 PUA 공급 선언을 함께 고정했다.
일반 제품의 TTF→TTC→OTF/CFF 선택 순서는 변경하지 않았다.

초기 잘못 공급한 Mac 환경의 757쪽 99.56%와 Native 자동 공급의 목차 98.25%는
[이전 실패 집계](protected-before-original-supply.json)·[조건 분석](protected-condition-analysis.md)에
진단 이력으로 보존하며 수용 결과로 재사용하지 않는다. 쪽별 최고값을 섞지 않았다.

[고정 base 대비 Native](native-base-head-comparison.json)도 66종·74쪽에서 상승 59쪽·하락 0쪽이다.
[공개 공통 4문서 Native/WASM 8조건과 명시적 Native 편람 383쪽](public-gates-final.json)은 통과했다.
[실제 소비 경로](actual-consumers-final.json)는 Native PNG 16개·Studio 화면/Print 48개를 산출했고,
CanvasKit 32개 캡처가 요청 backend를 사용했으며 runtime fallback·replay hard 위반은 없었다.
Native↔CanvasKit 대조 16건은 통과했다. 별도 report-only Canvas2D↔CanvasKit 대조 32건은
축소 fixture의 실제 잉크량이 복사된 `minimumInkPixels` 조건보다 작아 FAIL이며 통과로 바꾸지 않았다.
차이 예산은 충족했고 자동 선택 readiness gate는 실행하지 않았다. 같은 Print 페이지를
네 경로에서 직접 읽어 그림·표·문단 소속을 확인했다.

초기 48건의 CDP WASM 본문이 진단 캐시에서 사라진 문제는 별도 실제 초기화 재실행으로 보완했다.
Root pkg·Studio public·frozen JS/WASM과 실제 응답의 신원을 확인했다. Vite의 JS 로컬 WASM URL과
비실행 sourcemap 변경을 구분하며, WASM 응답은 원 바이트 해시가 일치했다.

## 제출 자료와 남은 원격 절차

[66개 입력·독립 Print PDF·base/후보 Native/WASM TSV 색인](visual-evidence-index.md)에 전체 원문을 연결했다.
개별 원 PR 판정은 로컬 검증 결과로 갱신했다. 통합 PR 최신 CI·병합·원 PR과 실제 해결 이슈의
종료·duration·devel 동기화·정리는 아직 원격 실행 결과를 확인해야 하는 후속 단계다.

## 통합 code head CI 완료

[code CI 영수증](code-ci-final.json)의 exact head `8fdcff5525`에서 여섯 workflow와 39개 check의
실제 결과를 확인했다. 기본 회귀 다섯 archive는 13,511 PASS·0 FAIL·45 skip이며
로컬 최소 글꼴 macOS 전체 13,696 PASS와 구분한다. Native Skia·frontend package gate도 성공했다.
전체 로그 다운로드의 GitHub storage HTTP 503은 개별 job 로그 수집으로 보완했으며 테스트 실패가 아니다.
339개 원격 파일의 해시와 PR 본문 이미지 23개의 실제 브라우저 표시도 확인했다.
현재 기록은 녹색 code CI 뒤 추가하는 문서-only trailing이며, 최신 check·병합·duration·이슈/원 PR 종료·정리는
실제 실행 뒤 별도로 확인한다.
