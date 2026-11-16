# 열린 PR 7건 통합 검토 증적

브랜치 `review/integration-20261116-open-prs`, 검증 base
`53774cc1508591b6ac794950f0063452fbaf6e87`, 제품 소스
`167921b3c52d78d3da180b746b22d91af9b3d3cc`의 검토 기록이다.
2026-11-17 현재 아래 완료 항목과 진행 중 항목을 구분한다. 통합 PR 제출·CI·병합은 아직 완료하지 않았다.

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
  **13,696 PASS·0 FAIL·45 skip**. Native Skia lib, Native 그림 placeholder·직접 PDF,
  fmt·Native/WASM/all-target Clippy·workspace build·suite 정책·unit tier·문서 테스트도 exit 0이었다.
  [전체 회귀 결과](full-nextest-final.json)의 source와 검증 head를 구분한다.
- 독립 데스크톱 한컴 2020/2024의 **Hancom PDF Print 66종·74쪽**:
  [Native](native-claims-final.json)와 [fresh WASM](wasm-claims-final.json) 모두 전쪽 90% 이상이다.
  [Print 영수증](mcp-print-references.json)에 입력/PDF 해시·실제 제품·출력 경로를 연결했다.
  웹한글기안기·Microsoft Print to PDF의 원 PR 자료를 새 공식 점수로 재사용하지 않았다.
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

## 진행 중인 필수 항목

fresh WASM 정본 보호 전쪽, 고정 base/후보의 두 출력 비교, 공개 공통 4조건,
명시적 Native 편람 383쪽 gate와 실제 Native/Studio Canvas2D/지원 CanvasKit 소비 검증이 남아 있다.
이 항목의 최종 결과·직접 판독을 확인하고 원 PR별 판정을 갱신한 뒤 통합 PR을 제출한다.
