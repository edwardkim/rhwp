# PR #1202 검토 — HWPX 미주/각주 prefixChar 파싱 (마커 접두문자 '문' 복원)

- **작성일**: 2026-06-01
- **PR**: #1202 (OPEN)
- **컨트리뷰터**: @planet6897 (핵심 컨트리뷰터 — #1182/#1164/#1148 머지, 미주/수식 시리즈)
- **연결 이슈**: #1199 (`closes #1199`)
- **base/head**: `devel` ← `fix/1199-note-prefixchar` (`826dacb7`)
- **mergeable**: MERGEABLE / **BEHIND** (merge-base `c884205d`, devel `84819f99` → 로컬 머지 필요)
- **규모**: 7 파일, +353/−0 (소스 `section.rs` +118 단 1파일, 나머지 docs/plans/stage 6)
- **CI**: 전부 SUCCESS (Build&Test / Analyze ×3 / CodeQL). WASM skip.
- **라벨**: enhancement / 마일스톤 v1.0.0

## 1. 문제와 원인 (PR 본문 검증)

`samples/3-09월_교육_통합_2022.hwpx` 9~10쪽 미주 마커 **"문N)"** 가 **"N)"** 로 렌더링됨
(접두문자 '문' 탈락). 한글 2022 PDF 는 "문1)"~"문7)".

근본 원인: HWPX `<hp:endNote>`/`<hp:footNote>` 는 접두/접미문자를 코드포인트 숫자 속성으로
인코딩(`prefixChar="47928"`=0xBB38 '문', `suffixChar="65289"`=0xFF09 '）'). 파서
`parse_ctrl_endnote()`/`parse_ctrl_footnote()`(`src/parser/hwpx/section.rs`) 가 **suffixChar
만 읽고 prefixChar 분기가 없어** `before_decoration_letter` 가 0 → `note_decoration_char(0)`
= None → 접두 탈락. 렌더 경로(`format_endnote_marker_text`, typeset.rs)는 이미 접두 지원 →
**파서 단일 원인**. (코드 확인: 정확한 진단.)

## 2. 수정 내용 검토

`section.rs` 두 함수에 `suffixChar` 와 대칭으로 `b"prefixChar"` 분기 추가:
```rust
b"prefixChar" => {
    // 코드포인트 숫자 파싱 → before_decoration_letter
    note.before_decoration_letter = v;   // 47928 (0xBB38 '문')
}
```
- prefixChar 없으면 0(접두 없음) 유지.
- **모델/렌더러/HWP3/공통 모듈 무변경** (파서 단일 파일) — 확인.
- 회귀 테스트 2건:
  1. `test_parse_note_prefix_char_maps_to_before_decoration_letter`: endnote+footnote
     prefixChar=47928 → before=47928, suffixChar=65289 → after=65289.
  2. `test_parse_note_without_prefix_char_keeps_zero_before_letter`: prefixChar 없으면
     before=0 유지 (회귀 방지).

## 3. 위험 평가

- **낮음.** 파서 단일 파일, 순수 추가(+118/−0), 기존 suffixChar 와 대칭 분기. prefixChar 없으면
  기존 동작(before=0) 유지 → 무회귀. 미주/각주 양쪽 동일 적용.

## 4. 검증 결과 (로컬 머지 시뮬레이션 `pr1202-verify`)

| 단계 | 명령 | 결과 |
|------|------|------|
| merge | `git merge --no-ff` | ✅ CLEAN (충돌 0) |
| fmt | `cargo fmt --all --check` | ✅ clean |
| build | `cargo build` | ✅ Finished |
| 전체 테스트 | `cargo test --tests` | ✅ **1896 passed, 0 failed** |
| 신규 회귀 2건 | prefix_char 매핑 / 누락 시 0 | ✅ 2 passed |
| **미주 '문' 접두 복원 (정량)** | 보정 전/후 SVG '문' 개수 | ✅ 9쪽 **0→7**, 10쪽 **0→6** |
| **PDF 정답지 정합** | 한글 2022 pdf 9쪽 "문1)~문7)" | ✅ 9쪽 7개 일치 |
| CI(PR) | Build&Test / Analyze ×3 / CodeQL | ✅ 전부 SUCCESS |

- 보정 전(devel): 9·10쪽 미주 마커 '문' **0개**(결함 재현).
- 보정 후: 9쪽 7개, 10쪽 6개 복원. 9쪽 7개 = PR 본문 "문1)~문7)" + 한글 2022 PDF 9쪽 마커 수 일치.
- 산출물: `output/poc/pr1202/3-09월_교육_통합_2022_009.svg`/`_010.svg`,
  PDF 정답지 `output/poc/pr1202/pdf_p-09.png`/`pdf_p-10.png`.

## 5. 비고

- 우측 단 "다른 풀이" 미표시는 별개 사안(#1200, curve 도형 외곽선 — PR #1203 에서 처리).
- @planet6897 미주/수식 시리즈: #1202(미주 prefix)/#1203(curve)/#1208(수식 토큰) 동시 진행.
  본 PR 은 독립적(파서 미주 단일) — 다른 PR 과 충돌 없음.

## 6. 판단 — 머지 완료

진단 정확 + 파서 단일 파일 + 회귀 테스트 + 정량/PDF 정합 + CI green → **머지**.
작업지시자 시각 판정 통과 후 메인테이너 로컬 `--no-ff` 머지(`84819f99..73034de9`) + push.
PR #1202 → MERGED, 이슈 #1199 → CLOSED. WASM 빌드 완료. 결과는 `pr_1202_report.md`.
