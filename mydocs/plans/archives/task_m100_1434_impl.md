# Task M100 #1434 구현계획서 — 누름틀 안내문 command 한컴 포맷 정정

- 이슈: #1434, 마일스톤 M100, 브랜치 `local/task1434`
- 작성일: 2026-06-19
- 수행계획서: `mydocs/plans/task_m100_1434.md`

## 1. 수정 본체 — `build_clickhere_command` 한컴 포맷 정정

### 1.1 현재 (`src/model/control.rs:313`)

```rust
pub fn build_clickhere_command(guide: &str, memo: &str, name: &str) -> String {
    let guide_len = guide.chars().count();
    let memo_len = memo.chars().count();
    let name_len = name.chars().count();
    let mut inner = format!(
        "Direction:wstring:{}:{} HelpState:wstring:{}:{} ",  // ← trailing 공백 1개
        guide_len, guide, memo_len, memo
    );
    if !name.is_empty() {
        inner.push_str(&format!("Name:wstring:{}:{} ", name_len, name));  // ← Name 키
    }
    let total_len = inner.chars().count();  // ← set = inner 전체
    format!("Clickhere:set:{}:{}", total_len, inner)
}
```

### 1.2 정정안

```rust
/// 누름틀 command 문자열을 한컴 포맷으로 재구축한다.
///
/// 한컴 정답지(field-01/form-01) 동형:
///   `Clickhere:set:{N}:Direction:wstring:{gl}:{guide} HelpState:wstring:{ml}:{memo}  `
/// - HelpState 값 뒤 공백 2개(구분 1 + trailing 1).
/// - set 길이 N = inner_len − 1 (마지막 trailing 공백 제외).
/// - 필드 이름(Name)은 command 에 넣지 않는다 — CTRL_DATA 레코드(0x57) 전담(#1434).
pub fn build_clickhere_command(guide: &str, memo: &str) -> String {
    let guide_len = guide.chars().count();
    let memo_len = memo.chars().count();
    let inner = format!(
        "Direction:wstring:{}:{} HelpState:wstring:{}:{}  ",  // 공백 2개
        guide_len, guide, memo_len, memo
    );
    let set_len = inner.chars().count() - 1;  // trailing 1개 제외
    format!("Clickhere:set:{}:{}", set_len, inner)
}
```

검증(수행계획서 2절): 여기에 입력(6) → `set:48` + inner 49, 제목 입력(5) → `set:47`
+ inner 48 — 한컴 2케이스 바이트 동형.

## 2. 호출처 정합 (name 인자 제거)

- `src/model/control.rs` — 시그니처 `(guide, memo, name)` → `(guide, memo)`.
- `src/document_core/queries/field_query.rs:1247` — `build_clickhere_command(guide, memo, name)`
  → `(guide, memo)`. 이름은 `ctrl_data_name: Some(name)` 로 이미 별도 저장(불변).
- `src/wasm_api.rs:4045` — `build_clickhere_command(guide, memo, "")` → `(guide, memo)`
  (기존에도 name="" 전달이라 동작 불변, 시그니처만 정합).

## 3. 단계별 구현

### 1단계 — command 정정 + 회귀 가드 단위 테스트
- `build_clickhere_command` 한컴 포맷 정정 + 시그니처 단순화.
- 단위 테스트 (`src/model/control.rs` tests):
  - `task1434_clickhere_command_hancom_format`: 여기에 입력/제목 입력 2케이스 → 한컴
    원본 문자열과 정확히 일치(set 길이·공백 2개·Name 부재).
  - `task1434_command_has_no_name_key`: name 정보가 command 에 안 들어감.
  - guide_text/memo_text 왕복: 생성한 command 에서 안내문·메모 재추출 정합.
- 호출처 2곳 시그니처 정합.
- `cargo test --lib model::control` + 빌드 그린.

### 2단계 — 저장→재파싱 정합 + 전수/CI
- 누름틀(안내문 포함) IR → HWP 저장 → 재파싱 → guide_text/memo_text/field_name(=CTRL_DATA
  이름) 보존 확인 (round-trip 테스트).
- 기존 field 파싱 회귀 0 (field-01/form-01 dump 불변 확인).
- `cargo test --profile release-test --tests` + fmt + clippy.

### 3단계 — 한컴 판정 + 문서
- 누름틀+안내문 HWP 샘플 산출(`output/poc/task1434/`) → **작업지시자 Windows 한컴
  편집기 바인딩 판정** 요청.
- 트러블슈팅 `hwp_clickhere_guide_command_format.md` + 단계별/최종 보고서.

## 4. 검증

- 단위: 한컴 2케이스 바이트 동형 + Name 부재 + guide/memo 왕복.
- 저장→재파싱: guide/memo/name 보존.
- `cargo test --profile release-test --tests` + fmt + clippy.
- **한컴 편집기 바인딩 판정** (Windows 정답지, 최종 게이트).

## 5. 위험과 대응

| 위험 | 대응 |
|------|------|
| field_name 회귀(Name 제거) | field_name은 ctrl_data_name 우선 → CTRL_DATA 이름 보존. 재파싱 테스트로 봉인 |
| 기존 저장본 command 보존 경로(wasm_api) | guide/memo 불변 시 command 보존 로직 유지 — 변경 시에만 새 포맷 |
| 자기 검증 ≠ 한컴 호환 | 3단계 한컴 편집기 판정 필수 게이트 |
