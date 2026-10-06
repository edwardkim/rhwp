# Task #7280 Stage 56 — R6 구조 정본·변경 관리·재계측

- Issue: #7280. 이전: [Stage55](task_m100_7280_stage55.md).
- 승인: 작업지시자의 “다음 진행을 승인합니다.”에 따라 R6 구조 문서·규칙 지도·재계측 진행.
- 시작 head: `52d7fbfe8`, 제품 SHA: `7947ee45fa560eab3dd6f2d6a7af2ce94f759e88`.
- 상태: **2026-09-23 R6 문서·재계측 완료, 최종 제출 통합 게이트 대기**.
  제품·테스트·IR·API·baseline·ignore·CI 변경과 원격 push·PR·댓글은 없다.

## 구조 정본과 변경 경로

[조판 책임 경계와 변경 지도](../tech/typesetting_architecture.md)를 추가했다.
문단→컨트롤→하위 문단 구조, 실제 모듈 지도, Query/조정/Command,
상태 소유자, 대표 의미 ID 8개와 입력/결과·소비자·기존 검사를 연결했다.
기존 표 의미 규칙은 중복 작성하지 않고 `table_layout_rules.md`로 연결한다.
CONTRIBUTING, tech 지도, 렌더링 설계, canonical manifest에는 진입 링크만 추가했다.

추가·수정·삭제/대체·구조 이동의 기여자 확인점을 기존 PR 절차에 연결했다.
별도 양식·규칙 등록 서비스·새 CI를 만들지 않았으며 새로운 테스트는 `tests/cases/`에 두는
현행 정책을 그대로 따른다. 기존 private 테스트를 제품에 남긴 것은 신규 추가 허가가 아니다.

실제 소비 경로를 다시 검색하여 줄 조회의 소비자는 일반 paragraph scan이 아니라 root의
수식/TAC 판별과 미주 prepare/fit/paragraph임을 문서에 반영했다.
표 각주 큐 검사는 존재하는 `issue_5966_queued_table_footnote_fresh_page`로 연결하되
쪽수/완주만 검사하는 한계를 명시했다. 독립적인 모든 helper 경계 검증을 새로 했다고 주장하지 않는다.

확인한 구조 경계:

- `TypesetState`의 private data와 불변 Deref, 상태 소유 모듈의 쓰기 메서드. 넓은 읽기 관측면은 잔존.
- scan의 ScanProgress, continuation context/cursor, section 지역 상태의 별도 소유.
- Native drain/WASM job의 공통 continuation과 최종 결과 이동.
- LayoutEngine 컷 계산·scratch 미주 측정, document_core pile helper 의존과 엔진 Cell/RefCell 잔존.
- 기존 허용치/호환 예외는 **근거 확인 필요**이며 이번 구조 이동으로 의미 타당성을 승인하지 않음.

## 동일 조건 재계측

최초 측정 `18a9fa85e`와 실제 작업 baseline은 다르다. 따라서 기존 숫자를 그대로 전후 비교하지 않고
clean baseline worktree `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`와 clean 제품 worktree
`7947ee45f`에서 다시 측정했다. Rust 1.93.1/Clippy 0.1.93, native 기본 feature/default-member,
CC 임계값 5 초과, JSON primary span 중복 제거로 같은 조건이다.

| typeset.rs + typeset/ 범위 | baseline | 제품 |
| --- | ---: | ---: |
| root 파일 줄 수 | 31,937 | 7,902 |
| 영역 파일 수 / 총 줄 수 | 2 / 32,037 | 109 / 38,891 |
| 1,200줄 초과 파일 | 1 | 4 |
| CC 수집 항목 / 수집 CC 합 | 50 / 1,254 | 73 / 1,095 |
| 최대 CC | 186 | 88 |
| CC >15 / >25 / >100 | 15 / 9 / 4 | 21 / 7 / 0 |

전체 CC >25는 114→112, 최대 CC는 186→180이며 typeset 외 진단의 위치·이름·값은 동일했다.
미주 문단(CC88), block 진입(CC84), 일반 행 스캔 closure(CC52), place_table_with_text(CC50)
등 7개 CC25 초과 항목은 남아 있다. 파일 분리로 모든 복잡도 목표를 달성했다는 판정은 하지 않는다.
총 줄 수 증가에는 명시적 입력/결과·명령·import·주석/빈 줄이 포함된다. CC는 실행 성능이 아니다.

로컬 증적은 `output/7280/r6-complexity/`의 `report.md`, `comparison.json`,
`{base,head}/metrics.json`, `{clippy,complexity}.jsonl/.stderr`다.
`run.sh`로 base→head 각 기본/CC Clippy를 순차 실행했고 4건 모두 exit 0 및
`build-finished.success=true`, 기본 경고 0건을 확인했다.
`summarize.mjs`는 최초 스크립트의 root/output만 인자화했다. `compare.mjs`는 비변경 영역까지 비교한다.
테스트/coverage는 이번 계측에서 null이며, 기존 dashboard 설정·tracked metrics snapshot은 변경하지 않았다.

## 검증과 남은 절차

- 변경 Markdown 링크 및 `git diff --check`: 통과.
- 메타데이터 전체 검사: 기존 4문서의 누락 16건으로 exit 1. 제품 worktree와 오류 집합을
  비교하여 **동일 16건, 신규 오류 0건**을 확인했다. 무관 문서는 수정하지 않았다.
  `metadata-{base,head}.log`에 남겼다. 검사기는 인자와 무관하게 전체 정본 경로를 검사한다.
- 제품 변경 없음. Stage55의 같은 제품 SHA에 대한 집중395·전체10,096통과/기존제외50,
  Native353쪽·WASM329쪽 데이터 및 선택19쪽 보존 증거를 연결하며 이번에 재실행했다고 쓰지 않는다.
- 현재 문서 작업에 전체 Cargo/시각 sweep을 반복하지 않았다. CC용 Clippy는 최종 제출 lint 묶음을
  대체하지 않는다. 회귀·시각 의미 검증은 [Stage55](task_m100_7280_stage55.md)의 한계를 그대로 유지한다.

R1–R5 구조 구현과 R6 구조 정본/변경 관리/계측이 마련됐다. 다음은 별도 승인 후 최종 제출 검증이다:
실제 제출 base/head 고정 → 적용 정책·fmt·Native/WASM/workspace Clippy와 workspace build →
전체 회귀·Native Skia·fresh WASM/시각 증거의 정확한 head 대조 → 결과보고·PR 본문 준비.
base 통합이나 제품 변경이 발생하면 영향 검증을 다시 수행한다. 원격 push·PR 생성과 이슈 종료는
별도 승인 절차이며 **#7280 전체 완료/제출 준비 완료로 보고하지 않는다**.
