---
kind: investigation
status: active
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-10
---

# Task #3820 Stage 108 — 정책연구 p131→p132 stale 원장 정정

## 범위와 시작 상태

- 브랜치: `task/3820-production-fidelity`
- 시작 commit: `e9253d21d`
- 입력: `samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp`
- 한컴 2020 기준 PDF:
  `pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf`
- 선행 기록: `mydocs/working/task_m100_3820_stage103_policy_body_footnote_fragments.md`
- 코드 변경: 없음. 이 stage는 p131→p132 후보의 stale 근거와 문서 내부 모순만
  바로잡고, 현행 renderer 계약은 유지한다.

## 현재 형상과 PDF 정답

정책연구 문서의 1-based p131→p132 각주 owner 정답은 다음과 같다.

| 페이지 | 한컴 PDF | 현재 rhwp 계약 |
| --- | --- | --- |
| p131 | 각주 179만 소유 | 각주 179만 소유하고 각주 180은 소유하지 않음 |
| p132 | 각주 180 전체와 각주 181 소유 | 각주 180 전체와 각주 181 소유 |

각주 180은 p131에서 시작해 p132로 이어지는 continuation이 아니다. 각주 자체의 저장
`LINE_SEG`는 2줄 `(textpos, vpos) = (0,0), (72,1172)`이며 내부 page reset이 없다.
marker가 있는 본문 `pi=1382`의 다음 저장 body line에서 page reset이 시작되고 p131에는
이미 각주 179가 있으므로, 각주 180 전체를 p132의 footnote lane으로 보내는 것이 PDF
정답이다. p132의 각주 180에는 `대로 호적 등으로`를 포함한 본문 전체가 있고 그 뒤에
각주 181이 배치된다.

현행 실물 회귀
`native_hwp5_body_footnotes_follow_the_p129_and_p131_reset_pages`도 이 계약을 직접
단언한다. p131은 `179)`를 포함하고 `180)`을 포함하지 않아야 하며, p132는 `180)`,
`181)`, `대로 호적 등으로`를 모두 포함해야 한다. 문서 전체 쪽수 계약은 215쪽이다.

## stale 원인

Stage 103의 초기 재현 문단은 각주 180도 p131에서 시작해 p132로 이어지는 분할
각주라고 기록했고, 이어지는 페이지별 bullet도 같은 초기 가설을 남겼다. 그러나 같은
문서의 source 감사와 최종 결과는 이미 다음 사실을 올바르게 기록한다.

- 각주 180에는 내부 stored reset이 없다.
- p131은 각주 179만 소유한다.
- p132는 각주 180 전체와 각주 181을 소유한다.

따라서 p131→p132 후보는 최신 renderer가 만든 새로운 owner 회귀가 아니라, Stage 103
초기 판정 서술이 최종 판정으로 교체되지 않아 남은 **stale 문서/원장 후보**다. 최신
source의 코드 주석과 실물 회귀는 모두 whole-note-to-tail-page 계약을 가리키므로,
검증에서 이 형상이 유지되는 한 추가 pagination 보정이나 baseline 변경을 하지 않는다.

## 완료 기준

1. Stage 103의 초기 판정 문단에서 잘못된 “각주 180 분할” 서술을 제거한다.
2. 최신 committed source로 p131에는 각주 179가 정확히 한 번 있고 각주 180이 없음을
   확인한다.
3. p132에는 각주 180의 번호와 전체 본문, 각주 181이 각각 정확히 한 번 있고 순서가
   PDF와 일치함을 확인한다.
4. p131→p132 본문 `pi=1382` owner와 전체 215쪽 계약을 유지한다.
5. 한컴 PDF raster 직접 대조와 focused 실물 회귀가 모두 통과하면 stale 후보를
   종결한다. 하나라도 다르면 문서 정정만으로 종결하지 않고 별도 코드 원인 분석으로
   전환한다.

## 검증 계획

1. 시작 commit `e9253d21d`의 검증 binary로 정책연구 p131·p132만 SVG/PNG로 다시
   산출한다.
2. 같은 1-based 페이지의 한컴 2020 PDF raster와 페이지별로 직접 비교해 footnote
   separator, 번호, 본문 시작·끝, 179→180→181 순서를 판정한다.
3. render tree에서 p131/p132의 FootnoteArea 텍스트를 추출해 179/180/181의 소유와
   중복·소실 여부를 기계적으로 확인한다.
4. 다음 focused 실물 회귀를 `CARGO_TARGET_DIR=target/pr-review`,
   `CARGO_INCREMENTAL=0`, `release-test`로 실행한다.

   ```bash
   cargo test --profile release-test \
     --test issue_3738_rowbreak_table_footnote_fragment \
     native_hwp5_body_footnotes_follow_the_p129_and_p131_reset_pages -- --exact
   ```

5. 이 stage는 코드 변경이 없으므로 focused 판정이 맞으면 전체 회귀는 최종 PR gate의
   기존 순차 실행 결과를 공유하고, stale 후보 때문에 별도 baseline을 갱신하지 않는다.

## 회귀 보강

기존 실물 회귀는 `pi=1382` line index를 정렬한 뒤 `dedup()`하여, 같은 line이 중복
paint되는 회귀를 숨길 수 있었다. 다음 계약을 p129–p133 5쪽 창으로 강화했다.

- `pi=1372` owner: p129 `0..6`, p130 `6..9`, p131–p133 없음
- `pi=1382` owner: p129–p130 없음, p131 `[0,1]`, p132 `[2]`, p133 없음
- `dedup()` 없이 raw line index 배열을 비교해 중복 paint도 실패 처리
- `pi=1377`, `pi=1379` 표는 p131에 각각 정확히 하나만 존재하고, 표→표→본문
  순서와 각주 separator 비침범을 유지
- 각주 179/180/181의 번호는 기대 페이지에 정확히 한 번만 존재하고, 고유 본문
  `KAKENHI-PROJECT-24593293`, `본인 확인뿐만 아니라`/`대로 호적 등으로`,
  `hishinzoku.pdf`도 다른 페이지에 marker 없이 유출되지 않음
- p131·p132의 `pi=1382` 본문 bottom은 separator 위, FootnoteArea bottom은 footer
  위에 있어야 함

## 최신 원장과 시각 판정

commit `e9253d21d811819b5a6ed997f1082f4c18c98667`에서 SHA-256
`c88b9d91254920dad1ff28805219b4540c76770110e33bcc8422eec7202e72dd`인
`target/pr-review/release-test/rhwp`로 p131·p132를 다시 산출했다.

- 요청/완료 2/2, 누락 0, run state `complete`
- SVG/render tree 215/215, 기준 PDF 215쪽
- 자동 visual flag 0건
- text-only 대조: p131·p132 모두 reference-only/SVG-only 0/0
- owner-shift, owner-sequence, page-boundary, visible-text-excess 후보 0건
- 평균 pixel match 89.928%, 최저 89.213%. 글꼴 raster 차이에 민감한 ink proxy는
  합격 기준으로 쓰지 않고 3-way review를 확대해 직접 판정했다.
- 직접 판정: p131에는 각주 179만 있고, p132에는 각주 180의 처음부터 끝까지와
  각주 181이 있다. 두 페이지 모두 표·본문·각주·footer가 겹치지 않아 한컴 PDF와
  owner 및 페이지 경계가 일치한다.

증적:

- [p131 3-way review](../pr/assets/task_m100_3820_stage108_policy_p131_p132_stale_ledger/review_p131_final.png)
- [p132 3-way review](../pr/assets/task_m100_3820_stage108_policy_p131_p132_stale_ledger/review_p132_final.png)
- [visual sweep manifest](../pr/assets/task_m100_3820_stage108_policy_p131_p132_stale_ledger/visual_sweep_manifest.json)
- [visual sweep summary](../pr/assets/task_m100_3820_stage108_policy_p131_p132_stale_ledger/visual_sweep_summary.json)
- [overlay metrics](../pr/assets/task_m100_3820_stage108_policy_p131_p132_stale_ledger/overlay_metrics.json)
- [page-count ledger](../pr/assets/task_m100_3820_stage108_policy_p131_p132_stale_ledger/page-count-ledger.tsv)
- [text report](../pr/assets/task_m100_3820_stage108_policy_p131_p132_stale_ledger/text-report.tsv)
- [page-boundary candidates](../pr/assets/task_m100_3820_stage108_policy_p131_p132_stale_ledger/page-boundary-fidelity-candidates.tsv)
- [provenance](../pr/assets/task_m100_3820_stage108_policy_p131_p132_stale_ledger/provenance.tsv)

## 검증 결과

`CARGO_TARGET_DIR=target/pr-review`, `CARGO_INCREMENTAL=0`, `release-test` 기준:

- `issue_2430_cell_rewrap_threshold`: **2/2 passed**
- 강화한 p129–p133 exact 회귀: **1/1 passed**
- `issue_3738_rowbreak_table_footnote_fragment`: **33/33 passed**
- 변경한 실물 회귀 파일의 `rustfmt --check`, `git diff --check`: **passed**

이 stage는 renderer source를 변경하지 않았다. 최신 source 전체
`cargo test --profile release-test --tests`, Clippy, Native Skia는 바로 앞 Stage 107에서
각각 exit 0으로 확정됐고, Stage 108에서는 문서·증적 및 실물 회귀 단언만 강화했다.

## 판정

Stage 98의 p131→p132 96자 owner/28자 sequence/body-footnote 후보는 Stage 103에서
이미 해소됐다. Stage 103 초기 가설 문구가 최종 판정으로 교체되지 않았고 오래된
원장을 다시 순위화하면서 미해결처럼 보인 것이다. 최신 text 원장, p129–p133 실물
회귀, 한컴 PDF 직접 대조가 모두 같은 owner를 확인했으므로 추가 renderer 보정 없이
**stale 후보로 폐기**한다.
