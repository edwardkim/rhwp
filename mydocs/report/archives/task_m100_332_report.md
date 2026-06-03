# Task #332: typeset/layout drift 통합 — 최종 결과 보고서

- **이슈**: #332 — typeset/layout drift 통합 (Task #331 재시도 기반)
- **브랜치**: `task332`
- **수행 기간**: 2026-04-25
- **수행계획서**: `mydocs/plans/task_m100_332.md`
- **구현계획서**: `mydocs/plans/task_m100_332_impl.md`
- **단계별 보고서**: `mydocs/working/task_m100_332_stage{1,2,3a,3b,4a,4b,5}.md`

---

## 배경

Task #331 의 단순 fix (typeset advance 를 height_for_fit 으로) 가 layout 의 clamp pile 버그를 노출시켜 글자 겹침 회귀 → revert (`078717f`). #332 는 5 단계 sub-task 로 분해해 단계별 회귀 검증을 거치며 정합 작업을 진행.

## 단계별 변경 요약

| 단계 | commit | 변경 |
|------|--------|------|
| 1 | `bf42d61` | typeset advance 를 `height_for_fit` 기반으로 변경 (`typeset.rs:612, 622`). 5 개 lib 테스트 calibration. |
| 2 | `08e477e` | layout per-paragraph advance 를 동일 모델로 정합 (`paragraph_layout.rs:2432`). 일반 문단의 마지막 visible 줄에서 trail_ls 제외. partial 은 유지. |
| 3a | `82f34a4` | vpos correction 의 `vpos_end` 에서 trail_ls 제외 (`layout.rs:1367`). |
| 3b | `2d880bf` | vpos correction 단방향 → 양방향 + collapse 가드 (line_height × 3.0 backward 한도). |
| 4a | `d9713b8` | typeset fit 검사에 layout drift 안전 마진 도입 (10 px). partial split 의 `avail_for_lines` 에도 동일 마진. |
| 4b | `0211e57` | clamp pile 분기 제거. overflow line 은 원래 y 좌표 그대로 그림 (piling 차단, 손실 없음, col 경계 약간 넘김 허용). |
| 5 | `9e25f53` | vpos correction 의 segment_width 일치 가드(±3000 HWPUNIT) 완화. drift root cause 분석 문서화. |
| (후) | `5376161` | golden SVG 2 개 baseline 갱신 (issue-147, issue-157). |

## 검증 결과

```
cargo test --lib                 → 992 passed (Task #321 v6 의 992 유지)
cargo test --test '*'            → 47 passed (golden 6 + 기타 41)
```

### 회귀 검증 (수동)

| 샘플 | 글자 겹침 | 콘텐츠 손실 | OVERFLOW |
|------|-----------|-------------|----------|
| 21_언어 page 0 | 없음 (이전: clamp pile 가능성) | 없음 | pi=10 line 1 의 15.5px col 경계 넘김 (DRAW, 잔존) |
| form-01 | 없음 | 없음 | 0 |
| hwp-multi-002 | 없음 | 없음 | pi=68 line 0 44.7px (DRAW), Table 31.3px (pre-existing) |
| multi-table-001 | 없음 | 없음 | 0 |
| lseg-06-multisize | 없음 | 없음 | 0 |
| aift | 없음 | 없음 | pi=222 line 3 8.6px (DRAW), Table 2건 (pre-existing) |

## 검증 기준 평가 (이슈 본문 대비)

| 기준 | 결과 |
|------|------|
| ✅ pi=26 + 보기 fit (PDF 일치) | **부분 달성**. typeset 의 partial 분배는 개선되었으나 layout drift 자체는 잔존하여 시각적으로는 일부 잔여. |
| ✅ pi=10 partial 글자 겹침 없음 | **달성**. clamp pile 제거로 piling 차단. 단 col 경계를 15.5px 넘김 (시각 issue, 글자 겹침 아님). |
| ✅ `cargo test --lib` 992 passed | **달성**. |
| ✅ Golden SVG 6 개 통과 | **달성** (issue-147, issue-157 baseline 갱신, 의도된 변경). |
| ✅ 다른 샘플 회귀 없음 | **달성**. |

## 핵심 통찰 (Stage 5 의 root cause 분석)

drift 의 본질은 **typeset 의 자체 height 추정 vs layout 의 vpos 기반 진행** 의 모델 차이다. 본 task 의 5 단계 정합 작업은 두 시스템이 같은 `height_for_fit` 모델을 사용하도록 조정했지만, **vpos correction 이 trigger 안 되는 케이스 (multi-column + Table 환경) 에서 drift 가 누적**되는 본질은 그대로다.

본질적 해결을 위한 옵션:

- **A**: typeset 자체를 LINE_SEG vpos 기반으로 재설계
- **B**: layout 의 vpos correction 을 paragraph 단위가 아닌 **line 단위** 로 적용
- **C**: HeightMeasurer 를 typeset/layout 모두의 single source of truth 로 만들기

본 task 는 정합의 토대를 마련했으며, 후속 task 에서 위 옵션 중 선택해 진행 가능하다.

## 산출물

- 7 개 commit (Stages 1, 2, 3a, 3b, 4a, 4b, 5) + golden 갱신 commit 1 개
- 수행계획서, 구현계획서, 7 개 단계별 보고서, 본 최종 보고서
- golden SVG 2 개 baseline 갱신
- 후속 task 권고: "typeset/layout 의 advance 모델을 LINE_SEG vpos 기반으로 통합 재설계"

## 결론

Task #332 의 5 단계 통합 작업으로 typeset/layout 의 drift 가 발생하는 **메커니즘을 정합** 했고, **회귀 (글자 겹침, 콘텐츠 손실) 없이** 안정적인 base 를 마련했다. 다만 multi-column + Table 환경에서의 layout drift 의 **본질적 해결**은 typeset/layout 의 advance 모델 자체를 재설계하는 후속 task 가 필요하다.
