# Task M100 #6788 — 혼합 글자 서식 보존 구현계획서

- Issue: [#6788](https://github.com/edwardkim/rhwp/issues/6788)
- 기준: `51ad998e33ef7f5191b0e1b0b656dc44cef33a1c`
- 상태: **3단계 완료 (2026-09-06) — push·PR 생성 승인 대기**
- 단계 결과: [1단계 완료보고서](../working/task_m100_6788_stage1.md)
  · [2단계 완료보고서](../working/task_m100_6788_stage2.md)
  · [3단계 완료보고서](../working/task_m100_6788_stage3.md)
  · [최종보고서](../report/task_m100_6788_report.md)
- 상위 문서: [수행계획서](task_m100_6788.md)

## 1. 설계 결정

현재 저수준 `Paragraph::apply_char_shape_range`는 범위를 단일 ID로 교체하는 함수다.
이 함수의 의미를 바꾸지 않고, 그 위에서 기존 구간을 열거하여 각 구간에 `CharShapeMods`를
적용한다. 명시적으로 모양 ID 전체를 지정하는 API의 기존 의미는 보존한다.

Undo/Redo는 ID 기반 복원 원칙을 유지하되, 문단당 ID 하나를 **구간 경계와 ID 목록**으로 바꾼다.
전체 문서 snapshot으로 본문 서식 history를 교체하지 않는다. 기존 F5·머리말/꼬리말의 snapshot
경로는 그대로 두고 적용 공통 처리와 함께 정상 복원되는지 확인한다.

## 2. 1단계 — 코어 적용

수정 후보:

- `src/model/paragraph.rs`: 선택 범위와 기존 `char_shapes`가 겹치는 구간을 얻는 내부 처리.
- `src/model/document.rs`: 기존 shape ID별 `find_or_create_char_shape` 재사용.
- `src/document_core/commands/formatting.rs`: 본문·일반 셀·중첩 셀의 단일 base ID 적용을 공통 처리로 연결.
- `src/document_core/commands/header_footer_ops.rs`: 같은 원인의 머리말/꼬리말 적용 연결.
- `tests/cases/issue_6788_mixed_char_format.rs`: 코어 보존·경계·저장 왕복 회귀 원본.

알고리즘:

1. 기존 좌표 변환을 재사용해 `[start, end)`와 겹치는 모양 구간을 **변경 전에** 확보한다.
2. 각 원본 ID에 동일한 mods를 적용해 새 ID를 찾거나 생성한다. 반복되는 원본 ID는 재사용한다.
3. 선택 밖 구간과 경계의 원래 모양을 보존하면서 선택 안 구간만 교체한다.
4. 인접한 동일 ID 구간만 정규화한다. 문단 끝·제어문자 경계의 원래 의미는 유지한다.
5. 원래 mutation API의 후처리에서 raw stream·dirty·flow 변경 판정과 rebuild를 수행한다.
   구간마다 reflow·pagination을 반복하지 않는다.

1단계 테스트:

- 검정/보라/검정 + shadeColor만 변경, 보라에서 시작하는 부분 선택.
- 색이 같아도 굵기·글꼴이 다른 구간에서 미지정 속성 보존.
- 단일 구간, 빈 범위, 선택이 구간 중간에서 시작/끝, 문단 끝, 보조 평면 문자·제어 offset.
- 본문, 일반·중첩 셀, 머리말/꼬리말의 실제 지원 경로.
- 새 문서는 `createBlankDocument`와 동등한 유효 기본 스타일을 가진 경로로 만든다.
  더미 `createEmpty`만으로는 제품 새 문서의 전제가 충족되지 않는다.
- 기존 paint-only·텍스트 흐름 변경 판정과 관련 서식·셀 reflow 계약을 함께 확인한다.

종료: 코어 적용의 focused 계약 성공과 단계 보고·commit. 이 시점의 Studio Undo 결함은
해결 완료로 표시하지 않으며 2단계 승인을 요청한다.

## 3. 2단계 — 구간 단위 history

### 3.1 구간 전송 계약

범위 조회·복원 payload는 다음 의미를 갖는다. API 이름은 아래 후보를 기준으로 기존 export와
충돌을 확인해 확정하고, 외부 API 의미가 달라지면 구현 전에 계획 차이를 보고한다.

```ts
interface CharShapeRun {
  startOffset: number; // 해당 문단 기준 문자 offset, 포함
  endOffset: number;   // 해당 문단 기준 문자 offset, 제외
  charShapeId: number;
}
```

- 후보: `getCharShapeRuns` / `setCharShapeRuns` 및 셀 `...InCellByPath` 변형.
- 전달 offset 단위는 기존 char formatting API와 같은 문자 단위다. IR UTF-16 위치를 그대로
  JS 문자 offset으로 넘기지 않는다.
- 조회 결과는 요청 범위를 빈틈·겹침 없이 덮는다. 빈 범위는 빈 목록이다.
- 복원은 대상·범위·ID·구간 정합을 mutation 전에 검사한다. 잘못된 목록의 중간 적용은 허용하지 않는다.
- Rust에서 문단 단위로 복원하고 기존 invalidation/reflow 후처리를 한 번 수행한다.
- 경계 밖 원본 모양, 끝 sentinel, 중첩 셀 소유 경로를 보존한다.

수정 후보:

- `src/document_core/commands/formatting.rs`: 조회·복원 네이티브 경로 및 구간 검증.
  구현에서는 같은 commands 계층의 `formatting_runs.rs`로 분리하여 기존 대형 파일과
  독립적으로 범위 검증·복원 계약을 관리한다(공개 API/기능 범위는 계획과 동일).
- `src/wasm_api.rs`: WASM export 연결.
- `rhwp-studio/src/core/types.ts`, `rhwp-studio/src/core/wasm-bridge.ts`: typed bridge와 입력/응답 정합.
- `rhwp-studio/src/engine/command.ts`: `ParaFormatEntry`의 before/after 구간 목록 저장·복원.
- 관련 mutation registry·binding 계약: 새 mutation export가 있으면 기존 분류에 명시적으로 등록.

### 3.2 Studio 실행·Undo·Redo

1. 최초 실행은 문단별 선택 범위를 산정하고 before 구간 목록을 확보한다.
2. 기존 apply 호출로 서식을 바꾸고 after 목록을 확보한다.
3. Undo는 before 목록, Redo는 after 목록을 같은 범위에 복원한다.
4. 본문·셀 경로 모두 적용하고 셀은 최내곽 `cellPath` 의미를 유지한다.
5. 문단 여러 개의 batch 정책과 실패 처리·history stack 이동 계약을 유지한다.
6. 필요한 export가 없는 오래된 WASM을 단일 ID 복원으로 조용히 fallback하지 않는다.
   배포 산출물의 JS/WASM 정합 및 기존 호환성 계약을 확인하고 명시적 처리 방침을 기록한다.

2단계 테스트:

- 실제 `ApplyCharFormatCommand` + `CommandHistory` + 새 WASM으로 원 신고 시나리오 실행.
- 적용/Undo/Redo 전체 구간의 ID 또는 모양 비교. 첫 글자만 확인하는 테스트로 끝내지 않는다.
- 같은 구역의 복수 문단, 일반·중첩 셀, 단일 구간 대조군, 여러 회 Undo/Redo.
- F5 셀 블록과 머리말/꼬리말 snapshot 경로의 결과 대조.
- malformed 구간 payload에서 문서가 부분 변경되지 않는지 확인.
- Node용 WASM 미준비에 따른 skip을 통과로 간주하지 않고 fresh 빌드로 실제 실행한다.

기존 `rhwp-studio/tests/support/pending-char-shape.runner.mjs`의 실제 클래스 import·WASM
어댑터 방식을 재사용하되, 이 이슈 전용 행위 회귀는 명확한 이름으로 분리한다.

## 4. 3단계 — 최종 검증과 인계

검증 순서:

1. 최종 후보의 focused Rust/Studio 회귀와 HWP·HWPX 저장·재적재.
2. 최신 web WASM과 Studio 빌드, 실제 형광펜 UI → Undo → Redo → 저장·재열기.
3. Firefox 확장 빌드·실행 가능 환경을 확인하고 원 신고 경로를 직접 검증한다.
4. 아래 필수 Rust lint 묶음과 전체 integration 회귀, Native Skia 3종, TS·npm gate.
5. 적용 전/후/Undo/Redo 화면과 구간 데이터의 일치, 선택 밖 무변경을 기록한다.
6. 단계·최종 보고, 남은 한계, PR 본문 초안과 정확한 검증 head를 준비한다.

별도 review worktree에서 파생 suite를 준비하고 아래 Cargo 명령은 고정 target으로 순차 실행한다.

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
cargo fmt --all
cargo fmt --all -- --check
cargo clippy --locked --target-dir target/pr-review -- -D warnings
cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown \
  --target-dir target/pr-review -- -D warnings
cargo build --locked --workspace --target-dir target/pr-review
cargo clippy --locked --workspace --all-targets --target-dir target/pr-review -- -D warnings
node scripts/rust-test-suite-manifest.mjs --check
cargo nextest run --locked --cargo-profile release-test \
  --target-dir target/pr-review --tests --no-fail-fast
```

source-side `#[cfg(test)]`를 변경한 경우에만
`node scripts/rust-unit-test-tiers.mjs --check`를 추가한다.
Native Skia 3종과 WASM 환경별 명령은 [로컬 검증](../manual/pr_review/local_validation.md) 및
[개발 환경](../manual/dev_environment_guide.md)을 따른다. Docker 부재 시 대체 빌드와 그 한계를 명시한다.
Node 자식 프로세스가 포함된 Studio npm test는 프로젝트 메모리에 따라 sandbox 밖에서 실행한다.
이번 계획 문서만으로 최종 광범위 검증을 시작하지 않고 3단계/PR 준비 승인을 받는다.

## 5. 위험과 판정

| 위험 | 대응 |
| --- | --- |
| 적용만 수정하고 Undo는 기존 단일 ID 유지 | 독립 구간 보존·Undo·Redo assertions로 두 계약을 구분한다. |
| 문자 offset/UTF-16 혼용 | 보조 평면 문자·제어 offset·선택 경계 계약을 둔다. |
| 셀 경로를 flat 좌표로 되돌림 | `cellPath`와 실제 최내곽 문단을 검증한다. |
| 구간마다 재조판하여 지연 증가 | 문단 단위 mutation·후처리와 호출 횟수를 확인한다. |
| stale JS/WASM으로 잘못된 green | 빌드 source SHA·artifact와 실제 사용 경로를 기록한다. |
| 새 범위 API의 부분 실패 | 전체 목록 사전 검사 후 mutation한다. |
| 브라우저 확인을 Firefox 확인으로 오인 | 환경·버전·미실행 항목을 분리해 보고한다. |

최종 성공은 원 신고 재현이 정상화되고, 혼합 모양·경계·저장 왕복·Undo/Redo 계약과 필수
로컬 검증이 모두 충족되는 것이다. 승인 전 remote push·PR 생성·merge·이슈 종료는 수행하지 않는다.

실행 결과는 3단계 보고서에 기록했다. 사용자가 승인한 CLI 분리 검증에 따라 실제 Studio history와
CLI 저장 왕복·8개 PNG 전수 비교로 저장 파일을 검증하고, Chrome에서 사용자가 재열어둔 문서의
직접 UI를 확인했다. Firefox UI·OS 저장 창·최적화 배포본은 완료로 간주하지 않는다.
