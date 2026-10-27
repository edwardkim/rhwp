---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase9-source-predicate-report.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase9-source-predicate-report.md` · 원문 SHA-256: `7dd4787c1378ca6c2df758f2cac4baad83219f9704b7b58769589ffda70306ad`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

## #8232 추가 조사 — 소스 내부 검사와 오래된 렌더러 전제

조사 head는 `238de6de41a208648f4ade6b494fe433a81c1fce`입니다. 전수 조사 중이며 제품·테스트 재작성과 이름 변경·커밋·PR 제출은 아직 없습니다.

### 추가 source 160그룹

전체 단언의 직접 수치 의미와 필요한 producer 문맥을 대조했습니다. 158그룹은 문자/자원 인덱스·HWP bit/byte·필드 편집/저장·CSV/수식 데이터·보안 진단 건수·직접 작성 EMF 형식/좌표·캐럿 탐색/메모리/시간 한도 등의 계약입니다. 소수나 width라는 이름이 있다는 이유로 실물 페이지의 절대 px 핀으로 취급하지 않습니다. 함수의 직접 수치 분류는 전체 helper 호출과 독립 시각 수용의 완료를 뜻하지 않습니다.

나머지 두 그룹에서는 기존 기대값의 전제와 현재 renderer의 불일치를 발견해 추가 근거 대조 대상으로 남겼습니다.

### 상대 글자 크기를 무시한다는 전제가 현재 코드와 다릅니다

`src/document_core/queries/hidden_text.rs`의 다음 두 검사는 실제 진단 결과를 10pt로 고정합니다.

- `effective_size_ignores_relative_sizes_like_the_renderer_does`
- `default_relative_sizes_can_never_cause_a_zero_size_misjudgment`

이 파일의 설명은 renderer가 `relative_sizes`를 전혀 읽지 않으므로 진단도 곱하지 않는다고 합니다. 현재 `src/renderer/style_resolver.rs`와 `ResolvedCharStyle::font_size_for_lang`은 언어별 상대 크기를 적용합니다. 진단 호출은 여전히 `classify_char` → `effective_pt`로 기본 크기만 소비하며 문자별 언어 슬롯을 전달하지 않습니다.

현재 source의 Native rlib를 이용한 별도 관찰 프로그램을 컴파일·실행했습니다. rlib SHA-256은 `5768866158c10fc9e1336cc286e2daa3a8d548baaaf602a89359a2e8de217c27`입니다. 직접 작성한 기본 1000HU(10pt) 입력에 대해 같은 언어 슬롯을 비교한 결과입니다.

| 상대 크기 | 선언 비율로 계산한 크기 | 실제 resolved 크기 | 진단 크기 | 2pt 기준 진단 |
| --- | ---: | ---: | ---: | --- |
| 100% | 10pt | 10pt | 10pt | 없음 |
| 10% | 1pt | 1pt | 10pt | 없음 |
| 0% | 0pt | 0pt | 10pt | 없음 |

이는 언어별 resolved size와 실제 진단 함수 반환의 실행 증거입니다. 최종 paint의 크기, 독립 한컴 Print, 실제 문서 전체 시각 수용을 검증한 것으로 확대하지 않습니다. 0%는 원 유효 범위 밖 입력이므로 정상 10% 사례와 구분하며, 잘못된 입력에 대한 계약을 독립적으로 확인해야 합니다.

같은 source의 기존 release-test root binary에서 위 두 정의를 각각 exact filter로 실행했습니다. 두 검사 모두 1 PASS/0 FAIL, 각 3812 filtered로 끝났습니다. 전체 회귀 또는 보정 후 통과가 아닙니다. 현재 PASS가 오래된 기대값의 유효성을 입증하지 않는 추가 사례입니다.

### 원 제안과 실제 반영 경로

- 원 제안 #3809는 CLOSED·미병합입니다. 실제 통합은 #3816의 `6ab503fe97b7abfd1839800c5c018da9f9abf4c5`이며, 해당 함수 도입 커밋은 `8a1db1933c719a9f8ccf3aa36bf51e9245e06b72`입니다.
- #4171의 `23ff5b6f1acd67aaccdf1ecfb47a7dc1918a13ed`는 HWP3 상대 크기 기본값을 100으로 보정하면서 진단 설명을 갱신했지만, 여전히 renderer가 상대 크기를 곱하지 않는다는 결론을 유지했습니다.
- 언어별 상대 크기 소비가 들어간 `7eddd19da61247fcd291cb3c1912ab4037d8daae`는 실제 통합 #7505에 연결됩니다. 후속 #7805의 글자 장치 격자·자간 규칙도 상대 크기 소비를 전제로 합니다.
- 위 실제 통합 커밋들이 현재 HEAD의 ancestor임을 확인했습니다. 원 PR의 `merge_commit_sha` 필드만 보고 병합됐다고 판단하지 않았습니다.

두 검사에는 실물 페이지의 절대 px 위치 핀은 없습니다. 다만 현재 구현을 따라야 한다는 오래된 가정으로 기대값을 정한 문제여서, 단순 정상 단위 검사로 승인하지 않았습니다. 독립 근거와 실제 소비 경로를 추가 연결해 필요한 기대 계약과 보정 범위를 정합니다. 모든 보안 검사를 변경하거나 범위를 다른 기능으로 확대하는 근거로 쓰지 않습니다.

전수 조사·독립 증거 판정·필요한 실제 보정과 `issue_8232` 명명·등록/검출력 재검증은 계속 미완료입니다. PR은 나중에 진행합니다.

### 추가 원본 모델·paint 경로 대조

이후 source 160그룹을 더 대조했습니다. 문단 문자 축·표 모델의 행/열/병합/분할·원본 HU 치수·raw provenance·OLE 차트 원본 데이터·작성한 viewport 인수·paint op/자원/clip 보존 검사입니다. 직접 단언 의미상 실물 페이지의 잠정 절대 px 핀은 없었습니다. 실제 호출/helper와 독립 출력 대조를 생략했다는 뜻은 아닙니다.

이번 두 묶음은 합계 320그룹이며, 318그룹의 직접 수치 목적을 구분하고 위 두 그룹의 오래된 renderer 전제를 추가 근거 대조 대상으로 남겼습니다. 감사 중 일부 기록의 원본 수치를 잘못 옮긴 부분은 해당 source와 다시 대조해 공개 전 정정했습니다. 제품·테스트·지침은 변경하지 않았습니다.
