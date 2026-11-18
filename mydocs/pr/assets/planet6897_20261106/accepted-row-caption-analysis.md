# 수용한 개체 줄 원점과 캡션 저장 상자

## 기준과 원본

2026-11-06, base `45684640be5eddcf5972b063260250345b34d928`, 구현 시작 head
`3de4766e14e244a58261eea4af93d2879fcda41e`에서 보정했다. R8·R9는 이 head에
수정한 파일의 SHA-256을 각각 `output/pr-review/planet6897-integration-20261106/`의
`common-fixes-r8-source.json`·`common-fixes-r9-source.json`에 고정한 별도 엔진이다.
`*-producer.json`은 실제 실행 파일 해시와 빌드 종료 시각을 기록한다.

원본·독립 한컴 Print는 다음 정식 기록을 유지한다. 진단 저장본으로 바꾸지 않는다.

- [36474334 원본·Print](pr8681/36474334-original.md): 2쪽 결재문서.
- [838 보고서 원본·Print](pr8642/report-838-print-reference.md): 109쪽 보고서.
- [628 보고서 원본·Print](pr8642/report-628-print-reference.md): 195쪽 보고서.

## 두 번째 TAC 표가 첫 표의 높이만큼 더 내려간 원인

원본 조판을 재구성한 문단 3은 각 TAC 표를 서로 다른 물리 줄에 배정한다.
첫 표 앞 pen은 117.7467px, 두 번째 표 앞 pen은 458.8933px다. 두 표의 실제 줄 전진은
341.1467·383.0133px이며 뒤 앵커 줄도 별도로 소비한다. 문단 번호·좌표를 분기 조건으로
사용하지 않는다.

| 생산 → 소비 | 실제 보정 |
| --- | --- |
| 현재 구성한 제어 소유 줄 | `composer::composed_tac_row`가 같은 줄의 여러 표와 저장 줄을 제외하고 독립 개체 줄을 판정 |
| fit·흐름의 실제 수용 pen/끝 | `place_table_with_text`가 같은 줄 높이와 실제 전진을 확인한 `InlineBoxPlacement`를 발행 |
| 배치의 실제 표 원점 | `layout_table_control_block`이 확정된 세로 프레임을 직접 소비해 레거시 원점 재해석을 생략 |
| 마지막 원점 덮어쓰기 | `composed_empty_host_tac_line_offset`은 확정 프레임이 없을 때만 더함 |
| 공동 앵커와 표 뒤 내용 | 확정 개체 줄 pen으로 문단 원점을 덮어쓰지 않아 글앞 표의 원래 문단 앵커도 보존 |

R8은 pen을 발행했지만 마지막 offset helper가 이를 다시 더해 1쪽 46.97%에 머물렀다.
R9는 최종 소비 지점과 문단 앵커 덮어쓰기까지 바로잡았다. 독립 PDF와 같은 2쪽이며
1쪽 99.90%·2쪽 100.00%다. 두 표의 붙는 위치, 아래 표와 글앞 결재란, 내용 순서·보존,
1쪽의 실제 내용을 review·overlay로 직접 확인했다. 2쪽은 독립 Print와 RHWP 모두 빈 쪽이다.
빈 쪽의 100.00%를 내용 보존 증거로 삼지 않으며 원본·Print와 같은 쪽 경계인지 별도로 확인했다.

## 위 캡션이 포함된 원본 외곽 줄 상자

838 보고서의 24쪽 각주 표는 저장 줄 높이가
`본체 7970 + 위 캡션 900 + 캡션 간격 852 + 상하 여백 280 = 10002 HU`다.
원래 무캡션/아래 캡션 판정에서 이 줄을 거절하고 기준선 폴백을 사용한 뒤 위 캡션을
다시 배치해 532HU(96 DPI에서 7.093px) 이동했다. 수치는 원인 확인용 원본 구성 성분이며
보정 상수가 아니다.

`stored_vertical_caption_hu`가 유효한 저장 위·아래 캡션의 같은 높이를 제공하고,
`tac_table_stored_outer_band_top`과 캡션 포함 외곽 판정이 이를 소비한다. 저장 줄의 편집·
합성 상태와 좌우 캡션은 제외한다. 아래 캡션 전용 후속 흐름 계약과 무캡션 원점 판정은
유지한다. R8 Native 전 109쪽 최저 94.99%, 24쪽 100.00%, 10쪽 100.00%를 확인했다.
R9의 최종 소비 경로 변경 후 동일 보고서의 재측정은 별도로 완료해야 한다.

## 검증 범위와 미완료 항목

- R8 Native: 20개 입력 341쪽 실제 완료. 838 보고서가 미달에서 통과로 개선됐고 새 base의
  PR #8658 관련 8개 입력 10쪽은 모두 100.00%를 유지했다.
- R9 Native 집중: 결재 원본 36474334 전 2쪽 99.90/100.00%. 인접 원본·캡션·저장 띠
  대조군까지 5개 입력 8쪽이 같은 엔진에서 모두 90% 이상이다. `common-fixes-r9-boundary.log`와 케이스별 exit·TSV를 따른다.
- R9 최소 글꼴 Rust 집중 검사는 `common-fixes-r9-focused.log`에서 별도로 실행 중이다.
  이전 R7의 23 PASS/1 잘못된 원본 전제 FAIL을 새 실행 성공으로 재표기하지 않는다.
- 628 보고서 13쪽 미달의 수식·부동 표 결함, fresh WASM, 최종 전체 Rust/lint/Skia,
  보호 PR #8389·#8397 TSV와 새 base 실제 비교, 편람 게이트와 개별 최종 리뷰는 미완료다.

정식 px 회귀는 양쪽 raw 98% 이상과 직접 판독 조건을 충족한 뒤 등록한다. Native 단계의
개선을 최종 승인이나 새 회귀 등록 근거로 승격하지 않는다. 승인·push·PR 생성은 하지 않았다.

## 보존한 Native 증거

| 원본·엔진 | 전체 TSV | 같은 주장 영역의 review·overlay |
| --- | --- | --- |
| 36474334·R9 | [전 2쪽](accepted-row-caption/36474334-r9-native.tsv) | [1쪽 review](accepted-row-caption/36474334-r9-p001-review.png) · [overlay](accepted-row-caption/36474334-r9-p001-overlay.png) |
| 838 보고서·R8 | [전 109쪽](accepted-row-caption/report838-r8-native.tsv) | [24쪽 review](accepted-row-caption/report838-r8-p024-review.png) · [overlay](accepted-row-caption/report838-r8-p024-overlay.png) |

[출처·엔진·파일별 SHA](accepted-row-caption/provenance.json)를 함께 보존했다. 두 엔진의 결과를 같은 최종 head의 결과로 합치지 않는다.
