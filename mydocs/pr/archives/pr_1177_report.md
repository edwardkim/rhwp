# PR #1177 검토 — 표 + picture 한컴 정합 (Task #1151)

## 1. PR 정보

| 항목 | 내용 |
|------|------|
| 번호 | #1177 |
| 제목 | Task #1151: 표 + picture 한컴 정합 (삽입+토글+시각+클릭, v8/v9 6 결함 fix) |
| 작성자 | [@johndoekim](https://github.com/johndoekim) (johndoe) — 핵심 컨트리뷰터 |
| base ← head | `devel` ← `johndoekim:local/task1151` (cross-repo) |
| 상태 | OPEN, MERGEABLE, **CLEAN** |
| 변경 | +6244 / -323, 55 파일 (코드 18 + 샘플 9 + 문서 28) |
| 라벨 | enhancement / v1.0.0 |
| 연결 | Closes #1151 |
| CI | 전부 pass (Build&Test, Analyze rust/js/py, Canvas visual diff, CodeQL) |

## 2. 컨트리뷰터 이력 + 재제출 맥락

@johndoekim: #1150(#1138 머지), #859/#693 등. 표/셀/click hit-test 영역 핵심 기여자.

**#1151 3차 제출** (#1160 → #1173 → #1177):
- #1160/#1173 은 메인테이너 거절이 아니라 **컨트리뷰터 자발적 close** — #1173: "최종 시각 테스트 후 결함 발생으로 결함 조치 후 재PR".
- #1177 은 그 결함(v8/v9 6개)을 조치한 재제출.
- 작업지시자가 이전 세션에서 "#1151 은 컨트리뷰터가 해결" 결정 → 의도된 외부 위임.

## 3. 변경 내용 (4 측면 + v5~v9 후속)

| Phase | 내용 |
|------|------|
| v1 | 셀 안 picture 신규 삽입 — 한컴 패턴 (표 sibling floating, tac=false, wrap=Square, Page-relative) |
| v2 | floating↔inline tac 토글 model 정합 (한컴 Scenario A~D dump 4 필드 갱신 확정) |
| v3 | paragraph 안 sibling TopAndBottom 표 + tac picture 시각 layout (y 보정) |
| v4 | 셀 안 inline picture click hit-test + dialog (ImageNode 3 필드 + 8 caller + by_path API) |
| v5 | tac toggle 시 invalidate_page_tree_cache 누락 (다른 6 setter 와 일관성) |
| v6 | Table::update_ctrl_dimensions 의 common.width/height 미동기화 |
| v7 | audit 기반 helper 4 추출 (행위 무변경) |
| v8/v9 | 한컴 native 시연 비교 6 결함 (dialog 가로/세로 표기, picture wrap 분배 등) |

## 4. 코드 검토 (핵심 점검)

- **5-layer fault (v4)**: ImageNode struct + 8 caller — 동시 갱신 필요 영역. `feedback_image_renderer_paths_separate` 정신과 정합 (struct 변경 시 전 caller 점검).
- **v5 invalidate_page_tree_cache 누락**: 다른 setter 와 일관성 정정 — #1144/#1172 와 같은 패턴(cache invalidation 일관성).
- **v6 common 동기화**: raw_ctrl_data ↔ self.common 이중 표현 동기화 — 표 영역 reserved 계산이 stale 값 쓰던 결함.
- 한컴 정합 근거: Scenario A~D dump 분석 문서(`hancom_picture_tac_toggle.md`) + 양방향 산출물 검증.

## 5. 검증

| 항목 | 결과 |
|------|------|
| auto-merge | ✅ CLEAN (충돌 0) |
| CI 전부 | ✅ pass |
| PR 자체 | cargo test --lib 1454 passed, clippy clean, fmt (PR 기재) |
| 신규 테스트 | v1~v9 19개+, 회귀 0 |
| 메인테이너 빌드+test | (확인 중) |
| 시각 | 한컴 2022 양방향 (Windows 편집기) — PR 기재, 9 시나리오 |

## 6. 우려/확인

- **규모 +6244/-323 55파일** — 큰 PR. 그러나 단일 이슈(#1151) 의 다측면(4+후속) 이 응집돼 있고, helper 추출(v7)로 정리됨. 11 stage 보고서로 추적 가능. 작은 단위 회전 정책과 긴장은 있으나, 컨트리뷰터가 phase 별로 잘게 진행 후 통합 제출한 형태.
- **후속 #1171** (이중 nested glyphbox picture click) 별도 분리 — 적절.

## 7. 판단 (잠정)

다측면 한컴 정합 + 19 테스트 + helper 정리 + CI pass + auto-merge clean + 양방향 시각 검증. 컨트리뷰터 자발적 품질 관리(#1173 자진 close 후 결함 조치). **merge 권장**.
- 처리: CLEAN → devel 직접 머지.
- 표+picture 는 시각 정밀 영역 — 작업지시자 시각 판정 권장 (단, PR 이 한컴 양방향 검증 자료 첨부).

> 승인 시: 메인테이너 빌드+test 검증 → merge → `pr_1177_report.md`.

---

## 8. 처리 결과 (보고)

- **MERGED**: devel `84a09e8c` (이슈 #1151 close)
- auto-merge CLEAN, CI pass, cargo test --tests 97 스위트 전부 통과 + fmt
- PR 코멘트 등록 + 이슈 #1151 close
- 시각 판정: 작업지시자 직접 진행 (PR 한컴 2022 양방향 자료 첨부)
- 후속 #1171 별도 분리
