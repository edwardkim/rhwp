# PR #7699 리뷰 — 저장 Square 상단 예약과 본문 빈 줄 소유 보정

## 최종 판정

**메인터너 보정 후 수용 가능 — 저장 Square 표와 본문 빈 줄 배치 범위의 로컬 검증 완료.**

원 PR의 다단 제목 표 아래 본문 배치는 개선되지만 allowOverlap·좁은 표·정상 빈 줄에서 추가 보정이 필요했다. 원 contributor 변경과 메인터너 보정을 같은 그래프에 보존한다. 정확한 통합 code head b32de5256의 CI·CodeQL·Render Diff·Adapter·Proptest·정책 검사도 모두 완료했다. 이 review와 오늘할일은 같은 통합 PR의 문서 trailing commit으로 반영하며 새 trailing head CI·mergeability는 별도 확인한다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR / 작성자 / base | #7699 / mytrashcan (JEONG KITAE), 첫 rhwp 기여 / devel |
| 원 source / 체리픽 | `11f3e5838af0c5b5bf97da8a672ad934e1510b04` / `b6a865ca6` (-x·author 보존) |
| 검증 devel | `4662c069e557e928ed7968d061121fde90b1f027` (#7913) |
| 메인터너 보정 / 실제 producer | `9e2cca42a015c965106698a1689040fa56e871e3` |
| 검증한 code candidate | `b32de5256dbab1a2122a58e219f05c9b2cab8deb` (9e2 이후 문서·PNG만 변경) |
| 로컬 작업 브랜치 | `review/mytrashcan-7699-20261016` |
| 관련 이슈 | Refs #7548, #7699. #7548 전체 종료를 주장하지 않는다. |
| 원격 상태 | 사용자 승인으로 통합 [PR #7948](https://github.com/edwardkim/rhwp/pull/7948)을 생성했다. 정확한 code head의 필수 CI를 모두 확인한 뒤 이 기록을 같은 PR의 문서 trailing commit으로 반영한다. |

## 변경과 조판 원칙

[생산→측정→실제 배치와 반례 기록](pr_7699_review_impl.md)을 실제 호출 경로와 대조했다. 동일한 저장 셀 높이·절대 표 원점·바깥여백·LayoutFrame 차선 판정을 페이지네이션과 첫/다음 단이 소비한다. 문서 ID·80% 폭 비율로 분기하지 않는다. 옆 차선이 남는85% 표, allowOverlap만 변경한 독립 Print, 편집/합성 저장 줄 비적용 경계를 구분한다.

빈 줄의 저장 사다리가 닫힌다는 사실은 소유 증거가 아니다. 절대 Page/Paper 표가 host 본문만 전진하는 기존 분기에서만 정상 저장 빈 줄을 본문으로 처리하고, 문단 상대 표의 안내 줄은 통째 예약·이월 뒤에도 표 프레임에 보존한다. 첫 보정의 전체12실패와 밴드 유무만 본 중간5실패를 구현에서 고쳤다. 기존 기대값·golden·baseline은 완화·제거·이관하지 않았다.

표 셀 컷·rowspan·continuation 규칙은 변경하지 않는다. 본문의 단/쪽 이어받기는 전체 내용 순서·누락·중복·소유와 Print3쪽을 함께 검사한다. 해당 범위의 판단은 충족이며, 별도 캡션·편집 후 저장 정보·다른 anchor 일반성까지 검증했다고 확대하지 않는다.

## 입력·전쪽 시각 증거

[원본·독립 Print 출처와 SHA](../../../tests/fixtures/issue_7699/README.md), [전쪽 수치·페이지별 px·대표 PNG](../assets/pr7699/README.md).

공식 학술대회 공개 양식 원본과 실제 한컴2020 저장 대조군을 사용했다. 원본 HWP SHA는 `0e9d59a6ecf52dce759598997568680edc3955b189efdfb9b115ec9b2cc3057c`다. 원본/입력5개와 독립 Print4개는9e2 Git blob과 같은 바이트이며, 각 Print는3쪽·약227KB로50MiB 미만이다. 폰트 프로그램은 추가하지 않았다.

| 항목 | Native Skia | fresh WASM Canvas2D |
| --- | ---: | ---: |
| 전체 입력 / 쪽 | 9 / 17 | 9 / 17 |
| 최저2px 관용 내용 실루엣 | 99.73828% | 99.69093% |
| 90% 미만·누락·쪽수 불일치 | 0 | 0 |
| 기준 대비0.01%p 이상 하락 | 0 | 0 |

96dpi·compat2022·PrintMethod0·동일 실제 글꼴 공급 조건이다. 최신 devel와 보정본의 원수치34쌍을 비교했다. 수치만으로 승인하지 않으며 원시 manifest의 not_evaluated는 그대로 보존한다. 전체 TSV 원본36개·68행은 통합 PR 본문에 직접 게시했고 API로 원문 일치를 확인했다. [백업 원본](https://gist.github.com/jangster77/0e4e73ffd4a4065801bc75f37c20d56b)의37파일도 API 바이트 일치를 확인했다.

p1 본문/양단 시작·의도된 안내 상자 겹침·좁은 표 옆 차선/빈 줄, p2 수식 Σ/φ·괄호·첨자·이탤릭·식 번호·주변 간격·표/그림/참고문헌, p3 안내 표를 같은 배율로 직접 읽었다.

## 정식 회귀와 로컬 검증

`tests/cases/issue_7699_stored_square_top_band.rs`5함수는 전체 쪽수·내용 순서/누락/중복·본문/표 포함 및 비겹침·빈 줄 소유를 검사한다. 동일 정식 소스는 원 PR 라이브러리에서2 PASS/3 FAIL이다. 원본3종과 좁은 표 각각 핵심 p1의 양쪽99%대·직접 판독을 근거로 독립 Print 문자 origin의72pt→96dpi 기준선±2px도 보호한다. 핵심 동작이 없는 높은 p3를 대신 선택하지 않았다.

| 검사 | 현재 결과 |
| --- | --- |
| 관련 Native | 26 PASS |
| Native Skia lib | 4,043 PASS / 12 ignored |
| 필수 그림 / 직접 PDF | 2 PASS / 4 PASS |
| fresh WASM | 새 빌드·실제 성공 초기화 해시 확인 |
| fmt / Native·WASM Clippy / workspace build·all-target Clippy / manifest | PASS |
| release-test 전체 nextest | 10,440 PASS / 45 skipped / 실패0; 1,307.752초 |

Cargo jobs8·공유 target/pr-review·test threads12, fresh WASM codegen-units16을 사용했다. 최초 workspace ENOSPC 링크 실패는 실행 종료·열린 파일 없음 확인 뒤 옛 증분 캐시8.96GiB만 정리하고 동일 명령으로 재실행해 통과했다.

실제 Studio Canvas2D·CanvasKit compat/default × screen/print ×3입력18캡처는 backend/profile·성공 초기화 WASM 해시가 일치하며 fallback·replay hard 위반0이다. 실제 CanvasKit Print p1은 원본99.94575%, 좁은 표99.93156%, 정상 대조군99.84876%다. 보고용 backend 픽셀 동일성12건은 미달이며 전체 readiness/performance 성공으로 바꾸지 않았다.

## 남은 차이와 판정 범위

Native p1 `영문제목`은 기존 HFT `한양견명조` 요청에 bundled `Noto Sans KR`가 선택되어 독립 Print보다 얇다. 최신 base/head의 동일 제목 crop RGB는 같으며 이번 변경은 본문 배치를 해결한다. 이 기존 glyph 선택을 px 기준으로 고정하지 않고, 서체 완전 일치나 #7548 전체 해결을 주장하지 않는다. 설치 글꼴·family name table은 변경하지 않았다.

## 정확한 code head CI

검증 후보 `b32de5256dbab1a2122a58e219f05c9b2cab8deb`의 아래 workflow는 모두 completed/success다. source·test·fixture·workflow를 바꾸는 trailing 보정은 없다.

- [CI](https://github.com/edwardkim/rhwp/actions/runs/38875897707)
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/38875897869)
- [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/38875897448)
- [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/38875897827)
- [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/38875897936)
- [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/38875897949)

정확한 head의 check/status36개는 SUCCESS 32·정책상 SKIPPED 4·NEUTRAL 0이며 실패·대기0이다. 별도 GHAS CodeQL SUCCESS와 CodeQL 네 언어 Analyze 성공도 확인했다. 원 PR의 과거 CI로 대신하지 않았다.

PR 본문은 UTF-8 LF와 SHA 고정 이미지6개를 API·실제 브라우저에서 확인했다. TSV36개·데이터68행을 본문에 직접 표시했고 모든 원문 일치와 브라우저36블록 표시를 확인했다. 이 code candidate와 새 trailing head의 CI를 구분한다.

## 원격 CI·후속 절차

사용자는 PR 생성·정확한 code CI 확인·문서 trailing·최종 CI·merge·후속 처리를 승인했다. 정확한 code candidate CI 성공을 확인했으며, 이 원 PR 번호별 review·오늘할일을 같은 통합 PR의 문서 trailing commit에 반영한다. 새 trailing head의 CI·mergeability를 확인한 뒤 일반 merge한다. merge 후에는 원 PR에 존댓말로 최초 기여 감사·메인터너 보정 이유·정확한 merge SHA 고정 이미지·전쪽 TSV를 안내하고 통합 PR 링크와 함께 superseded close한다. post_merge.md에 따라 devel 동기화와 전용 중간 자료 정리를 진행한다.

## 첫 기여자 확인과 보정 출처

GitHub author_association은 FIRST_TIME_CONTRIBUTOR이고 저장소 author 검색의 PR 수는1개(#7699)다. 원 fork `mytrashcan/rhwp:fix/body-wide-square-table`와 source11f3가 그대로임을 확인했다. 원 fork에 devel·다른 PR 이력을 밀어 넣지 않았으며 fork branch를 삭제하지 않는다. 현재 통합 경로는 collaborator_external_pr의 체리픽 별도 branch 절차이며, 원 author와 -x 출처를 유지하고 보정 commit을 분리했다. 원 PR 라벨 bug/rust/layout/rendering/table 및 milestone v1.0.0을 통합 PR에도 반영했다. self integration PR에는 reviewer를 지정하지 않는다.

## Merge 후 contributor PR comment 계획

[Visual Sweep GitHub merge comment 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을 안내하고, 아래 실제 증거만 한국어 존댓말로 게시한다. 별도 자동 flagged 후보 집계는 실루엣 전용 실행에서 산출하지 않았으므로0건으로 보고하지 않는다. 전쪽90% 미만·누락·쪽수 차이0은 별도 TSV·Print 대조 사실이다. 원본/정상 HWPX/allowOverlap/좁은 표 각3쪽과 정상 대조군5쪽을 두 backend로 확인했고, 영향 p1/p2와 p3 전체 서식 표를 직접 판독했다.

| 대표 p1 | 전체 픽셀 일치 | 내용 픽셀 proxy | 2px 내용 실루엣 |
| --- | ---: | ---: | ---: |
| native / kiit-standard | 92.93582% | 16.4357% | 99.98326% |
| native / kiit-narrow85 | 93.09386% | 19.35665% | 99.98678% |
| wasm / kiit-standard | 92.93964% | 16.75119% | 99.90991% |
| wasm / kiit-narrow85 | 93.18045% | 20.52009% | 99.93899% |

전체 픽셀·내용 proxy와2px 관용 실루엣은 다른 값이다. 실루엣99%를 엄격 픽셀 일치99%로 설명하지 않는다. 글꼴 획·안티앨리어싱·기존 Native 제목 차이는 남으며, 완료 범위는 표 아래/옆 본문·빈 줄·다음 단 소유다. 관계/px 검사와 직접 판독을 수치와 함께 설명한다. Studio report-only 픽셀 비교12건 미달도 같은 범위로 유지한다.

대표 asset은 `mydocs/pr/assets/pr7699/{native,wasm}-standard-{review,overlay}.png`와 `{native,wasm}-narrow85-review.png`다. merge 뒤 실제 SHA를 넣어 아래 형식의 review·standalone overlay 이미지를 표시한다.

`https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr7699/native-standard-review.png`

게시 초안은 다음 내용을 포함한다.

> rhwp 첫 기여를 보내주셔서 감사드리며 환영합니다. 다단 문서의 전폭 Square 표 아래 본문 배치를 개선해 주셨습니다. 독립 한컴 Print와 대조한 뒤 allowOverlap, 실제 옆 차선이 남는 표와 빈 줄 소유를 추가 보정했습니다. 기존 검사 기대값을 낮추지 않고 원 기여와 보정을 분리했습니다. 통합 PR의 최신 CI와 전쪽 Native/fresh WASM·전체 회귀 결과를 확인해 머지했습니다. 남은 제목 글꼴 차이와 관련 이슈 전체가 해결된 것은 아니라는 점도 함께 안내드립니다. contributor fork branch는 유지했습니다. 다음 기여도 환영합니다.

실제 통합 PR·merge SHA·정확한 CI URL·본문 TSV·시각 가이드·PNG를 위 문안에 연결한다. asset의 devel 존재·merge SHA 포함을 확인한 뒤에만 UTF-8 LF 파일을 --body-file로 게시하고 API로 한글·본문·이미지를 재조회한다. 원 PR은 통합 PR을 링크해 superseded close한다. #7548은 전체 해결로 닫지 않는다.
