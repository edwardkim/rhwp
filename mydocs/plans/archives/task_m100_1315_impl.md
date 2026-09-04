# 구현 계획서 — Task M100 #1315

## 개요

- **이슈**: #1315 "HWPX 기본 시리얼라이제이션 baseline 구축"
- **수행 계획서**: `mydocs/plans/task_m100_1315.md` (승인됨)
- **브랜치**: `local/task1315`
- **단계 수**: 5단계

## 설계 방침

1. **기존 코어 재사용**: `src/serializer/hwpx/roundtrip.rs`의 `roundtrip_ir_diff()`를 그대로 사용한다. IR 의미 비교 원칙(바이트 비교 금지)을 유지한다.
2. **serializer 본체 무수정**: `serializer/hwpx/{section,context,table}.rs` 등 본체는 건드리지 않는다 (PR #1366 재제출 충돌 회피). 신규 코드는 검증 전용 모듈·CLI·테스트로 한정한다.
3. **측정 우선, 수정 금지**: 전수 측정에서 실패가 나와도 이번 타스크에서 serializer를 고치지 않는다. xfail 분류 + 사유 기록까지가 범위다 (이슈 주의사항).
4. **누적-만-가능**: 기존 `tests/hwpx_roundtrip_integration.rs`의 Stage 0~5 테스트는 삭제·완화하지 않는다.

## 단계별 계획

### 1단계 — roundtrip 배치 명령 추가 + 전수 인벤토리 측정

**구현:**

- CLI 서브커맨드 `hwpx-roundtrip` 추가 (신규 파일 `src/diagnostics/hwpx_roundtrip_batch.rs` + `src/main.rs` dispatch 1줄)
  ```bash
  rhwp hwpx-roundtrip sample.hwpx -o output/poc/task1315/        # 단일 파일
  rhwp hwpx-roundtrip --batch samples/hwpx -o output/poc/task1315/  # 디렉토리 전수
  ```
- 파일별 측정 항목: parse 성공 / serialize 성공 / 재parse 성공 / IrDiff 건수 / 소요 시간
- 배치 결과를 stdout 표 + TSV(`output/poc/task1315/inventory.tsv`)로 출력
- roundtrip 산출 `.hwpx`를 `output/poc/task1315/{원본 stem}.rt.hwpx`로 저장

**산출물:** CLI 명령, 전수 인벤토리 TSV, `task_m100_1315_stage1.md` (전수 측정 결과 표 포함)

**검증:** `cargo test --lib`, `cargo fmt --check`, `cargo clippy --lib -- -D warnings`, 전수 배치 실행 완주(패닉 없음)

### 2단계 — 패키지 구조 검증기 추가

**구현:**

- 신규 모듈 `src/serializer/hwpx/package_check.rs`
- 재조립 HWPX ZIP에 대해 검사:
  - `mimetype` 엔트리 존재 + 내용 일치
  - `META-INF/container.xml`, `Contents/content.hpf` 존재 + content.hpf가 참조하는 엔트리 실재
  - `Contents/header.xml`, `Contents/section{N}.xml` 엔트리 수가 IR 섹션 수와 일치
  - BinData 엔트리 수·이름 보존 (원본 대비)
- 2-round 안정성 검사: `parse(serialize(parse(x)))` vs `parse(serialize(parse(serialize(parse(x)))))` IrDiff 0 (LINE_SEG reflow 함정 검출)
- 1단계 CLI에 패키지 검사 + 2-round 결과 컬럼 추가

**산출물:** `package_check.rs` + 단위 테스트, 갱신된 인벤토리 TSV, `task_m100_1315_stage2.md`

**검증:** 1단계와 동일 + 신규 단위 테스트 통과

### 3단계 — 샘플 등급화 + 전수 통합 테스트

**구현:**

- 1~2단계 측정 결과로 등급 확정:
  - **A (baseline pass)**: roundtrip + 재파싱 + IrDiff 0 + 패키지 검사 통과
  - **B (xfail)**: 실패하나 사유가 식별된 미지원 기능 — 사유 코드와 함께 기록
  - **C (oracle 부적합)**: roundtrip은 되나 full fidelity oracle 금지 명시 (`tac-img-02.hwpx` 류)
- 신규 통합 테스트 `tests/hwpx_roundtrip_baseline.rs`:
  - baseline 목록 전체: IrDiff 0 + 패키지 검사 통과 assert
  - xfail 목록: 크래시 없음만 assert + 사유 주석 (실패가 사라지면 알리도록 `should_panic` 대신 명시적 분기)
  - 대형 실문서는 `include_bytes!` 대신 경로 로드, 필요 시 `#[ignore]` + 사유
- 기존 `hwpx_roundtrip_integration.rs`는 무수정 유지

**산출물:** 등급표(보고서 내 표), `tests/hwpx_roundtrip_baseline.rs`, `task_m100_1315_stage3.md`

**검증:** `cargo test --test hwpx_roundtrip_baseline`, `cargo test --tests`, fmt/clippy

### 4단계 — 대표 산출물 생성 + 시각 판정 요청

**구현:**

- baseline 등급 중 대표 샘플 5~8개 선정 (빈 문서/텍스트/표/그림/양식/대형 실문서 각 1개 이상)
- `output/poc/task1315/` 에 roundtrip `.hwpx` 생성 + 원본/roundtrip SVG export 비교 자료 생성
- rhwp-studio 열림 확인 (E2E 또는 수동 로드 확인 — headless Chrome)
- **작업지시자 판정 요청**: 한컴에디터에서 대표 roundtrip 파일 열기 오류/무응답 여부 (Windows 환경)

**산출물:** 대표 roundtrip 파일 세트, SVG 비교 자료, `task_m100_1315_stage4.md` (판정 요청 목록 포함)

**검증:** rhwp-studio 로드 성공, 작업지시자 한컴 판정 결과 기록

### 5단계 — 문서화 + 최종 보고

**구현:**

- `mydocs/manual/hwpx_roundtrip_baseline.md` 작성:
  - 검증 절차 (CLI 사용법, 테스트 실행법, 산출물 위치)
  - 등급표 (baseline / xfail+사유 / oracle 부적합)
  - 최소 호환성 기준 (HWPX 저장 API 공개 판단 기준)
  - full fidelity oracle 오해 방지 가이드
- CLAUDE.md에 `hwpx-roundtrip` 명령 안내 추가 (간략)
- 최종 보고서 `mydocs/report/task_m100_1315_report.md`
- 오늘 할일 갱신

**산출물:** 매뉴얼, 최종 보고서, `task_m100_1315_report.md`

**검증:** push 전 CI급 검증 — `cargo test --tests` + `cargo test --lib` + `cargo fmt --check` + `cargo clippy --lib -- -D warnings` + `cargo check --lib --target wasm32-unknown-unknown`

## 커밋 계획

- 단계별 1~2 커밋, 메시지 `Task #1315: ...`
- 단계별 완료보고서(`_stage{N}.md`)는 해당 단계 소스 커밋과 함께 커밋
- 기능 변경과 포맷 변경 분리, 전체 `cargo fmt --all` 미실행

## 위험 및 대응

| 위험 | 대응 |
|------|------|
| 전수 측정에서 예상보다 많은 실패 | xfail 분류로 흡수, serializer 수정 확전 금지. 심각한 결함은 별도 이슈 등록 제안 |
| 파일명에 공백·한글 포함 샘플 | 배치 처리에서 경로 인용 처리, TSV에 원본명 그대로 기록 |
| 대형 실문서 처리 시간 | release 빌드 측정, 테스트는 경로 로드 + ignore 옵션 |
| PR #1366 재제출 충돌 | serializer 본체 무수정 방침으로 회피 |
