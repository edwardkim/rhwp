> **PR base 브랜치가 `devel` 인지 확인해주세요** (`main` 아님 — GitHub 기본 선택이 main 일 수 있습니다).
> 작업 브랜치는 최신 `upstream/devel` 에서 생성합니다. 상세: [CONTRIBUTING.md](https://github.com/edwardkim/rhwp/blob/devel/CONTRIBUTING.md)

## 변경 요약

이 PR이 해결하는 문제와 변경 내용을 간결하게 설명해주세요.

## 관련 이슈

closes #

## 테스트

<!-- 해당하는 항목만 선택하고 실행 명령·결과를 적어주세요. 해당 없음은 사유를 적고, 실패·미실행을 PASS로 표시하지 마세요. -->

- 변경 범위: <!-- Rust / Studio 단독 / 혼합 / package / 문서 / 기타 -->
- 검증한 commit SHA / 정책 비교 PR base SHA:
- 실행 명령·결과 및 해당 없음 사유:

- [ ] [변경 범위별 필수 검증](https://github.com/edwardkim/rhwp/blob/devel/CONTRIBUTING.md#pr-전-체크리스트)을 수행하고, 제출 HEAD가 검증한 commit과 같음을 확인
- [ ] Rust source·test/baseline helper·Rust 검증 입력 변경 시: [별도 worktree 준비](https://github.com/edwardkim/rhwp/blob/devel/CONTRIBUTING.md#rust-검증-worktree-준비와-실행) 후 `cargo fmt --all -- --check`, native·WASM32·workspace all-target Clippy 통과
- [ ] Rust 변경 시: 범위에 해당하는 focused·전체 integration·Native Skia 회귀 및 시각 검증 수행
- [ ] 조판 영향 여부를 실제 소비 경로로 판단했고, 영향이 있으면 Native/fresh WASM Visual Sweep·페이지별 TSV·직접 review/overlay 확인 완료 (한 페이지라도 **90% 미만** 또는 측정 불가이면 본인 branch에서 수정·재실행; 정확히 90%는 통과)
- [ ] 기준 PDF는 원본 저장 버전에 맞는 한컴 **Print 인쇄 경로**로 출력했고, 제품/빌드·인쇄 설정·출처 확인 ([MCP 2020/2024 및 수동 출력 계약](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/mcp_hwp2024Convert_usage.md#기준-pdf-인쇄-계약))
- [ ] 새 integration test는 원본을 `tests/cases/*.rs`에만 추가했고 `tests/generated/`, `tests/suites/manifest.json`, 일반 PR의 Cargo generated test target을 포함하지 않음 (`--sync-cargo-targets` 메인터너 registry PR은 marker 블록만 예외)
- [ ] `src/**` 또는 `crates/*/src/**`의 `#[cfg(test)]` 변경 시: `node scripts/rust-unit-test-tiers.mjs --check --base-ref <검증한-PR-base-SHA>` 통과 (무생성 검사)
- [ ] Studio 변경 시: [테스트 전용 unit 또는 fresh dev WASM package 검증](https://github.com/edwardkim/rhwp/blob/devel/CONTRIBUTING.md#프런트엔드-변경-검증) 통과, 브라우저 동작 변경 시 관련 E2E·실제 동작 확인 · 명령/결과:
- [ ] 편집 command·Undo/Redo 변경 시: [편집 체크리스트](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/edit_command_review_checklist.md) 확인; E2E 미실행 시 사유와 대체 증적 기록
- [ ] npm/editor 변경 시: 해당 package·embed 계약 검사 수행
- [ ] 문서 변경 시: 최신 base 기준 `git diff --check upstream/devel...HEAD` 및 `git diff --check`, 링크·내용 정합성과 변경한 실행 절차 확인
- [ ] (에이전트 보조 작업 권장) 작업 증빙 첨부 — `rhwp replay --capsule` 영수증 캡슐 또는 관련 `--json` 봉투 원문 ([AGENTS.md 작업 증빙 절](https://github.com/edwardkim/rhwp/blob/devel/AGENTS.md#작업-증빙--에이전트-기본-경로-권장))
- [ ] `.claude/agents/`, `.claude/skills/`, `.agents/skills/` 변경 시: [capability 카탈로그](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/agent_capability_registry.md)의 등록·검증 규칙을 반영

> Rust 검증이 필요하면 기여자 본인이 원본 commit의 별도 review worktree를 만들어
> `--prepare` → fmt·lint·해당 회귀 → manifest `--check` 순서로 검증합니다. source 제출 checkout에서
> 생성하지 않으며, 파생 파일은 PR에 포함하지 않습니다. suite 누락으로 실패한 fmt를 PASS로 기록하지 마세요.
> Studio 단독 변경은 frontend 검증, 혼합 변경은 두 범위를 모두 적용합니다. 최신 GitHub required checks도
> 충족해야 합니다. 상세 순서는 [공개 검증 절차](https://github.com/edwardkim/rhwp/blob/devel/CONTRIBUTING.md#rust-검증-worktree-준비와-실행)를 따릅니다.

## 조판 변경 근거 (해당하는 경우)

<!-- 조판 변경이 없으면 실제 소비 경로에 영향이 없다는 비해당 근거를 테스트 절에 적으세요. renderer 파일을 바꾸지 않았다는 사유만으로 면제하지 않습니다. 증적의 기존 표·링크를 재사용할 수 있습니다.
분할·이어받기는 AGENTS.md의 "분할·이어받기 변경의 입증"을 적용합니다. 체크 표시나 helper 이름만으로 충족 처리하지 마세요.
아래 표는 기존 증적의 구체적인 위치로 대신할 수 있습니다. 경로에는 helper 이후 값 덮어쓰기·별도 배치를,
검사 항목에는 주장한 위치·테두리·내용 보존을 실제로 판별하는 assertion 또는 Visual Sweep 관측을 연결하세요. -->

[구현 주장과 검증 증거의 대조](https://github.com/edwardkim/rhwp/blob/devel/AGENTS.md#구현-주장과-검증-증거의-대조)에 따라 기록합니다.

| 구현 주장 | 적용·비적용 경로 | 독립 기대값·실제 검사 항목 | 수정 전후 결과·증적 | 판정·남은 범위 |
| --- | --- | --- | --- | --- |
| | | | | 충족/미충족/미검증/비해당 및 근거 |

- 위반된 계약과 입력 생성 방식(실제 저장본/합성·수동 수정), 저장 정보 재사용/편집 후 재조판의 적용 범위:
- 분할 변경이면 컷·유닛 소유, 요구/예약/배치 높이, 예산 실패·종료 처리의 코드 위치와 위 표의 증거 연결:
- 내용으로 대응시킨 Visual Sweep 페이지·영역, 앞뒤 조각·다음 내용 확인, baseline 변경 근거:
- 부분 개선이면 남은 문제와 이슈 종료 여부 (PR 본문·최종 merge 메시지에 동일 범위 적용):

## Visual Sweep 직접 증적 (해당하는 경우)

<!-- 조판/렌더링 영향 변경은 파일 경로와 관계없이 Native/fresh WASM Visual Sweep·TSV를 반드시 실행합니다.
리뷰 요청 전에 최종 PR head에 있는 대표 review·overlay PNG를 아래에 실제 Markdown 이미지로 표시한다.
경로·임시 output·review 문서 링크만 남기지 않는다. PR 번호가 아직 없으면 issue 또는 변경 주제의 안정 경로를
사용하고, raw URL에는 PR head의 repository owner/name과 정확한 head SHA를 쓴다. 이후 코드가 바뀌면
새 head의 PNG와 URL로 다시 캡처·교체한다. 비해당이면 이 절을 제거하고 사유를 위 검증 결과에 적는다. -->

- 문서 비교: [PDF/SVG visual sweep 가이드](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#pr-body-visual-evidence)를 따름
- **페이지별 TSV 명령·저장 위치:** [「실루엣 보조값만 빠르게 TSV 산출」](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#실루엣-보조값만-빠르게-tsv-산출)의 Native/fresh WASM 예제를 각각 실행
- 신규 렌더링 회귀 추가 시: 관련 전체 범위의 Native/fresh WASM 최저 일치율 / 최저 페이지 / manifest: <!-- 최저 90% 미만·측정 불가이면 신규 회귀를 추가하지 않습니다. 쪽수 검사는 전체 페이지, 평균값·글꼴 예외로 면제하지 않습니다. -->
- PR head repository / SHA:
- 입력·기준 PDF·대응 페이지·영역:
- 한컴 제품/빌드·저장 제품 판정·Print 출력 방법/설정·PDF 출처/해시 (MCP이면 engine·job 식별자):
- TSV 실행 명령·source/build SHA·로컬 산출 경로: <!-- TSV·실행 로그·중간 JSON은 ignored output에 보존하고 Git에 커밋하지 않습니다. -->

| TSV 출력 경로 | 검증 문서·페이지 범위/전체 쪽수 | 최저 일치율·페이지 | 90% 미만/누락/측정 불가 쪽 | 결과·증적 출처 |
| --- | --- | --- | --- | --- |
| Native | | | | |
| fresh WASM | | | | |

<!-- 한 페이지라도 90% 미만 또는 측정 불가이면 스스로 PDF/overlay 원인을 재검토·수정하고 새 head에서 재실행합니다.
평균값·높은 페이지 선별·글꼴 차이·CI 녹색으로 면제하지 않습니다. 쪽수/분할 변경은 전체 페이지를 비교합니다. -->
- 사람 판독 및 남은 차이: <!-- 자동 수치는 보조값이며, 표·그림·줄바꿈·테두리·앞뒤 내용의 직접 판독을 적는다. -->

| 출력 경로 | review | overlay |
| --- | --- | --- |
| Native | ![Native review](https://raw.githubusercontent.com/OWNER/REPOSITORY/SHA/REVIEW_PNG_PATH) | ![Native overlay](https://raw.githubusercontent.com/OWNER/REPOSITORY/SHA/OVERLAY_PNG_PATH) |
| fresh WASM | ![fresh WASM review](https://raw.githubusercontent.com/OWNER/REPOSITORY/SHA/WASM_REVIEW_PNG_PATH) | ![fresh WASM overlay](https://raw.githubusercontent.com/OWNER/REPOSITORY/SHA/WASM_OVERLAY_PNG_PATH) |

<!-- 조판 영향 변경에서는 Native/fresh WASM 행을 생략하지 않습니다. 실제 asset 존재와 PR 본문 Markdown을
PR 생성·수정 뒤 다시 확인한다. merge 뒤 comment에는 같은 asset의 merge SHA 고정 URL을 별도로 사용한다. -->

## 성능 영향 및 측정 결과 (해당하는 경우)

- 예상 영향: <!-- 개선 / 회귀 가능성 / 영향 없음 / 미확인 -->
- 재현·측정: <!-- 공개 sample, 명령, 환경, 변경 전후 관측값. 측정 환경이 없으면 "미측정" -->

> 특정 장비의 절대 성능 수치나 메인테이너 전용·비공개 벤치마크 통과는 PR 제출 조건이 아닙니다.
> 공개된 결정적 성능 회귀 테스트와 GitHub required checks는 기존과 같이 적용됩니다.

## 스크린샷

변경 전후 비교가 필요한 경우 첨부해주세요.
