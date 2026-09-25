---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-26
---

# PR #7406 리뷰 — KoPub 라틴 전진폭

## 최종 판정

**머지 보류.** 메인터너 보정으로 PrEP 본문/표 겹침을 15건에서 1건으로 줄였고 67~70쪽의 국소 Visual Sweep은 통과했다. 추가 OLE 차트 보정 뒤 남성 차트가 있는 26쪽은 94.19%, 분리한 HWP/HWPX 샘플은 각각 90.67%/90.58%로 통과했다. 여성 차트가 있는 27쪽 전체는 66.83%로 보류 상태이며, 새 head의 전체 회귀·fresh WASM·전체 페이지 시각 검증도 남았다. 현재 원 PR을 승인하거나 통합하지 않는다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR·작성자·base | [#7406](https://github.com/edwardkim/rhwp/pull/7406), `planet6897`, `devel` |
| 원 head / `-x` cherry-pick | `f49ebdefb9254d153e20a8cfd1424be398290eb4` / `c611aa5aff6e72ac02df722e07d4bda0d2b8b6e1` |
| 검토 base / branch | `c80a8370ab294259557c850c2e54495ecd0e79c0` / `review/planet6897-7406-20260925` |
| 관련 이슈 | [#7390](https://github.com/edwardkim/rhwp/issues/7390); 이 검토로 닫지 않음 |
| 원격 참고값 | 2026-09-25 조회: OPEN, Draft 아님, `MERGEABLE/CLEAN`, 원 head의 CI 성공. 머지 전 재조회 필요 |
| 메인터너 보정 | KoPub 줄 구성·각주·표 분할과 겹침 상한 15→1 보정 진행. OLE 차트의 OOXML 값축·백분율 레이블·범주·미리보기 색상표 보정. 검증 및 단계별 커밋 진행 중 |

## 변경과 조판 원칙 검토

원 PR은 KoPub Dotum/Batang의 Basic Latin hmtx 폭을 `text_measurement.rs`에서 사용하고, 집중 검사와 p94 Native 이미지를 추가했다. 변경 폭은 줄 나눔과 표 안 텍스트 및 선행 본문 위치에 전파된다. `tests/fixtures/text_overlap_baseline.tsv`의 PrEP 상한도 14→15로 올렸다.

최초 검토에서 section 2 para 27의 비-TAC TopAndBottom RowBreak 표가 선행 본문·캡션 위로 올라오는 **미충족**을 확인했다. 메인터너 보정에서는 행 컷·예약 높이·페인트 점유를 함께 수정했고, 67~70쪽 국소 게이트와 정식 회귀가 통과했다. 전체 문서의 남은 차이는 계속 검토한다. 호출 경로·전후 좌표와 명령은 Git 제외 `output/pr-review/planet6897-7406-20260925/result.md`에 기록한다.

OLE 차트는 HWPX의 `Chart/chart*.xml`과 HWP의 중첩 `OOXMLChartContents`가 편집 가능한 값·레이블을 담고, 중첩 EMF 미리보기가 문서 전용 색을 담는다. `colorIndex=-1`인 누적 막대에서 색만 미리보기로 복원하고 축·막대·레이블은 OOXML 데이터로 렌더한다. `samples/issue7406`의 값을 바꿔도 낡은 미리보기 막대를 재사용하지 않는 집중 검사와 한컴 PDF 비교를 수행했다. HWPX 구조 근거는 `mydocs/tech/hwp_ole_spec.md` 및 사용자가 제공한 `hwpx_complete_guide.pdf`의 4.2절이다.

## 검증 입력과 결과

| 항목 | 결과 |
| --- | --- |
| 원 HWPX | `samples/issue2006/1790387_prep_final_report.hwpx`, SHA-256 `c68baed24096386f9041930d24d39409b61ac99463bf04dfd242440dfdeb739f`, `hancom-office-2024` 저장 메타데이터, 140쪽 |
| 신규 한컴 PDF | [1790387_prep_final_report-2024.pdf](../../pdf/issue2006/1790387_prep_final_report-2024.pdf), 원본을 HWP MCP `--engine 2024`로 변환, SHA-256 `04b95a6e41420fb45934ce2ee5abd8cf6dac4ce12fd47977dacbe7fca28018a8`, 140쪽, Hancom PDF 1.3.0.550 |
| KoPub 집중 검사 | 검토 source head `regression_suite_016` 2/2 PASS |
| 겹침 baseline | base 14건, 원 PR 단독 15건, 메인터너 보정 후보 1건. 상한은 15→1로 축소했고 검사 PASS. 최종 head 재검증 필요 |
| Native Visual Sweep | 67/68/69/70/94/108쪽 국소 실루엣 **99.94489/96.51123/99.94315/99.97650/99.55839/96.98045%** 통과. 차트 26쪽 **94.19%**, 27쪽 **66.83% 보류** |
| OLE 분리 입력 | HWP `a29d415a…`, HWPX `a64ee3e5…`, [한컴 PDF](../../pdf/issue7406/7406_OLE__CHART.pdf) `bd3a5a0e…`. 한 페이지 Native HWP **90.67%**, HWPX **90.58%**, 두 gate 통과 |
| 전체 release-test·lint·fresh WASM | 앞선 코드 후보의 nextest 10,241 PASS/50 skip 및 lint 통과. 추가 OLE 수정 후 최종 head 기준 재실행 필요; fresh WASM 미검증 |

기존 `...-hwp2020-20260814.pdf`의 절차적 생성 출처는 [#7399 검토](archives/pr_7399_review.md)에서 미검증이었다. 이번에는 원본에 대응하는 새 한컴 2024 PDF를 생성해 판정에 썼다. PDF의 KoPubDotum subset과 Mac 글꼴 공급을 확인했고 94·108쪽은 99% 이상이다. 68~70쪽의 표·본문 배치 차이를 글꼴 예외로 분류하지 않는다.

## 시각 증적과 남은 차이

물리 69쪽의 표 보정과 94쪽 KoPub 대조군은 기존 이미지에 남겼다. OLE 분리 샘플의 백분율 축·막대 길이·색·레이블과 여성 차트가 있는 원본 27쪽의 잔여 차이를 직접 확인했다. 대표 이미지:

![#7406 OLE 차트 분리 샘플 Native review](assets/pr7406_20260925/ole_chart_review_001.png)

![#7406 OLE 차트 분리 샘플 Native overlay](assets/pr7406_20260925/ole_chart_overlay_001.png)

![#7406 원본 27쪽 Native review](assets/pr7406_20260925/native_review_027.png)

![#7406 69쪽 Native review](assets/pr7406_20260925/native_review_069.png)

![#7406 69쪽 Native overlay](assets/pr7406_20260925/native_overlay_069.png)

![#7406 94쪽 Native review](assets/pr7406_20260925/native_review_094.png)

![#7406 94쪽 Native overlay](assets/pr7406_20260925/native_overlay_094.png)

## 보류 해제와 contributor 안내

원본 27쪽의 차트 아래 벤다이어그램·텍스트 배치를 분리 조사하고, 전체 Native/fresh WASM Visual Sweep과 최종 head의 nextest·lint를 마친 뒤 판정한다. 90% 미만 페이지가 남으면 PR을 생성·승인·통합하지 않는다. 머지 시에는 실제 merge SHA와 CI URL, 같은 이미지의 merge SHA 고정 raw URL로 기여자에게 결과를 안내한다.
