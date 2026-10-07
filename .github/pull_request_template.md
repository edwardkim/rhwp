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
- [ ] 90% 미만의 해결 불가능한 글꼴 문제로 제출한다면 [글꼴 예외 계약](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#해결-불가능한-글꼴의-pr-제출-예외)의 공급 시도·배치/쪽수 일치 근거와 `font_mismatch_exception` 기록 첨부 (TSV 원값 유지, 측정 누락·배치 결함은 면제하지 않음)
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
PR 생성 시부터 대표 review·overlay PNG를 아래에 실제 Markdown 이미지로 표시한다.
기여자는 PR 번호를 미리 알 필요가 없다. PNG를 mydocs/pr/assets/issue_<이슈번호>_<주제>/ 또는
주제별 안정 경로에 커밋하고 실제 head repository에 push한 뒤 git rev-parse HEAD로 확인한 SHA를 URL에 넣는다.
경로·임시 output·review 문서 링크만 남기지 않는다. raw URL의 OWNER/REPOSITORY는 기여자 fork 등 실제
head repository를 사용하며 target repository로 바꾸지 않는다. 이후 코드가 바뀌면
새 head의 PNG와 URL로 다시 캡처·교체한다. 비해당이면 이 절을 제거하고 사유를 위 검증 결과에 적는다. -->

- 문서 비교: [PDF/SVG visual sweep 가이드](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#pr-body-visual-evidence)를 따름
- **페이지별 TSV 명령·저장 위치:** [「실루엣 보조값만 빠르게 TSV 산출」](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#실루엣-보조값만-빠르게-tsv-산출)의 Native/fresh WASM 예제를 각각 실행
- 신규 렌더링 회귀 추가 시: 관련 전체 범위의 Native/fresh WASM 최저 일치율 / 최저 페이지 / manifest: <!-- 최저 90% 미만·측정 불가이면 신규 회귀를 추가하지 않습니다. 쪽수 검사는 전체 페이지, 평균값·글꼴 예외로 면제하지 않습니다. -->
- PR head repository / SHA:
- 입력·기준 PDF·대응 페이지·영역:
- 한컴 제품/빌드·저장 제품 판정·Print 출력 방법/설정·PDF 출처/해시 (MCP이면 engine·job 식별자):
- TSV 실행 명령·source/build SHA·로컬 산출 경로: <!-- TSV·실행 로그·중간 JSON은 ignored output에 보존하고 Git에 커밋하지 않습니다. -->
- **전체 검증 범위의 Native TSV 원본 첨부 링크:** <!-- Native TSV만 제출합니다. WASM TSV 첨부는 필요하지 않습니다. Native TSV를 묶은 ZIP을 PR 본문에 업로드하거나 동일 파일의 다운로드 링크를 넣습니다. 입력·출력 경로·검증 head를 명시하고, 요약 표·최저값·로컬 경로만으로 대신하지 않습니다. -->
- **대표 페이지 이미지:** <!-- 변경 효과·주요 경계를 보여주는 대표 페이지에만 아래 표를 작성합니다. 전체 페이지의 PNG 생성·커밋·첨부는 필요하지 않습니다. 전체 검증 범위는 위 TSV로 제공합니다. -->

<!-- 입력마다 Native/fresh WASM 두 행을 반복합니다. 대표 이미지의 점수와 검증 범위 전체의 최저값을 구분하세요.
검증 source SHA·Native binary/fresh WASM 해시·인쇄 프로필·글꼴 공급과 실제 빌드 옵션도 위 환경 정보에 적습니다. -->

| 입력·기준 Print PDF | TSV 출력 경로 | 검증 페이지 범위/전체 쪽수 | 최저 일치율·페이지 | 90% 미만/누락/측정 불가 쪽 | 결과·증적 출처 |
| --- | --- | --- | --- | --- | --- |
| | Native | | | | |
| | fresh WASM | | | | |

<!-- 검증 범위의 한 페이지라도 90% 미만·누락·측정 불가이거나 필수 Native/fresh WASM 경로 미실행이면
PR 생성·제출 갱신·검토 요청을 하지 않습니다. 시간 제약·기존 차이·부분 개선·추후 검증 등의 사유로 허용하지 않습니다.
실패/미실행 사유는 제출 전 로컬 작업 기록이며 PR 제출 허가가 아닙니다. 자기 branch에서 수정·재실행해 통과한 뒤 본문을 완성합니다.
평균값·높은 페이지 선별·단순 글꼴 차이 추정·CI 녹색으로 면제하지 않습니다. 쪽수/분할 변경은 전체 페이지를 비교합니다.
공급으로 해결 불가능한 실제 글꼴 차이만 위 계약의 예외로 제출하고 근거·미달 쪽·최종 font_mismatch_exception을 명시합니다. -->
<!-- PR #7551의 Native/fresh WASM 증적 표처럼 대표 입력·페이지마다 아래 소제목/표/판독 묶음을 반복합니다.
https://github.com/edwardkim/rhwp/pull/7551
같은 입력·쪽의 Native/fresh WASM review/standalone overlay 네 이미지를 한 표에서 바로 대조하게 합니다.
대표 이미지 대신 파일 경로만 적거나 서로 다른 쪽을 섞은 표를 쓰지 않습니다. 전체 페이지 이미지 나열·전쪽 overlay 합성은 요구하지 않습니다. -->

### 입력명 · 기준 PDF pN ↔ rhwp pN

- 기준 Print PDF·입력 식별자: <!-- 위 전체 요약의 입력과 연결하고, 같은 이름의 파일은 경로/해시로 구분합니다. -->

| 출력 경로 | review | overlay |
| --- | --- | --- |
| Native | ![입력명 pN Native review](https://raw.githubusercontent.com/OWNER/REPOSITORY/HEAD_SHA/mydocs/pr/assets/ISSUE_OR_TOPIC/INPUT-pNNN-native-review.png) | ![입력명 pN Native overlay](https://raw.githubusercontent.com/OWNER/REPOSITORY/HEAD_SHA/mydocs/pr/assets/ISSUE_OR_TOPIC/INPUT-pNNN-native-overlay.png) |
| fresh WASM | ![입력명 pN fresh WASM review](https://raw.githubusercontent.com/OWNER/REPOSITORY/HEAD_SHA/mydocs/pr/assets/ISSUE_OR_TOPIC/INPUT-pNNN-wasm-review.png) | ![입력명 pN fresh WASM overlay](https://raw.githubusercontent.com/OWNER/REPOSITORY/HEAD_SHA/mydocs/pr/assets/ISSUE_OR_TOPIC/INPUT-pNNN-wasm-overlay.png) |

- 이 쪽의 실루엣 보조값·gate: Native / fresh WASM <!-- 실행한 값과 판정을 각각 적습니다. 개선을 주장하면 같은 입력·쪽의 수정 전후 값과 source SHA를 연결합니다. -->
- 사람 판독 및 남은 차이: <!-- 이 쪽에서 직접 확인한 표·그림·줄바꿈·테두리·앞뒤 내용과 남은 차이를 적습니다. 자동 수치는 보조값입니다. -->

<!-- 조판 영향 변경에서는 Native/fresh WASM 두 경로를 모두 검증한 뒤 각 경로의 이미지를 넣습니다.
미실행 경로의 행을 지우거나 사유만 적어 제출하지 않습니다. 실제 asset 존재와 PR 본문 Markdown을
PR 생성·수정 뒤 다시 확인한다. OWNER/REPOSITORY/HEAD_SHA/ISSUE_OR_TOPIC/INPUT/pNNN을 실제 값으로 치환한다.
ISSUE_OR_TOPIC에는 이슈 번호·주제 또는 주제 이름을 넣는다 (PR 번호 불필요).
본문의 네 이미지는 자신의 입력·페이지·출력 경로에 대응하는 제출 head 증적을 사용한다.
merge 뒤 comment에는 같은 asset의 merge SHA 고정 URL을 별도로 사용한다. -->

## 성능 영향 및 측정 결과 (해당하는 경우)

- 예상 영향: <!-- 개선 / 회귀 가능성 / 영향 없음 / 미확인 -->
- 재현·측정: <!-- 공개 sample, 명령, 환경, 변경 전후 관측값. 측정 환경이 없으면 "미측정" -->

> 특정 장비의 절대 성능 수치나 메인테이너 전용·비공개 벤치마크 통과는 PR 제출 조건이 아닙니다.
> 공개된 결정적 성능 회귀 테스트와 GitHub required checks는 기존과 같이 적용됩니다.

## 스크린샷

변경 전후 비교가 필요한 경우 첨부해주세요.
