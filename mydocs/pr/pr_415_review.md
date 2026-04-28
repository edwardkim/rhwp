# PR #415 검토 — Task #352 dash 시퀀스 Justify 폭 부풀림 정정

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#415](https://github.com/edwardkim/rhwp/pull/415) |
| 작성자 | [@planet6897](https://github.com/planet6897) (Jaeuk Ryu) — 신뢰 컨트리뷰터 |
| base / head | `devel` (`4828937`) ← `planet6897:local/task352` |
| state | OPEN, BEHIND |
| **CI** | 모두 SUCCESS ✅ |
| 변경 통계 | +6746 / -191, **67 files**, **40 commits** |
| 이슈 | [#352](https://github.com/edwardkim/rhwp/issues/352) (closes #352) |

## 작성자 정황

@planet6897 — 신뢰 컨트리뷰터, 다수 머지 이력 (PR #371, #373, #392 등). CLAUDE.md 절차 엄격 준수.

## 핵심 정황 — 본 PR 은 다른 OPEN PR 들의 변경분을 누적

본 PR head 의 40 commits 분석:

| Task | Commits | 관련 OPEN PR | 본 저장소 devel 머지 여부 |
|------|---------|-------------|---------------------|
| **#352 (본 PR 핵심)** | 7 (Stage 1-5 + WASM + 폭 보정) | **#415** | 진행 중 |
| #412 | 4 (Stage 1-4) | [#414](https://github.com/edwardkim/rhwp/pull/414) | OPEN, base=main |
| #409 (v1, v2, v3) | 9 | [#410](https://github.com/edwardkim/rhwp/pull/410) | OPEN, BEHIND |
| #404 | 3 | [#408](https://github.com/edwardkim/rhwp/pull/408) (Task #402+#404 합산) | OPEN, BEHIND |
| #402 | 3 | #408 | OPEN, BEHIND |
| #398 | 3 | [#401](https://github.com/edwardkim/rhwp/pull/401) | **OPEN — 작성자 재정정 요청 상태 (synam-001 회귀)** |
| 샘플 PDF 갱신 | 1 | (PR #401 의 samples) | |

작성자가 자신의 다른 OPEN PR 들을 base 로 쌓아서 본 PR 작업 — 즉 본 PR 머지 시 PR #401, #406/#408, #410, #412 의 변경분이 모두 함께 들어옴.

## 결정적 발견 — 머지 시 synam-001 회귀 도입 (PR #401 정황)

### Dry-run merge 결과

devel 위에 자동 머지 성공.

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ **1044 passed** (1037 → +7 신규) |
| `cargo test --test svg_snapshot` | ✅ 6/6 |
| `cargo test --test issue_418` | ✅ 1/1 (Task #418 보존) |

### samples/synam-001.hwp 5 페이지 회귀 (PR #401 의 회귀와 동일)

| 항목 | devel | PR #415 cherry-pick (전체 PR) |
|------|-------|------------------------------|
| 전체 페이지 수 | 35 | **37** (+2 회귀) |
| 페이지 5 의 PartialTable pi=69 | rows=0..5 | **rows=0..2** (회귀) |

**PR #401 의 synam-001 회귀가 PR #415 에 그대로 포함**되어 있음. 작업지시자가 PR #401 처리 시 옵션 A (작성자 재정정 요청) 으로 결정한 회귀 정황.

## Task #352 핵심 변경 (본 PR 의 본질)

### 결함 (이슈 #352)

`samples/exam_eng.hwp` 5 페이지 32번 문항 빈칸 라인:
- 29 ASCII 하이픈 dash 시퀀스가 PDF 대비 ~3.5 배 부풀어 후속 텍스트 잘림
- 원인: HY신명조 폰트 메트릭의 ASCII '-' 폭 비정상 (853/1024 em ≈ 0.83 em, 일반 구두점 대비 3 배)
- + Justify Branch A 의 음수 슬랙 처리 한계

### 정정 (5 단계)

| Stage | 내용 |
|-------|------|
| 1 | 원인 확정 — 폰트 메트릭 + Justify 분배 |
| 2 | dash advance 자연 폭 보정 (leader-aware, ≥3 연속 dash 만 0.3 em 으로 강제 좁힘) |
| 3 | dash run 시각 라인 통합 — `<text>` 글리프 → 단일 `<line>` (PDF 와 동일 표현) |
| 4 | 폭 미세 보정 (0.32 em → 0.5 em PDF 실측 반영) |
| 5 | dash leader elastic Justify 분배 — `extra_dash_advance` 신규 필드 + word slack → dash slack 흡수 |
| WASM | text_measurement.rs 의 leader 패치를 WASM 경로에도 적용 |

### Stage 5 의 핵심 — Justify Branch 3-tuple 확장

`(extra_word_sp, extra_char_sp)` → `(extra_word_sp, extra_char_sp, extra_dash_sp)` 로 7 분기 (Branch A/B/Distribute/overflow/cell underflow/...) 의 반환 튜플 변경.

### 검증 (작성자 보고)

| 항목 | v0 (devel) | v4 (Stage 5) | PDF 목표 |
|------|----------|--------------|---------|
| Q32 dash advance | 12.11 px | **7.06 px** | ~7.4 px |
| 29 dash 폭 | 351 px | **204.7 px** | ~218 px |
| dash 글리프 | 29 개 | 0 개 (line 대체) | 0 개 |

p6 Q33 "to be free, then" 13 글자 폭:
- devel: 83.6 px (Branch A min-clamp 압축)
- Stage 3: 125.7 px (공백 +14~24 px 팽창 — 사용자 컴플레인)
- **Stage 5: 95.6 px** (자연 폭, 압축 해제)

cargo test --release: 1023 passed, svg_snapshot 6/6, tab_cross_run 1 passed.

## 평가

### 강점 (Task #352 본 task)

1. **결함 분석 깊이** — 5 단계 진단 (폰트 메트릭 + Justify 분배 + WASM)
2. **PDF 측정 정량 비교** — pdftotext bbox 로 실측 반영
3. **사용자 피드백 반영** — Stage 3 의 공백 팽창 → Stage 5 의 elastic 분배로 정정
4. **회귀 영향 점검** — exam_kor / exam_math / aift / biz_plan 무회귀
5. **5 단계 Stage 보고서** + 트러블슈팅 (`mydocs/troubleshootings/issue_352_root_cause.md`)
6. **Branch 3-tuple 확장** — Justify 7 분기 모두 일관 적용
7. **WASM 경로 동기화** — text_measurement.rs 의 leader 패치를 WASM 측에도 적용
8. **CI 통과** + cargo test 1044 passed
9. **CLAUDE.md 절차 준수** — 수행/구현/Stage/최종 보고서 모두 포함

### 약점 / 점검 필요

#### 1. **본 PR 이 다른 OPEN PR 들의 변경분을 누적** (결정적)

작성자가 `local/task352` 를 자신의 다른 OPEN PR 들 (#401, #406/#408, #410, #414) 위에서 작업. 본 PR 의 40 commits 중 **본 task #352 관련은 7 개** 뿐. 나머지 33 commits 는 다른 task / merge.

본 PR 전체 머지 시 **PR #401 의 synam-001 회귀** (작업지시자가 작성자에게 재정정 요청한 사안) 가 그대로 도입됨.

#### 2. exam_eng.hwp 작업지시자 시각 판정 미수행

본 PR 의 본질은 시각적 (dash leader 폭 보정). 작성자가 측정값으로 정량 검증했지만, **작업지시자 환경의 한컴 출력과 비교 시각 판정** 이 머지 전 게이트.

#### 3. tests/golden_svg/issue-147/aift-page3.svg +108/-108

골든 SVG 갱신. 정합성 점검 — leader 패치가 dash 글리프 → line 대체로 변경했으니 자연스러운 갱신이지만 회귀 여부 작업지시자 확인 필요.

#### 4. extra_dash_advance 도입의 영향 범위

새 TextStyle 필드 추가. 본 PR 외 다른 호출처에서 default 값으로 작동하는지 (특히 cell underflow 분기 등) 점검 필요.

## 메인테이너 작업과의 관계

### 충돌 가능성

본 PR head 가 BEHIND (base sha `4828937`) 이지만 dry-run merge 자동 성공.

다만 본 PR 의 변경분이 다른 OPEN PR 의 변경분과 **중복** — PR #401 (Task #398), PR #406/#408 (Task #402/#404), PR #410 (Task #409), PR #414 (Task #412) 가 이미 OPEN 상태에서 본 PR 도 같은 변경 포함.

## 처리 방향 후보

| 옵션 | 내용 |
|------|------|
| **A** | **Task #352 7 commits 만 cherry-pick** (다른 task 들의 처리 흐름과 분리) |
| B | PR #415 전체 cherry-pick — PR #401 의 synam-001 회귀 그대로 도입 (위험) |
| C | 작성자에게 rebase 요청 — PR #401 등 다른 PR 처리 후 본 PR 재제출 |
| D | 현 상태 OPEN 유지 + PR #401 처리 우선 → PR #415 자연스럽게 충돌 정리 |

### 옵션 분석

**옵션 B**: PR #415 의 본 task (Task #352 dash leader) 는 합리적이고 정량 검증 양호하지만, PR #401 회귀를 함께 도입하므로 위험. **추천 안 함**.

**옵션 A**: 7 commits 만 cherry-pick — 작업지시자 시각 판정 후 머지. 다른 task 들은 자체 PR 흐름대로 처리. cherry-pick 시 충돌 발생 가능 (Task #398 의 변경과 같은 영역 — height_measurer.rs 의 +258 줄이 Task #398 의 변경분 + Task #352 의 leader 변경 합산일 수 있음). 정확히 분리 가능한지 확인 필요.

**옵션 C**: PR #401 등이 모두 처리될 때까지 OPEN 유지. 작성자 부담 큼.

**옵션 D**: 본 PR OPEN 유지 + 다른 PR 들을 먼저 처리. PR #401 정정 후 작성자가 본 PR rebase. 자연스러운 흐름.

### 권장 — 옵션 A 시도 후 충돌 시 옵션 D

이유:
1. Task #352 자체는 합리적이고 정량 검증 양호 — 분리 가능하면 머지 가치 있음
2. 다만 본 PR 의 commit 들이 다른 task 의 변경분과 같은 파일 / 같은 영역 변경 가능성 — 충돌 시 분리 어려움
3. 충돌 시 옵션 D 로 자연스러운 흐름 진행

### 시각 판정 게이트 — push 전 필수

본 PR 의 본질은 **시각적 변경** (dash leader 폭). 작업지시자 환경에서 다음 비교:
- `samples/exam_eng.hwp` 5 페이지 Q32 라인 — 한컴 출력 vs PR #415 적용 후 SVG / web 에디터
- 다른 dash leader 사용 hwp 샘플 무회귀

## 검토 항목 (Claude 점검 완료)

- [x] 작성자 신뢰도 — 신뢰 컨트리뷰터 ✅
- [x] CI 통과 ✅
- [x] dry-run merge — 자동 성공 ✅
- [x] cargo test --lib — 1044 passed ✅
- [x] 본 PR 본질 (Task #352) 코드 품질 — 합리적 ✅
- [x] **synam-001 회귀** — PR #401 회귀 그대로 도입 ⚠️
- [ ] **작업지시자 시각 판정** (exam_eng Q32 + 다른 샘플) — 필수 게이트
- [ ] Task #352 7 commits 만 분리 cherry-pick 가능 여부 — 시도 필요

## 다음 단계 — 작업지시자 결정

A / B / C / D 중 결정 부탁드립니다.

권장: **옵션 A 시도** (Task #352 7 commits 분리 cherry-pick) **→ 충돌 시 옵션 D** (다른 PR 처리 후 재제출).

## 참고

- PR: [#415](https://github.com/edwardkim/rhwp/pull/415)
- 이슈: [#352](https://github.com/edwardkim/rhwp/issues/352)
- 회귀 정황 PR: [#401](https://github.com/edwardkim/rhwp/pull/401) (synam-001 회귀, 작성자 재정정 요청)
- 본 PR 누적 다른 task PR: #406, #408, #410, #414
