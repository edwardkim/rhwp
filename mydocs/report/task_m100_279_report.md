# Task #279 최종 결과 보고서

> **원본 분석·구현**: [@seanshin](https://github.com/seanshin) (Shin hyoun mouk) — PR [#282](https://github.com/edwardkim/rhwp/pull/282)
> **인수·마무리**: 메인테이너 (edwardkim)

## 이슈

[#279](https://github.com/edwardkim/rhwp/issues/279) — 목차 right tab 리더 점 렌더링 + 소제목 페이지 번호 정렬 불일치

## 결론

✅ **해결 완료** — KTX.hwp 목차 페이지의 모든 결함 (리더 점 모양, 장제목·소제목 정렬, 페이지번호 폭별 leader 길이, 셀 padding 인지) 해결. 모든 페이지번호 right edge 가 동일 위치에 정렬되며 한 자리/두 자리 페이지번호 모두 leader 와 일정 gap 유지.

## 인수 경위

본 task 는 외부 기여자 [@seanshin](https://github.com/seanshin) 의 PR #282 로 시작 (2026-04-24). devel 충돌 + #279 범위 외 파일 다수 포함 + 24h+ 응답 부재로 메인테이너가 직접 인수 (작업지시자 결정, 2026-04-25). 작성자 핵심 3 커밋 author 정보를 보존한 채 `local/task279` (origin/devel 기준) 에 cherry-pick + 메인테이너 추가 보강.

## 처리 절차

1. ✅ Stage 0: 트러블슈팅 사전 검색 + 이슈 assignee 지정 + 인수 안내 코멘트
2. ✅ Stage 1: 작성자 핵심 3 커밋 cherry-pick (author=hyoun mouk shin 보존) + 메인테이너 강화 수행계획서·구현계획서
3. ✅ Stage 2: 빌드/테스트/clippy/wasm32 검증 + svg_snapshot 골든 갱신
4. ✅ Stage 3: 6가지 추가 결함 식별 + 보강 + 트러블슈팅 등록
5. ✅ Stage 4: 최종 보고서 + 기여 인정 산출물 + force-push + admin merge

## 근본 원인 (8가지 결함, 작성자·메인테이너 합산)

### 작성자 [@seanshin](https://github.com/seanshin) 분석 (2가지)

1. **리더 도트 사각 대시**: `dasharray="1 2" stroke-width="0.5"` → 한컴은 원형 점
2. **`find_next_tab_stop` 일률 클램핑**: RIGHT 탭 (`tab_type=1`) 도 `available_width` 로 잘림 → 들여쓰기 문단 정렬 어긋남

### 메인테이너 추가 식별 (6가지)

3. **trailing 공백 \t 케이스 누락**: `run.text.ends_with('\t')` 가드가 `\t ` 형태 (한컴 목차 소제목) 를 놓침 → cross-run RIGHT 진입 자체 안 됨
4. **리더 시멘틱 부재**: 한컴은 리더 (`fill_type ≠ 0`) RIGHT 탭을 "**inner content 우측 끝까지**" 의미로 재해석. rhwp 도 동일 방어 로직 필요 (셀 padding 침범 해소)
5. **리더 길이가 페이지번호 폭 무시**: 한 자리/두 자리 페이지번호 무관 leader 끝이 같은 x → 페이지번호와 겹침
6. **공백 only run 정렬 부적합**: 장제목 케이스 (`"...\t" + " " + "3"`) 에서 ` ` 단독 run 에 RIGHT 정렬 적용 → 페이지번호 +공백폭 우측으로 밀림
7. **leader-bearing TextRun 검색**: leader.end_x 단축 시 직전 = 공백 run (leader 없음) → 한 단계 위 \t 가진 진짜 leader run 을 찾아야 함
8. **선행 공백 시각 보정**: 장제목 두 자리 케이스 (`" 16"` 한 run) 에서 trim_start 후 폭으로 정렬하면 draw_text 가 공백 포함 텍스트를 그려 페이지번호가 +공백폭 우측 출력 → 전체 run 폭 사용

## 변경 내역

### 코드 (3 파일)

| 파일 | 변경 | 내용 |
|------|------|------|
| `src/renderer/svg.rs` | +2/-2 | (작성자) `fill_type=3` round cap 원형 점 |
| `src/renderer/web_canvas.rs` | +6/-1 | (작성자) `set_line_cap("round") + dash=[0.1, 3.0]` |
| `src/renderer/layout/text_measurement.rs` | +4/-2 | (작성자) `tab_type != 1` 클램핑 가드 |
| `src/renderer/layout/paragraph_layout.rs` | +60 / -10 | (메인테이너) trailing 공백 가드 + leader 시멘틱 + 페이지번호 폭별 leader 단축 + 공백 only carry-over + leader-bearing 검색 + trim 제거 |
| `src/renderer/layout/tests.rs` | +4/-4 | (메인테이너) `resolve_last_tab_pending` 시그니처 변경 (3-tuple) 반영 |

### 골든 svg_snapshot (의도된 갱신)

- `tests/golden_svg/issue-267/ktx-toc-page.svg` (KTX 목차 — 본 task 핵심)
- `tests/golden_svg/issue-147/aift-page3.svg` (aift 표 안 leader — dasharray 통일)

### 문서

- `mydocs/plans/task_m100_279.md` — 수행계획서 (메인테이너 강화)
- `mydocs/plans/task_m100_279_impl.md` — 구현계획서 (신규)
- `mydocs/working/task_m100_279_stage{1,2,3}.md` — 단계 보고서
- `mydocs/working/task_m100_279_stage3.md` (작성자 cherry-pick) — 인용 보존
- `mydocs/report/task_m100_279_report.md` (이 문서)
- `mydocs/troubleshootings/toc_leader_right_tab_alignment.md` (재발 방지)
- `CHANGELOG.md` — 항목 추가
- `mydocs/orders/20260425.md` — Task #279 섹션 추가

## 검증 결과

### 단위/통합 테스트

| 항목 | 결과 |
|------|------|
| `cargo build --release` | ✅ |
| `cargo test --lib` | ✅ **992 passed / 0 failed / 1 ignored** |
| `cargo test --test svg_snapshot` | ✅ 6 passed (UPDATE_GOLDEN 후) |
| `cargo test --test issue_301` | ✅ z-table 가드 |
| `cargo clippy --lib -- -D warnings` | ✅ clean |
| `cargo check --target wasm32` | ✅ clean |

### KTX 목차 좌표 변화

| 항목 | Devel (Before) | After |
|------|---------------|-------|
| 장제목 페이지번호 ("3") x | 709.76 | **690.76** |
| 소제목 한 자리 ("4") x | 689.88 | **690.76** |
| 소제목 두 자리 ("14") x | 681.43 | **680.76** |
| 장제목 두 자리 ("16") 첫글자 x | 690.09 | **681.09** |
| 한 자리 leader x2 | 712.80 | **686.09** |
| 두 자리 leader x2 | 712.80 | **676.09** |

→ 모든 페이지번호 right edge ≈ 700.0 으로 정렬 통일.

### 7 샘플 회귀 (페이지 수 무변화)

| 샘플 | 결과 |
|------|------|
| 21_언어_기출_편집가능본.hwp | 15 |
| exam_math.hwp | 20 |
| exam_kor.hwp | 24 |
| exam_eng.hwp | 9 |
| basic/KTX.hwp | 1 |
| aift.hwp | 74 |
| biz_plan.hwp | 6 |

모두 무변화 ✅.

### WASM 시각 검증

- WASM Docker 빌드: 17:25 갱신
- 작업지시자 직접 시각 검증: ✅ 통과 (KTX 목차 모든 정렬 한컴과 동등)

## 기여 인정 (7가지 산출물)

본 task 의 핵심 분석 (리더 도트 dasharray + right tab 클램핑 제외) 은 [@seanshin](https://github.com/seanshin) 의 분석. 메인테이너는 인수 후 6가지 추가 결함 식별·보강 + 워크플로우 문서화. 다음 7가지 방식으로 본인 기여 인정:

1. **Cherry-pick author 보존**: 3 커밋 (`5d1c80f` / `d48af5c` / `76436df`) → `f27477e` / `2eb1be5` / `4770a8a` author=hyoun mouk shin
2. **Co-Authored-By 체인**: 메인테이너 신규 4 커밋 모두 `Co-Authored-By: hyoun mouk shin` trailer 포함
3. **CHANGELOG.md 항목**: "분석·구현 by [@seanshin](https://github.com/seanshin)" 명시
4. **위키 페이지 등재**: `rhwp.wiki/HWP-Tab-Leader-Rendering.md` 신규 — 본인 크레딧 머리말
5. **HWP Spec Errata entry**: `mydocs/tech/hwp_spec_errata.md` 에 추가
6. **PR / 이슈 close 코멘트**: 분석·구현 본인 명기 + 감사
7. **본 보고서 머리말**: "원본 분석·구현: @seanshin — PR #282" 명시 + 작성자 stage3 보고서 인용 보존

## 학습

### 1. HWP 스펙 ≠ 한컴 조판 알고리즘

HWP 스펙은 데이터 포맷 정의일 뿐, 한컴이 그 값으로 어떻게 그리는지의 알고리즘은 비공개. 스펙대로 처리하면 한컴과 다르게 보인다. **rhwp 는 한컴 결과를 정답으로 삼는 자체 조판 엔진** 이어야 한다. 리더 도트의 시멘틱 ("이 줄 우측 끝까지 채움") 같은 한컴 의도를 자체 방어 로직으로 재해석해야 한다.

### 2. run 분할 패턴은 다양하다

같은 paragraph 라도 CharShape 변화에 따라 6 runs (장제목, 폰트 다양) 또는 3 runs (소제목, 단일 폰트) 등으로 다양하게 분할. cross-run 처리 로직은 다음 모든 패턴에 일관 동작해야 한다:
- `"...\t"` (마지막 \t)
- `"...\t "` (\t + trailing 공백)
- `" "` (공백 only run)
- `" 16"` (공백 + 두 자리 페이지번호)
- `"4"` (단일 한 자리 페이지번호)

### 3. 정렬은 "시각 right edge" 기준

right tab 정렬은 단순 좌표 적용이 아니라 **시각 출력 결과의 right edge 가 의도 위치에 오도록** 해야 한다. run 시작 x 와 draw_text 의 첫 글자 출력 x 가 다를 수 있고, 페이지번호 폭에 따라 leader 끝도 달라져야 하며, 셀 padding_right 영역 침범 여부도 검사해야 한다.

### 4. 작업지시자 시각 검증의 가치

매 단계 작업지시자가 시각 결과를 확인 → 다음 결함 식별 → 추가 보강의 사이클로 6가지 추가 결함을 단계별 발견. 자동 검증 (svg_snapshot) 만으로는 식별 불가능했던 시각적 가독성 문제 (leader 와 페이지번호 gap 일관성, 두 자리 정렬 미세 어긋남 등) 를 시각 검증으로 즉시 포착.

### 5. 외부 기여 인수의 모범

작성자 fork 에 force-push 가능 (`maintainerCanModify=true`) 한 점을 활용해 PR 자체를 정리하고 머지. 작성자 commit 의 author 정보 보존 + Co-Authored-By 체인 + 7가지 산출물로 본인 기여 인정. 외부 기여를 close 후 새 PR 로 가는 것보다 이 방식이 깔끔하고 기여 의식 존중에 더 적합.

## 관련

- 이슈: [#279](https://github.com/edwardkim/rhwp/issues/279)
- PR: [#282](https://github.com/edwardkim/rhwp/pull/282)
- 트러블슈팅: `mydocs/troubleshootings/toc_leader_right_tab_alignment.md`
- 관련 트러블슈팅: `hwpx_lineseg_reflow_trap.md`, `line_spacing_lineseg_sync.md`
- 관련 작업: Task #267, #274 (목차 right tab 선행 작업), Task #290/#296 (cross-run 탭 처리)
