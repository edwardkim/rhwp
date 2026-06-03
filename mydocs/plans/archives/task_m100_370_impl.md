# Task #370 구현계획서

## 단계 1: 폴백 체인 + 테스트 수정

### 1.1 `src/renderer/mod.rs:558`

```rust
return "'Batang','바탕','AppleMyungjo','Noto Serif KR',serif";
```
→
```rust
return "'Batang','바탕','AppleMyungjo','Noto Serif KR','Noto Serif CJK KR',serif";
```

### 1.2 `src/renderer/mod.rs:565`

동일하게 변경 (영문 세리프 분기).

### 1.3 `src/renderer/mod.rs:935` 테스트

```rust
let serif = "'Batang','바탕','AppleMyungjo','Noto Serif KR',serif";
```
→
```rust
let serif = "'Batang','바탕','AppleMyungjo','Noto Serif KR','Noto Serif CJK KR',serif";
```

### 1.4 검증

- `cargo build`
- `cargo test generic_fallback`
- `cargo test` 전체 (회귀 확인)

## 단계 2: 통합 검증

- `./target/release/rhwp export-svg samples/hwpx/form-002.hwpx -p 9 -o /tmp/task370/`
- 출력 SVG에서 `Noto Serif CJK KR` 포함 확인
- 브라우저 렌더 시 볼드 적용 확인 (해당 문구의 시각적 두께)

## 단계 3: 보고서 + 정리

- `mydocs/working/task_m100_370_stage1.md`
- `mydocs/working/task_m100_370_stage2.md`
- `mydocs/report/task_m100_370_report.md`
- `local/task370` → `local/devel` merge 준비
