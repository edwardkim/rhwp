---
kind: evidence
status: active
canonical: mydocs/manual/pr_review/visual_fixture_evidence.md
last_verified: 2026-10-12
---

# 교육과정 연결 맵의 기존 쪽수 검사 재검토

이 자료는 [열린 #7445](https://github.com/edwardkim/rhwp/issues/7445)에 이미 등록된 같은 원본·쪽 소속 결함의 추가 증거다. 정상 fixture, 출력 개선 완료 또는 새 회귀 복원으로 분류하지 않는다. 바이트 동일 사본을 확인한 뒤 활성 원본·검사·원장을 이관했다. 후속 필수 검증은 진행 중이며 제품 결함 해결과는 구분한다.

## 원본과 기준

- [원본 HWP](1342000_edu_curriculum_map.hwp): SHA `623b00d56beffc45d27c5bf23911bdc49d3a541ded8aecbb323d0716a2bc9f4e`. 고정 `upstream/devel` SHA `21ed64071f81f35f37618717693bd30f727392c9`의 `samples/task2287/1342000_edu_curriculum_map.hwp`와 현재 입력이 모두 바이트 동일하다.
- [같은 원본의 현재 한컴2020 Print](current-source-hancom2020-print.pdf): 415쪽, SHA `82fe8eefecbf3ad5176549976a8cd486ac0bb9c113e7ac34510a69ee6ccbf17f`, runtime `11.0.0.9136`, PrintMethod0/1-up, 실제 성공 job `33f4cda0-cf07-4ffd-8617-19ae17427c4d`.
- 기존 [2020 이름 bucket 출력](existing-hwp-hancom2020-print.pdf), [2022 출력](existing-hancom2022-print.pdf), [2024 출력](existing-hancom2024-print.pdf), [25–35쪽 발췌본](existing-pages025-035-excerpt.pdf)도 변경 없이 보존한다. bucket 이름과 실제 Creator를 구분한다. 발췌본은 전체 쪽수 정답지가 아니다. 이번 base/head 대조에는 위의 같은 원본 현재 Print만 사용한다.

## 고정 base와 현재 head

현재 커밋 `12d96686cdd24017b44becead2c4dfeb46867bf3`의 production source는 v86 retry2 / `9b53bd699126205b7a93fff326167f575252f1ea`와 같다. Native binary SHA `a41dbbebdf51f7c2fa928c6f2a7d5bd80a80a91bb9c904b3320fbd1afed8f2e2`를 사용한다. 고정 base의 별도 빌드 binary SHA는 `b5fa531bdf40ce2b80692b11d9451b1f96ab2579260f644ddbffdff3ac5a98e0`이다. 실행 루트의 git_head를 base binary의 source SHA로 오인하지 않는다.

같은 font 경로·96dpi/print profile/compat2022/2px 관용 조건으로 Native247/248/259–262쪽을 대조한다. base는 terminal exit1/555.895초/source불변·6쪽 모두 측정했으며 점수는 51.88058/71.40332/64.23382/66.64074/15.22185/17.20583%다. 실제 전체 쪽수는 base413/Print415다. 261쪽 직접 review에서 Native의 다문화교육 도식과 Print의 앞 표 내용이 서로 다른 쪽을 소유함을 확인했다. 현재 head도 terminal exit1/634.113초/source불변·선택6쪽 측정을 끝냈다. 같은 순서의 점수는73.30430/71.45219/72.98027/64.80141/68.90873/6.42706%이고 전체414/Print415쪽이다. Native6쪽 미달이 확인됐으며 전쪽/fresh WASM은 미검증이다. 전체415쪽 또는 fresh WASM 대조로 확대하지 않는다. 실제 embedded face는 HYGulim/HCRBatang regular·bold이며 모든 Print font와의 동일성은 주장하지 않는다. 내용 소속 차이를 글꼴 예외로 면제하지 않는다.

## 검사별 판정과 보존

| 검사 | 판정 | 변경 전 소스 |
| --- | --- | --- |
| 6761 `partial_retreat_inside_a_page_does_not_break` | 기존413쪽 고정은 Print415와 다르다. 실제 부분 후퇴 문단의 소속을 검사하지 않고 총 쪽수만 검사한다. 이관한 부적합 쪽수 검사. | [base](issue_6761_stored_vpos_rewind_page_break.rs.base.rs.txt), [현재](issue_6761_stored_vpos_rewind_page_break.rs.before-migration.rs.txt) |
| 7226 기존 `the_fix_does_not_add_a_page` → 현재 `page_count_matches_independent_print` | 415라는 독립 기대값 자체는 정확하지만 전쪽90% 전의 복원은 승인되지 않았다. 실제414대415 실패를 보존하며414로 허용하거나 틀린413으로 되돌리지 않는다. 이관한 부적합 쪽수 검사. | [base](issue_7226_rowspan_only_row_cut.rs.base.rs.txt), [현재](issue_7226_rowspan_only_row_cut.rs.before-migration.rs.txt) |

다른 원본 MARKET의 `stored_rewind_paragraph_starts_the_next_page`, 7226의 내용 소유/겹침 및 합성 예산 계약, 6981의 셀 포함/이어받기/종료, 저장 줄 메트릭, 기존2287/2291 검사 등 공유 계약은 이 문서의 전체 미달만으로 일괄 제거하지 않는다. 형식·IR roundtrip의 기존 논리 이름과 16626/216건 IR 원장도 유지한다. 정확한 기존 검사의 실패가 발견되면 구현을 고친다.

정확한 명령·binary/input/Print/font SHA·metric·로그는 ignored `output/pr-review/semanticist21-20261008/`의 `native-base-education-suitability-actual-faces-v92-command.json`, `native-current-education-suitability-actual-faces-v92-command.json`, 각 run_manifest 및 exit 기록에 보존한다. TSV·로그·중간 JSON·font 파일은 이 asset에 커밋하지 않는다.

## 복원 조건

같은 원본을 먼저 보정하고 같은 독립 Print와 Native/fresh WASM의 관련 모든 페이지에서 최저90% 이상·쪽수 일치·누락 없음 및 수식/미세 간격/쪽 소속의 직접 판독을 확인한다. 그 뒤 쪽수와 내용의 의미·소유·보존 관계를 검사한다. 부분 개선·이관·검사 PASS만으로 #7445를 종료하지 않는다.


## 내용 소속의 직접 대조

[base261쪽](base-native-review_261.png), [현재261쪽](current-native-review_261.png), [base262쪽](base-native-review_262.png), [현재262쪽](current-native-review_262.png)을 같은 Print와 직접 판독했다. base는261쪽에 다음 다문화교육 도식을,262쪽에 그 다음 표를 놓았다. 현재261쪽에는 앞 표의 내용이 오지만 일부 행을 앞당겨 넣고,262쪽에는 여전히 Print의 앞 표 마지막 밴드 대신 다음 도식을 놓는다. 현재262의 낮은 점수만으로 새 도식 손상이라고 판정하지 않는다. base261/current262의 **Body subtree 전체 내용·기하가 바이트 기준으로 동일**하고 한 쪽 뒤로 이동했음을 별도로 확인했다(`education-diagram-base-current-owner-comparison-v92.json`). 본문 내용·기하 보존과 잘못된 페이지 소속은 구분한다. 기존 쪽 소속 미달이 해소된 것은 아니며 전쪽 무회귀로 확대하지 않는다.

[같은 원본의 검사별 재검토 등록 댓글](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-6091651108)을 게시하고 API로 UTF-8 본문이 전송 원문과 동일함을 확인했다. 실제 이관 후 필수 검증은 아직 진행 중이다.

실제 이관은 부적합 쪽수2함수와body/oracle/render-page/PDF reference 각1행을 제거했고 기존text_overlap의불필요한deferred이름1개도제거했다. 새skip은없다. 정확한공유검사는같은이슈원본을계속읽으며 HWP5 roundtrip/IR baseline은원래논리이름을유지한다. 원본1개와PDF5개는바이트동일하게이동했다. 과거Markdown의실제파일링크만새경로로연결하며역사SHA/수치/label은변경하지않았다.


## 이관 후 재현 경로

기록한 원래 명령의 active samples/PDF 경로는 이관 전 실행 경로다. 재현할 때는 같은 바이트의 `mydocs/pr/assets/issue7445/education-curriculum-map/1342000_edu_curriculum_map.hwp`와 `current-source-hancom2020-print.pdf`를 `--file-target education`의 원본/기준 인자로 전달한다. 원래 파일명·전체 내용·SHA는 유지했다. base와 현재 binary, 같은7개 font 경로·페이지247/248/259–262·compat2022·96dpi 조건은 유지하고 새 `--out` 경로를 쓴다. 기존 로그/PNG를 덮어쓰지 않는다.

## #2291 첫 조각 고정 높이 검사의 추가 이관

같은 원본의 미해결 쪽 소속은 [#7445의 하위 이슈 #7743](https://github.com/edwardkim/rhwp/issues/7743)에서 추적한다.
고정 base `bfa28a77a2714482f647ee6a7b9aa7f7876d8404`에 검사와 원본이 이미 존재하고,
원본 바이트는 위 SHA와 동일하다. 현재 검토 head는 `6fee79e6afbad90d72c3ae7b94617c29922557f9`,
생산 소스는 `940ea1322617ccff5a7b47d6920ae96e94cfffad`와 동일하다.
기준은 위의 같은 원본 한컴2020 Print 415쪽이다.

| 동일 조건 출력 | 139쪽 | 140쪽 | 전체 쪽수 |
| --- | ---: | ---: | --- |
| 고정 base Native | 77.52827% | 75.29239% | 418 / Print 415 |
| 현재 Native | 74.39013% | 76.06998% | 422 / Print 415 |
| 현재 fresh WASM | 74.39013% | 76.06998% | 422 / Print 415 |

Native는 `--profile print --compat 2022 --embed-fonts=full`, fresh WASM은 같은 compat와
문서 전체 Native 글꼴 별칭 정책을 적용했다. 96dpi·2px 관용 지표와 같은 7개 검증 글꼴 경로를 썼다.
base WASM과 나머지 전쪽은 이번 검사 적합성 판단에서 실행하지 않았다.
현재 두 쪽의 Native/fresh PNG는 각각 decoded RGB가 동일하며, 네 review를 직접 판독했다.
base부터 Print와 행 내용·쪽 소속이 다르다. 현재의 미달과 쪽수 증가도 그대로 남기며 출력 해결이나
전쪽 무회귀로 보고하지 않는다.

- base: [139쪽](row183-base-native-review_139.png), [140쪽](row183-base-native-review_140.png)
- 현재 Native 및 동일 RGB의 fresh WASM: [139쪽](row183-current-native-and-identical-fresh-review_139.png), [140쪽](row183-current-native-and-identical-fresh-review_140.png)
- 검사 원문: [고정 base](issue_2291_rowspan_declared_residual.rs.base.rs.txt), [제거 전](issue_2291_rowspan_declared_residual.rs.before-migration.rs.txt)

제거 범위는 `tests/issue_2291_rowspan_declared_residual.rs`의
`issue_2291_rowspan_declared_residual_expands_last_spanned_row` 한 함수와 그 전용 높이 탐색 helper 두 개다.
이 검사는 원본 표 소유를 확인하지 않고 첫 `(row183,col8)` 조각에 전체 행 높이 55–58px를 요구한다.
실제 s5/pi8의 244×10 표를 추적하면 base의 한 조각은 56.3px이며, 현재는139쪽 41.9px와
140쪽의 글자 없는 물리 잔여 14.5px로 나뉜다. 현재 값을 새 기대값으로 복사하거나 허용치를 넓히지 않는다.
이미 base에서 정상 출력 기준을 충족하지 못한 잠정 배치 검사만 이관한다.

기존 `issue_2291_stored_single_seg_paragraph_rewraps_full_text`와 다른 정확한 공유 검사는 유지한다.
원본 HWP·Print PDF를 삭제하지 않았으며 baseline·총 쪽수·manifest 공차도 바꾸지 않았다.
위 복원 조건을 충족한 뒤 원본 표의 소유·내용 보존·물리 잔여의 관계 검사로 복원한다.

정확한 명령·글꼴 경로·해시·terminal 결과는 ignored 검토 출력의
`education-source-row-owner-v374/result.json`, `education-row-fragments-v375/result.json`,
`education-exact-row-visual-v376/`, `education-exact-row-visual-v385/`,
`education-suitability-review-v393/result.json`에 보존했다.
base Native 바이너리 SHA는 `8312404e26a74d317da14054b2bae22c460625a1928848302fe1d5d445184e47`,
현재 Native는 `3d50e1311eb579afd41d0dcf3bafade916b8b763a633083c8f89d2b001340ae3`,
fresh WASM은 `f16609ba3bac35ab0292a0c15c2bc36d8becb562834f98090fb1831cc2d517f9`다.
