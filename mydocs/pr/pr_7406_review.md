---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-25
---

# PR #7406 리뷰 — KoPub 라틴 전진폭

## 최종 판정

**머지 보류.** KoPub 집중 회귀는 통과했으나, PrEP 문서의 본문/표 겹침이 14건에서 15건으로 늘고 한컴 2024 정본 대비 68~70쪽 Visual Sweep gate가 90% 미만이다. 원 PR의 baseline 14→15 상향을 승인 근거로 볼 수 없다. 제품 조판·표 행 분할 보정과 새 head의 회귀·시각 재검증이 필요하다. 원격 CI 녹색과 `MERGEABLE/CLEAN`은 현재 로컬 실패를 해소하지 않는다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR·작성자·base | [#7406](https://github.com/edwardkim/rhwp/pull/7406), `planet6897`, `devel` |
| 원 head / `-x` cherry-pick | `f49ebdefb9254d153e20a8cfd1424be398290eb4` / `c611aa5aff6e72ac02df722e07d4bda0d2b8b6e1` |
| 검토 base / branch | `c80a8370ab294259557c850c2e54495ecd0e79c0` / `review/planet6897-7406-20260925` |
| 관련 이슈 | [#7390](https://github.com/edwardkim/rhwp/issues/7390); 이 검토로 닫지 않음 |
| 원격 참고값 | 2026-09-25 조회: OPEN, Draft 아님, `MERGEABLE/CLEAN`, 원 head의 CI 성공. 머지 전 재조회 필요 |
| 메인터너 보정 | 겹침 baseline 주석의 p68 전후 건수와 '불변' 오기 정정, 원본에서 생성한 2024 PDF·대표 PNG·검토 기록 보강. 동작 코드·15건 상한은 아직 미보정 |

## 변경과 조판 원칙 검토

원 PR은 KoPub Dotum/Batang의 Basic Latin hmtx 폭을 `text_measurement.rs`에서 사용하고, 집중 검사와 p94 Native 이미지를 추가했다. 변경 폭은 줄 나눔과 표 안 텍스트 및 선행 본문 위치에 전파된다. `tests/fixtures/text_overlap_baseline.tsv`의 PrEP 상한도 14→15로 올렸다.

실제 호출 경로에서 **미충족**을 확인했다. section 2 para 27의 빈 host 비-TAC TopAndBottom RowBreak 표는 선행 문단과 캡션 뒤 y208.0px을 전달받지만, 전체 표 높이가 본문에 맞지 않아 최종 `compute_table_y_position`이 y94.5px로 끌어올린다. 0기반 p68 render tree에서 본문 줄 y94.5/120.9/147.3px, 캡션 y187.0px, 첫 표 셀 y94.5px이다. 독립 한컴 PDF에는 선행 본문·캡션 뒤에 표가 있다. 이 경계는 행 컷/예약 높이/페인트를 함께 수정해야 하며, 최종 위치만 이동하는 보정은 미완이다. 상세 명령·노드·좌표는 Git 제외 `output/pr-review/planet6897-7406-20260925/result.md`에 보존한다.

## 검증 입력과 결과

| 항목 | 결과 |
| --- | --- |
| 원 HWPX | `samples/issue2006/1790387_prep_final_report.hwpx`, SHA-256 `c68baed24096386f9041930d24d39409b61ac99463bf04dfd242440dfdeb739f`, `hancom-office-2024` 저장 메타데이터, 140쪽 |
| 신규 한컴 PDF | [1790387_prep_final_report-2024.pdf](../../pdf/issue2006/1790387_prep_final_report-2024.pdf), 원본을 HWP MCP `--engine 2024`로 변환, SHA-256 `04b95a6e41420fb45934ce2ee5abd8cf6dac4ce12fd47977dacbe7fca28018a8`, 140쪽, Hancom PDF 1.3.0.550 |
| KoPub 집중 검사 | 검토 source head `regression_suite_016` 2/2 PASS |
| 겹침 baseline | base 실제 14건 PASS; 검토 head에서 상한 14면 15건으로 FAIL(exit 101), 상한 15면 PASS. 새 교차 4.60816px × 10.8667px; 기준값 올림은 회귀를 숨김 |
| Native Visual Sweep | 96dpi, `/Users/tsjang/Library/Fonts` 실제 KoPub 공급, 물리 67/68/69/70/94/108쪽 2px 내용 실루엣 **90.33392/50.05/41.16299/35.15547/99.55839/99.79788%**. `re_review_required` |
| 전체 release-test·lint·fresh WASM | **미검증**. 앞선 시각·겹침 게이트 실패로 승인 검증으로 진행하지 않음 |

기존 `...-hwp2020-20260814.pdf`의 절차적 생성 출처는 [#7399 검토](archives/pr_7399_review.md)에서 미검증이었다. 이번에는 원본에 대응하는 새 한컴 2024 PDF를 생성해 판정에 썼다. PDF의 KoPubDotum subset과 Mac 글꼴 공급을 확인했고 94·108쪽은 99% 이상이다. 68~70쪽의 표·본문 배치 차이를 글꼴 예외로 분류하지 않는다.

## 시각 증적과 남은 차이

물리 69쪽 Native review와 overlay에서 본문·캡션 위로 표가 올라온 상태를 직접 확인했다. 물리 94쪽은 KoPub 라틴 폭 변경의 국소 개선을 확인하는 대조군이다. 대표 이미지:

![#7406 69쪽 Native review](assets/pr7406_20260925/native_review_069.png)

![#7406 69쪽 Native overlay](assets/pr7406_20260925/native_overlay_069.png)

![#7406 94쪽 Native review](assets/pr7406_20260925/native_review_094.png)

![#7406 94쪽 Native overlay](assets/pr7406_20260925/native_overlay_094.png)

## 보류 해제와 contributor 안내

표의 실제 행 분할과 앞/뒤 문단·캡션의 점유를 공통 결과로 보정한다. 수정 전 실패·수정 후 통과하는 정식 회귀를 추가하고, 새 head의 Native/fresh WASM 67~70쪽 review·overlay를 직접 판독한다. 90% gate를 넘은 뒤 전체 검증과 최신 CI·mergeability를 확인한다. 머지한 경우에만 실제 merge SHA와 CI URL, 같은 이미지의 merge SHA 고정 raw URL로 기여자에게 변경 범위와 남은 차이를 안내한다. 현재 GitHub comment·approve·push·merge는 하지 않았다.
