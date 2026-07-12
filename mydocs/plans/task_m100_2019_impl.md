# 구현계획서 — #2019 부동 글상자 다수 별지 서식 과분할·조각 렌더

- 이슈: #2019 / 브랜치: `fix/2019-through-wrap-overlay-vpos`
- 수행계획서: `task_m100_2019.md` (승인됨)
- 활성 엔진: `TypesetEngine`(`typeset.rs`). 근본 수정점: `layout.rs:866-890 para_has_overlay_shape`.

## 설계 확정

`para_has_overlay_shape` 는 "직전 문단이 비-TAC 부동 개체를 소유해 stored vpos가 팽창 → 다음 항목 vpos 보정 base에서 bypass" 를 판정한다. 현재 인정: `InFrontOfText|BehindText`(전 vert_rel) + `TopAndBottom&Para`. **누락된 `Through`(어울림)** 를 추가한다.

- **핵심 근거**: `Through`(text가 개체를 통과)는 `InFrontOfText/BehindText` 와 동일하게 **개체가 인라인 흐름 공간을 차지하지 않는** 부류다. 즉 앵커 문단의 stored vpos는 개체 위치로 인해 팽창하며 다음 항목 base로 신뢰 불가. → InFrontOfText/BehindText 와 동일하게 **vert_rel 무관 overlay 인정**이 설계상 정합.
- **Square/Tight**: text가 개체 옆으로 흐름(측면 점유). Through 와 성격이 달라 회귀영향 불확실 → **Stage3에서 코퍼스 계측 후 포함 여부 결정**(초안: 미포함, Through만).
- **게이트 축소 검토**: Through 를 무조건 overlay로 하면 광범위 영향. Stage3 계측에서 회귀 발생 시, `vert_rel_to ∈ {Paper, Page}` 또는 `앵커 문단 text_len==0` 조건으로 축소하는 fallback 안을 예비.

## Stage 1 — 진단 하네스 + 기준선 캡처

- 산출: `output/poc/task2019/baseline.tsv` — (a) 74312 rhwp 페이지수(81, 결함) + 한글 오라클(18, 정답), (b) 무회귀 표본 페이지수 기준선.
- 무회귀 표본: 8차 서베이 `pipage.tsv` 에서 MATCH 판정 랜덤 60 + MORE 클러스터(글상자/도형 밀집) 20 + #2004 재현(1613000)·#2015 재현 파일 = 총 ~85문서의 **현재(HEAD, 미수정) 페이지수**를 dump-pages로 캡처.
- 74312 서식 페이지 export-png 현재 상태(조각 깨짐) 캡처(before).
- **커밋**: 하네스 스크립트 + baseline.tsv + `task_m100_2019_stage1.md`.
- **소스 미수정** (계측만).

## Stage 2 — Through 확장 + 74312 회귀

- 수정: `layout.rs:866-890 para_has_overlay_shape` 의 Shape/Picture arm 에 `TextWrap::Through` 추가.
  ```
  matches!(cm.text_wrap, InFrontOfText | BehindText | Through)
    || (matches!(cm.text_wrap, TopAndBottom) && vert_rel==Para)
  ```
- 검증(1차): 74312 dump-pages 페이지수 81 → 18(±1) 확인. export-png 서식 격자 정상 렌더 시각 확인(한글 PDF 대조, `scripts/visual_oracle_native.py` 또는 compare 헬퍼).
- 빌드: `cargo build --release --features native-skia`(rebuild 전 `rm target/release/rhwp.exe` — staleness 주의).
- **커밋**: 소스 1파일 + `task_m100_2019_stage2.md`(before/after 페이지수 + 시각 증거).

## Stage 3 — 광범위 무회귀 계측 + 게이트 확정

- Stage1 baseline.tsv 의 ~85문서 페이지수를 수정본으로 재측정 → **불변(Δ=0) 검증**. 변동 발생 문서는 개별 시각 판정(한글 오라클)으로 개선/회귀 판별.
- #2004(1613000 전면이미지 스택), #2015(부동 RowBreak 표) 재현 파일 페이지수 **불변** 확인(vpos 경로 공유 상호작용).
- 회귀 발생 시: 설계의 게이트 축소 fallback(`vert_rel∈{Paper,Page}` 또는 `text_len==0`) 적용 후 재계측.
- Square/Tight 포함 시 개선되는 서베이 문서가 있는지 별도 계측(있으면 포함 검토, 회귀 없을 때만).
- **커밋**: 계측 결과 + `task_m100_2019_stage3.md`.

## Stage 4 — 회귀테스트 + 최종 검증

- 회귀테스트 추가: `tests/issue_2019_through_overlay_pagination.rs` — 74312 축소 케이스(또는 74312 직접) 페이지수 assert(≤20), 무회귀 대표 1~2문서 페이지수 assert.
- 전체 검증: `cargo test`(renderer/document_core lib + baseline `hwpx_roundtrip_baseline` 4/4) 그린.
- roundtrip 무결(save 모델 무손상) 확인.
- **커밋**: 테스트 + `task_m100_2019_report.md`(최종 결과보고서). 오늘할일 갱신은 orders 불가침 규칙상 제외.

## 위험/완화 요약

| 위험 | 완화 |
|------|------|
| Through overlay 확대가 광범위 vpos 경로에 회귀 | Stage3 ~85문서 + #2004/#2015 표적 계측, 변동 시 개별 시각판정 |
| 게이트 과도 확대 | text_len==0 / vert_rel∈{Paper,Page} 축소 fallback 예비 |
| stale 바이너리 오판 | rebuild 전 `rm target/release/rhwp.exe` |
| 저장 경로 영향 | para_has_overlay_shape 는 렌더/페이지네이션 전용, save 무관(roundtrip으로 재확인) |

## 승인 요청

위 4단계로 진행해도 될지 승인 요청합니다. 승인 시 Stage1부터 착수하고 각 단계 완료 후 보고·승인 요청하겠습니다.
