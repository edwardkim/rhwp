# #3128 최종보고서 — continuation fragment 세로 배치 정합

- **Issue**: #3128
- **브랜치**: `codex/issue-3128-continuation-geometry`
- **작성일**: 2026-08-18 KST
- **상태**: **로컬 검증 완료 — commit·push·PR 게시 승인 완료**

> 하이퍼워터폴 계획 승인 전에 구현이 선행된 사실은 수행계획과 피드백 문서에 소급 고지했다. 이후
> 작업지시자 승인으로 최신 devel 전체 검증까지 완료했고, 이어서 commit·push와 한국어 PR 생성도
> 승인받았다. PR은 `devel`을 대상으로 하며 이슈는 merge 전까지 열린 상태로 유지한다.

## 1. 결과 요약

#3128은 #4764를 기다리지 않고 독립 수정할 수 있는 issue-specific layout 결함으로 확인됐다. 저장
`LINE_SEG`가 없는 장문 child의 tracking·반각 공백, `applyInnerMargin=false` content-box, terminal
RowBreak 빈 host tail 중복 예약을 좁은 구조 조건으로 보정했다.

로컬 결과는 다음과 같다.

- 기준과 동일한 82쪽
- p34 continuation 외곽과 후속 직접편익 표 anchor가 PDF 허용 오차 안에 진입
- `연동시스템 등` 줄바꿈 일치
- 수정 전 약 60px 후속 흐름 하강 제거
- focused renderer 회귀와 683-sample overflow baseline 통과
- 최신 devel release-test 6,895건과 Native Skia 64건 통과
- 네이티브 최적화 WASM·wasm-opt 통과, 8,679,328 bytes
- 34쪽 pixel match 87.70% → 90.03%

## 2. 범위 제한

- 일반 셀 recomposition API는 기존 동작을 유지한다.
- 새 경로는 literal-space 들여쓰기, 동일 font metric, 구간별 tracking, 1×1 content-box 조건을 모두
  만족할 때만 활성화된다.
- p81→p82 short owner child와 legacy bullet 경로를 명시적으로 제외한다.
- #4764의 전역 font/raster/paint 차이는 해결됐다고 주장하지 않는다.

## 3. 검증 상태

| Gate | 상태 |
| --- | --- |
| #3128 전용 테스트 | 통과 |
| 관련 pagination·nested table focused 회귀 | 통과 |
| 683-sample overflow baseline | 통과 |
| 34쪽 PDF visual sweep | 통과 |
| fmt·diff check | 통과 |
| 최신 upstream/devel 재기준화 | `0bc05ef81`, 완료 |
| 전체 release test·Clippy | 6,895 passed, Clippy warnings 0 |
| Native Skia 3종 | 58 + 2 + 4 passed |
| WASM gate | native optimized·wasm-opt 통과; Docker daemon/앱 부재로 컨테이너 경로 미실행 |
| commit·push·PR | 게시 승인 완료, 진행 중 |

## 4. PR 계획

- 대상: `devel`
- 제목: `#3128 분할 표 연속 페이지의 세로 배치 정합`
- 종료 문구: `Closes #3128`
- #4764: 참조만 하고 종료하지 않음
- 코드, 테스트, Hyper-Waterfall 절차 문서를 같은 PR에 포함

## 5. 게시 단계에서 갱신할 항목

1. PR 번호와 CI 결과는 GitHub PR에서 추적한다.
2. merge 이후 작업지시자 최종 판정을 기록한다.
