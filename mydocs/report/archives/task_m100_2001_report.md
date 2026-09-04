# 최종 보고서 — Task M100 #2001: 1차 리팩토링 라운드 3 (parse_paragraph_list 해체 + 도구 소품)

- 이슈: #2001 (계획 #1883 v2, umbrella #1582, 선행 #1904·#1925) / 브랜치: `local/task2001`
- 기간: 2026-07-06 / 거버넌스: v2 (§0 stage-gate / §1 금지 목록 / §5 산식 / §6 PR 규격) 준수

## 1. 결과 요약

`parse_paragraph_list`(HWP3)에서 컨트롤 코드 처리 전체를 추출해 **CC 234 → 76 (공식,
−67.5%), 2,270 → 896줄 (−61%)**. 행동 회귀 0. **전체 1위 함수 3라운드 연속 해소**:

| 라운드 | 대상 | CC |
|---|---|---|
| 1 (#1904) | `typeset_section_with_variant` | 282 → 104 |
| 2 (#1925) | `layout_composed_paragraph` | 288 → 226 |
| **3 (#2001)** | **`parse_paragraph_list`** | **234 → 76** |

전체 최대 CC: 288(영점) → **226** (`layout_composed_paragraph`가 1위 복귀).

## 2. 산출물

### 추출 3함수 + 공유 상태 struct (전건 HWP3 격리 영역 내부, 소스분기 접촉 0)

| 신규 | 크기 | CC(공식) | 내용 |
|---|---|---|---|
| `Hwp3CharScan` | struct 5필드 | — | 11개 arm 공유 캐리오버 묶음. 스칼라(i, utf16_len)는 값 전달+반환 — 본문 무변경 이동 보장 |
| `parse_object_control_char` | 1,039줄 | 104 | GSO/개체 catch-all arm. break 17곳 전수 분류(중첩 추적) 후 반환값 치환 |
| `parse_simple_control_char` | 241줄 | 47* | 고정 크기 데이터 코드 9종 |
| `parse_field_control_char` | 118줄 | ~24 | 필드/감추기 계열 (18..=21) |

\* 공식 스냅샷 매핑 기준 근사 — 상세는 `mydocs/metrics/2026-07-06-r3/metrics.json`.

### 도구 소품 3건 (`scripts/metrics.sh` + 매뉴얼)
요약 줄 버그(CC>25 자리에 전체 개수) 수정 · `--snapshot <라벨>`(같은 날짜 충돌 방지 +
기존 폴더 경고) · `--no-coverage` — **본 라운드 재평가에서 실전 검증 완료**
(`2026-07-06-r3/` 라벨 폴더, tarpaulin 생략, 분리 표기 정상).

커밋: `f57a7104`(소품) → `aad548d0`(분석·계획) → `e261444a`(추출 1) → `f1995532`(추출 2)
→ 본 보고서.

## 3. 게이트 (매 추출 전수 통과)

fmt ✓ / clippy 0 / `--tests` **2,912·실패 0** / hwp5_roundtrip_baseline 3/3 /
OVR 5샘플 **추가 변동 0** — rowbreak-problem-pages의 기지 3건(#1936발, 시각 판정 대기)과
시그니처 동일 유지, 나머지 4샘플 회귀 0.

## 4. 재평가 (스냅샷 `2026-07-06-r3/`)

- CC>25: 82(r2) → **84** — 신규 추출 함수의 경계 초과분(+2), v2 §5 분할 과도기 허용 범위.
  `parse_object_control_char`(104)는 다음 분해 후보(내부 ch==10 표 블록 861줄).
- **typeset.rs 재성장 공식 확인** (07-05 PR 유입의 영향, 라운드 2 관측의 정량화):
  `typeset_section_with_variant` CC 104 → **117**, `typeset_block_table` 112 → **127**.
  기능 가드 추가에 의한 자연 증가이나 추세 감시 대상 — 4차 리뷰의 재축적 패턴이
  수치로 재현되고 있음.

## 5. 이연/다음 라운드 후보

1. `parse_object_control_char` 내부 분해 (ch==10 표 블록 861줄 — 통추출 상태에서 재평가)
2. `parse_paragraph_list` 후처리 694줄 — `Hwp3FlowState` 설계 선행 (mut 12, 임계 초과)
3. `layout_composed_paragraph` 226 (현 1위) — B 블록 `RunEmitState` (#1925 이연)
4. typeset 재성장 대응 — 신규 가드들의 흡수 재설계 (Phase P Provenance 수렴과 연동)

## 6. 산출물 위치

계획 `plans/task_m100_2001{,_impl}.md` / 단계 보고 `working/task_m100_2001_stage{2,3}.md` /
스냅샷 `mydocs/metrics/2026-07-06-r3/`
