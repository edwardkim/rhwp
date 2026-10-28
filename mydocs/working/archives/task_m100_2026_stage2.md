# 단계 완료 보고 — Task M100 #2026 2단계: 추출 1 (en_para 루프 → typeset_endnote_paragraphs)

- 작성일: 2026-07-07 / 브랜치: `local/task2026`

## 수행 내용

`typeset_section_endnotes`의 en_para 루프(4,934줄, #1904 라운드 1 이연 본체)를
`typeset_endnote_paragraphs`로 통추출 (동작 불변).

- **`EndnoteFlowState`** (Copy 7필드: vpos_offset/prev_en_bottom_vpos ×2/emitted_count/
  last_render_idx/rewind·overestimate 플래그 2) — 값 왕복, caller가 ref-루프 꼬리에서
  prev/current 스왑 유지. **`EndnoteEmitVars`** (Copy 8필드: 미주-당 읽기 플래그/스칼라).
- `pre_emitted_endnote_para_indices`(HashSet)는 루프 후 미사용 실측 → **함수 로컬로 흡수**
  (인터페이스 제외). `prev_para`는 클로저 파라미터 오탐 제거.
- 제어 흐름 수술 0 (return/`?` 전부 클로저 내부 — 사전 전수 확인).
- 컴파일러 검증 보정 3건: struct 정의 위치(impl 밖), `Endnote` 타입 경로(model::footnote),
  `EndnoteFlowProfile` 값-self Copy 계약(`Option<T>`로 수령).

## 게이트 결과 (전수 통과)

| 게이트 | 결과 |
|---|---|
| cargo fmt --check / clippy | 통과 / **경고 0** |
| cargo test --profile release-test --tests | **2,924 통과 / 실패 0** |
| 미주 표적 핀 issue_1116 (sample16 한컴 핀) | **13/13** |
| OVR baseline 5샘플 | **추가 변동 0** (기지 #1936발 3건 동일) |

## 계측

| 함수 | 이전 | 이후 |
|---|---|---|
| `typeset_section_endnotes` | 5,539줄 · 분기 1,925 | **648줄 · 133** |
| `typeset_endnote_paragraphs` (신규) | — | 4,397줄 · 1,792 |

CC 총량은 신규 함수로 이동(통추출의 계획된 결과) — 공식 CC 재계측은 4단계.
다음 분해 대상은 신규 함수 내부(후속 라운드 입력).

## 다음 단계

3단계 — 추출 2: 프리앰블(~530줄) 분리 + 신규 함수 내부 재실측으로 축소 여부 결정. 승인 후 착수.
