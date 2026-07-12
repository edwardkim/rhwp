# 단계 완료 보고 — Task M100 #2026 3단계: 추출 2 (미주-당 프리앰블 → prepare_endnote_emit)

- 작성일: 2026-07-07 / 브랜치: `local/task2026`

## 수행 내용

`typeset_section_endnotes`의 미주-당 프리앰블(구분자 방출·간격/되감기 플래그 산출, 529줄)을
`prepare_endnote_emit`으로 추출 (동작 불변).

- **`EndnotePrepCarry`** (Copy 6필드: vpos_offset/prev_en_bottom ×2/prev_rewind/
  emitted_separator/current_overestimate) — caller 로컬과 값 왕복. **추출 1의
  `EndnoteFlowState`/`EndnoteEmitVars` 인터페이스는 무접촉** (커밋 간 독립 검증 유지).
- prepare가 `EndnoteEmitVars`의 **생산자**가 됨 — 산출 플래그 6종 + endnote_start 스냅샷을
  함수 말미에서 조립해 반환, caller는 `emit_vars`를 그대로 본체에 전달.
- 이동 조정 1건: `let mut current_… = false;`(미주-당 리셋 선언) → 캐리 필드 리셋 대입.
  caller에 fn-레벨 선언 보충(컴파일러 검출). doc/attr 재배치 1건(clippy 검출).

## 게이트 결과 (전수 통과)

| 게이트 | 결과 |
|---|---|
| cargo fmt --check / clippy | 통과 / **경고 0** (재배치 후) |
| cargo test --profile release-test --tests | **2,924 통과 / 실패 0** |
| 미주 표적 핀 issue_1116 | **13/13** |
| OVR baseline 5샘플 | **추가 변동 0** (기지 3건 동일) |

## 계측 (라운드 5 누적)

| 함수 | 라운드 시작 | 현재 |
|---|---|---|
| `typeset_section_endnotes` | 5,539줄 · 분기 1,925 | **141줄 · 분기 8** (오케스트레이터화) |
| `prepare_endnote_emit` (신규) | — | 519줄 · 125 |
| `typeset_endnote_paragraphs` (신규) | — | 4,397줄 · 1,792 (다음 라운드 분해 대상) |

구조가 계획 §2의 목표 형태로 정착: **resolve → prepare(플래그/구분자) → 본체(en_para 조판)
→ 꼬리 스왑**의 명시적 파이프라인.

## 다음 단계

4단계 — 재평가(`--snapshot r5 --no-coverage`) + 공식 CC 대비 + 최종 보고. 승인 후 착수.
