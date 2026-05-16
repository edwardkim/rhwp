# Task #937 구현계획서 — 복학원서 서명란 '(인)' 기호 렌더링 불일치

## 기본 원칙

- 원본 IR의 `U+F012B` 문자는 보존한다.
- 출력/측정 경로에서만 `(인)` 표시 문자열을 사용한다.
- 기존 PUA 매핑 함수와 테스트 패턴을 우선 재사용한다.

## Stage 1: 재현 고정 + 현재 경로 확인

**목표**: `samples/복학원서.hwp`의 서명란 PUA 코드포인트와 현재 렌더 치환 누락을 테스트로 고정한다.

**변경 후보 파일**

- `tests/issue_826.rs` 또는 신규 `tests/issue_937.rs`
- `src/renderer/layout/paragraph_layout.rs` 테스트 모듈

**작업**

1. `U+F012B`가 현재 매핑되지 않는 RED 테스트 작성
2. 단일 문자 매핑으로 충분한지, 다문자 표시 문자열 헬퍼가 필요한지 확인
3. 기존 `map_pua_bullet_char` 호출 지점이 복학원서 표 셀 텍스트 경로에 닿는지 확인

**완료 기준**

- 실패 테스트가 이슈 본질을 표현한다.
- Stage 1 완료 보고서 `mydocs/working/task_m100_937_stage1.md` 작성 후 승인 요청

## Stage 2: PUA U+F012B 표시 문자열 치환

**목표**: `U+F012B`를 `(인)`으로 표시한다.

**변경 후보 파일**

- `src/renderer/layout/paragraph_layout.rs`
- `src/renderer/composer.rs`
- `src/renderer/svg.rs`
- `src/renderer/web_canvas.rs`
- `src/renderer/skia/text_replay.rs`

**작업**

1. `U+F012B -> "(인)"` 표시 문자열 헬퍼 추가
2. SVG/Canvas/Skia 텍스트 출력 경로에서 동일하게 적용
3. 폭 측정 경로가 표시 문자열 기준으로 계산되도록 정합

**완료 기준**

- 신규 테스트 GREEN
- 기존 PUA 매핑 테스트 GREEN
- Stage 2 완료 보고서 `mydocs/working/task_m100_937_stage2.md` 작성 후 승인 요청

## Stage 3: 복학원서 회귀 검증

**목표**: 실제 fixture에서 서명란이 `(인)`으로 출력되고, 기존 복학원서 정합 회귀를 깨지 않는지 확인한다.

**작업**

1. `cargo test --test svg_snapshot issue_677_bokhakwonseo_page1`
2. 필요 시 `tests/golden_svg/issue-677/bokhakwonseo-page1.svg` 갱신
3. `rhwp export-svg samples/복학원서.hwp -o output/svg/task937`로 산출물 확인

**완료 기준**

- 관련 테스트 GREEN
- 변경된 SVG가 의도한 `(인)` 치환만 포함하는지 확인
- Stage 3 완료 보고서 `mydocs/working/task_m100_937_stage3.md` 작성 후 승인 요청

## Stage 4: 최종 정리

**목표**: 보고서와 오늘할일을 정리하고 커밋 가능한 상태로 만든다.

**작업**

1. 최종 보고서 `mydocs/working/task_m100_937_report.md` 작성
2. `mydocs/orders/20260517.md` 상태 갱신
3. `cargo test` 범위 재확인
4. `git status`로 미커밋 파일 확인

**완료 기준**

- 최종 보고서 승인 요청 가능 상태
- 작업 브랜치에 단계별 문서와 소스 변경이 정리됨

## 승인 요청

본 구현계획서 승인 후 Stage 1부터 진행한다.
