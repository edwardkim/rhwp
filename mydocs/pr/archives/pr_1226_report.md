# PR #1226 처리 보고서 — 수식 LEFT-RIGHT 그룹 뒤 첨자 결합 (|x|^3)

- **작성일**: 2026-06-01
- **PR**: #1226 → **MERGED** (devel, 로컬 `--no-ff` 머지 + 회귀 테스트 후속)
- **컨트리뷰터**: @twoLoop-40 (**rhwp 첫 기여** — 따뜻한 환영)
- **연결 이슈**: 없음 (KICE 시험지 수식 결함 제보형)
- **판단**: **머지** ✅ (작업지시자 시각 판정 통과)

## 결정 사유

`parse_command` 의 LEFT 분기가 `parse_left_right()` 를 try_parse_scripts 없이 직접 반환하여
`|x|^3` 의 `^3` 이 orphan 되던 결함을, 다른 primary 와 일관되게 try_parse_scripts 위임으로 해결.
최소 1줄, AST·SVG 좌표로 효과 실증, 회귀 0. 메인테이너가 회귀 테스트 1건 추가.

## 변경 요약 (parser.rs, +4/−1 + 회귀 테스트)

```rust
if cu == "LEFT" {
    let node = self.parse_left_right();
    return self.try_parse_scripts(node);   // 다른 primary 와 일관
}
```

메인테이너 추가: `left_right_group_binds_trailing_script` 회귀 테스트 (|x|^3 → Superscript base=Paren).

## base 브랜치 처리

PR base 가 **main** (우리 규칙은 devel). 변경이 devel 과 무충돌이라 메인테이너가 devel 로 머지 +
PR 에 base 안내 코멘트. head 는 v0.7.13(5c1f463e) 기반.

## 검증 결과

| 항목 | 결과 |
|------|------|
| merge (devel) | ✅ CLEAN |
| fmt / build / clippy(lib) | ✅ CLEAN |
| 전체 테스트 `cargo test --tests` | ✅ **1922 passed, 0 failed** (회귀 1건 추가) |
| AST 보정 전/후 | ✅ orphan `Superscript{base:Empty}` → `Superscript{base:Paren{\|x\|}}` |
| 회귀 탐침 | ✅ x^2 무영향 / \|x\|_3 Subscript / \|x\| 무첨자 / \|x\|^2_3 SubSup |
| **SVG 좌표 (시각)** | ✅ `3` y 17.40(baseline)→11.20(superscript, x baseline 22.20 대비 위) |
| **시각 판정** | ✅ **통과** (작업지시자, before/after SVG) |

산출물: `output/poc/pr1226/abs_x_pow3_{before,after}.svg`, `x_pow2_after.svg`.

## 처리 절차

1. PR 정보 — base=main(이슈), CI 미실행, parser.rs 1파일. @twoLoop-40 첫 기여.
2. LEFT 분기 진단 코드 확인 + try_parse_scripts 동작 검토. 트러블슈팅 검색(수식).
3. 로컬 `pr1226-verify`(devel): fmt/build/clippy(lib)/test / AST 전후 / 회귀 탐침 / SVG 좌표 정량.
4. 작업지시자 시각 판정 통과.
5. devel `--no-ff` 머지 + 회귀 테스트 후속 + push. base 안내 + 첫 기여 환영 코멘트.

## 비고

- 본 수정은 hwpeq 파서 일관성 정정(스펙 추정 없음) — 자체 검증으로 충분.
- base=main 은 다음 기여부터 devel 로 안내.
