# PR #1226 검토 — 수식 LEFT-RIGHT 구분기호 그룹 뒤 첨자 결합 (|x|^3)

- **작성일**: 2026-06-01
- **PR**: #1226 (OPEN)
- **컨트리뷰터**: @twoLoop-40 (**rhwp 첫 기여**)
- **연결 이슈**: 없음 (closes 미지정 — KICE 시험지 수식 결함 제보형)
- **base/head**: **`main`** ← `keepgong-eqn-superscript-fix` (cross-repo) ⚠️ base 이슈(아래 0절)
- **규모**: 1 파일, +4/−1 (`src/renderer/equation/parser.rs`)
- **mergeable**: MERGEABLE / BEHIND
- **CI**: 미실행 ("no checks reported") → 로컬 검증 필수
- **라벨**: enhancement / 마일스톤 v1.0.0

## 0. base 브랜치 이슈

PR 이 **`main`** 을 base 로 함. 우리 워크플로우상 외부 PR 은 `devel` 로 와야 한다(릴리즈 시
devel→main PR). 다만 변경이 parser.rs 1파일이고 devel 과 충돌 없이 적용되므로, **메인테이너가
devel 로 머지**하고 PR 에 base 안내를 코멘트로 남긴다. (head 는 `5c1f463e` v0.7.13 release 기반.)

## 1. 문제와 원인 (코드 확인 완료)

사이즈 구분기호 그룹(`left | x right |`) 뒤의 첨자(`^`/`_`)가 그룹에 결합되지 않음.
`|x|^3` 의 `3` 이 superscript 높이가 아니라 baseline 근처에 떨어져 렌더됨.

`parse_command`(`parser.rs:372`) 의 `LEFT` 분기가 `parse_left_right()` 결과를 **`try_parse_scripts`
없이 직접 반환**. 다른 모든 primary(Number 166/Text 179/group 197/symbol 321·531·541·546)는
`try_parse_scripts` 를 거쳐 trailing `^`/`_` 를 base 에 부착하는데, LEFT-RIGHT 만 누락 →
trailing script 가 base 없는 orphan `Superscript{ base: Empty, sup }` 가 됨. (코드 확인: 정확.)

## 2. 수정 내용 검토

```rust
// before
if cu == "LEFT" { return self.parse_left_right(); }
// after
if cu == "LEFT" {
    let node = self.parse_left_right();
    return self.try_parse_scripts(node);
}
```

- 다른 primary 와 **완전히 일관된** 1줄 위임. 순수 추가, 부작용 없음.

## 3. 검증 결과 (로컬 머지 시뮬 `pr1226-verify`, devel 기준)

| 단계 | 결과 |
|------|------|
| merge (devel 위) | ✅ CLEAN (parser.rs, 충돌 0) |
| fmt / build / clippy(lib) | ✅ CLEAN |
| 전체 테스트 `cargo test --tests` | ✅ **1921 passed, 0 failed** |
| **AST 보정 전/후** | ✅ 전: `Row([Paren{\|x\|}, Superscript{base:Empty,sup:3}])`(orphan) → 후: `Superscript{base:Paren{\|x\|}, sup:3}`(정상) |
| **회귀 탐침** | ✅ `x^2` 무영향 / `\|x\|_3`→Subscript 결합 / `\|x\|` 무첨자 그대로 / `\|x\|^2_3`→SubSup 정상 |
| **SVG 좌표 (시각)** | ✅ `3` y좌표 **보정 전 17.40(baseline) → 후 11.20(superscript, 위로 6.2px)**. x baseline=22.20 대비 명확히 위 |

산출물: `output/poc/pr1226/abs_x_pow3_{before,after}.svg`, `x_pow2_after.svg`.

## 4. 위험 평가

- **낮음.** 수식 파서 1줄, 다른 primary 와 일관. LEFT-RIGHT 그룹에만 영향, 첨자 없으면 무변화.
  회귀 탐침에서 x^2/subscript/중첩 모두 정상.

## 5. 판단 — 머지 권고 (시각 판정 게이트)

- 진단 정확(코드 확인), 수정 최소·일관, AST·SVG 좌표로 효과 실증, 회귀 0.
- 첨자 위치는 시각 영향 → **작업지시자 시각 판정** 게이트(`|x|^3` 의 3 이 위첨자로 렌더).
- base=main → 메인테이너가 devel 로 머지 + PR base 안내 코멘트. 첫 기여 → 따뜻한 환영.

## 6. 비고

- PR 이 회귀 테스트 미포함 — 머지 시 메인테이너가 LEFT-RIGHT script 결합 회귀 테스트 1건
  추가 권장(검토 중 AST 검증한 케이스).
- 수식은 한컴 핵심 경로지만 본 수정은 hwpeq 파서 일관성 정정으로 자체 검증으로 충분(스펙 추정 없음).
