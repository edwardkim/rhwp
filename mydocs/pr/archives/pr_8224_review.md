# PR #8224 리뷰 — 편집 후 탭 확장 정보 보존

## 최종 판정

**메인터너 보정 후 수용 가능 — #8222 해결 범위의 로컬 검증 완료.** 탭 편집·저장 소유를 보존하고 최신 Storage·묶음 빈칸 복사 계약을 유지했다. 통합 PR [#8395](https://github.com/edwardkim/rhwp/pull/8395)로 머지했고, 원 PR도 MERGED다. merge `21ef5020f763e474ff652df5c2c9adf94efededc`.

## 접수 정보

- 작성자: `chrisryugj`, CONTRIBUTOR, Open, non-Draft. Reviewer: `edwardkim`.
- 관련 이슈: #8222. 원 head: `caabfdc2c4568f8f151c79d9d86c8df56779c52a`.
- 비교 기준: `upstream/devel` `55f40ecd99485a1c9b1f7cdc709cb6efcb9e9b29`.
- 검토 브랜치: `review/integration-20261031-chrisryugj-storage`.
- #8224 통합: `c85560c786c61c5e8a93929c22bc0e9c0936b8b1`.
- #8233 누적 통합 후보: `86f4879934d1fc903b8497d011fdd7db9dede38f`.
- 작업지시자는 helper 추가 위치 충돌을 메인터너 보정으로 해결하도록 승인했다. 최신 devel의 묶음 빈칸 복사와 원 PR의 탭 개수 helper를 모두 유지했다.
- 작업지시자가 원격 push·Open PR·CI 성공 뒤 self-review·머지·후속 댓글을 승인했다.

## 검토 범위와 선행 해결 이력

#8222는 실제 탭 문자와 `tab_extended`의 순서 대응이 편집 중 깨져 저장 폭이 다른 탭에 붙는 문제다. 기존 #7344 → #7628 → 통합 #7832(`bfa28a77a`)의 저장 탭 폭 소비 기능을 유지하면서, 삽입·삭제·분할·병합 때 데이터 소유를 함께 옮긴다. 새 탭에는 명시적인 자리표를 넣어 기존 TabDef 계산을 사용한다. 문서별 좌표 보정은 추가하지 않는다.

최신 devel의 #8389 묶음 빈칸 복사와 #8174/#8360의 공통 Storage 계약도 확인한다. 원 PR의 문단 병합은 직접 `collect`·`clone`·`resize`로 탭 배열을 추가하므로, 메인터너 후보에서는 문자열 순회로 개수를 세고 추가 배열 공간을 전달된 Storage로 예약한다. 별도 레이아웃 규칙은 변경하지 않는다.

## 검증 입력과 진행 기록

- `samples/lseg-05-tab.hwp`: 실제 한컴 저장본, Hancom Office 2010. 기존 독립 기준 `pdf/lseg-05-tab-hwp-2020.pdf`.
- 기본 편집 경로: WASM `insertText`, `deleteText`, `splitParagraph`, `mergeParagraph` → HWP 저장 → 재열기.
- 기본 회귀 대조: 원 PR의 5개 계약 검사를 고정 devel과 누적 후보에서 실행했다. 별도 worktree와 공용 `target/pr-review`를 사용하며 Cargo는 순차 실행한다.
- 고정 devel 결과: 1개 통과, 삽입·삭제·분할·병합 4개 실패. 원 문제는 최신 devel에서도 남아 있다.
- base WASM에서 실제 탭 문서 편집·저장·재열기를 실행했다. 텍스트와 1쪽은 보존되지만, 이것만으로 탭 소유·시각 통과를 판정하지 않는다.
- 로그와 진단 산출물: `output/pr-review/chrisryugj-20261031/`. 시각 증거는 아래 정식 경로에 보존했고, 처리 종료 시 임시 산출물을 정리한다.

원 PR CI의 성공은 참고 정보이며 메인터너 후보의 검증을 대신하지 않는다. 원 PR의 집중 검사 5개는 실제 재컴파일한 후보에서 통과했다. 실제 편집 경로·시각 결과와 공통 검증 결과는 아래에 구분한다.

공용 target의 첫 후보 실행은 오래된 base rlib를 재사용해 대조 증거에서 제외했다. 후보 `src/lib.rs`의 수정 시각을 갱신해 실제 재컴파일을 확인한 뒤 5개 통과를 기록했다. 캐시 전체를 지우거나 기대값을 바꾸지 않았다.

## 사양과 실제 소비 경로

HWP 5.0 revision 1.3 표 6의 코드 9는 inline 탭이며 8개 WCHAR 슬롯을 차지한다. 원본 PARA_TEXT에서 읽은 7개 확장 word는 탭 문자 순서에 대응한다. `Paragraph` 편집 → `composer/composition.inc.rs`의 저장 줄 유효성 판정 → `layout/text_measurement.rs`의 탭 종류·폭·리더 소비 → paint 순서를 확인했다. 저장 줄이 없는 재조판 경로는 기존 규칙대로 TabDef를 사용하며, 이번 PR은 그 판정을 완화하지 않는다. 저장 후 다시 열린 문서와 편집 직후 화면을 각각 비교한다.

2026-10-31 새 한컴 2020 Print 작업 `81d841c7-2ac6-4897-8927-74fe1d6e348c`는 버전 `11.0.0.9136`, `pdf_print_method: 0`, `hancom2020_pdf_driver_one_up`으로 완료했다. 새 출력의 96 DPI 래스터는 기존 기준 PDF와 픽셀 단위로 동일해 기존 PDF를 재사용한다.

## 시각 검증과 실제 편집 결과

[정식 증거](../assets/chrisryugj_20261031/README.md)에 원본·삽입·삭제·분할·재병합의 각 1쪽 전체
Native/fresh WASM TSV, 원 PNG, 같은 영역의 확대 판독과 한컴 Print 출처를 보존했다.
전쪽 최저는 Native 98.04650%, 재열기 WASM 97.89252%이며 같은 입력의 devel 대비 하락은 0이다.
실제 편집 직후 Canvas는 삽입에서 97.32876% → 97.89491%로 개선되고 나머지 세 동작은 유지됐다.
탭 뒤 글자·분할 뒤 본문·후속 문단의 소유·내용 순서와 줄바꿈·간격을 직접 확인했다.

Native CLI와 WASM의 삽입 저장본은 SHA-256까지 같다. devel은 새 탭에 기존 폭을 잘못 붙이고
마지막 탭에 자리표를 썼으나, 후보는 새 탭의 자리표와 기존 두 탭의 원 확장 word를 제 순서에 쓴다.
기존 합성 계약 검사에 실제 DocumentCore 편집·저장·재열기 검사를 보강했다. 시각 검증을 마친
편집본 HWP/PDF는 `samples/pr8224/`, `pdf/pr8224/`에 보존하며 새 px golden은 추가하지 않는다.

Native 라이브러리 검사는 4,040개 통과, 기존 ignored 12개다. 기존 문단 병합 검사 안에서 작은
Storage 한도의 거부와 정상 한도의 처리를 확인했다. Native 그림 placeholder 2개, direct PDF 22개도 통과했다. Native/WASM 및 workspace all-target Clippy, workspace build, fmt, suite manifest와 unit-tier 검사를 통과했다. 전체 회귀와 신규 sample 보안 검사도 아래와 같이 통과했다.

## 동일 원본의 기존 회귀 검토

`lseg-05-tab.hwp` 참조를 추적했다. #6520의 출력 경로 간 동일성 검사는 이미 관계형이므로 유지한다.
#6800의 `a_mid_run_tab_with_text_after_it_is_untouched`는 `width > 400px` 핀을 사용하고 있었다.
원 #6801 → 메인터너 보정 `daf5744bfd4` → [기존 검토](pr_6801_review.md)와
후속 이관 `39a634e5476`을 확인했다. 이번 원본은 전쪽 Native/fresh WASM 90% 이상 및 직접 판독을
마쳤으므로 같은 함수 안에서 탭 뒤 가시문자의 문자 경계가 bbox에 포함되는 관계로 바꿨다.
공개 JSON의 0.1px 반올림 정밀도만 허용하며 고정 위치·폭을 새로 지정하지 않는다. 다른 원본의 검사는
유지한다. 새 실제 DocumentCore 검사는 편집 후 저장·재열기의 텍스트 순서와 탭 소유를 보완한다.

## 최종 공통 검증과 원격 처리 조건

- 전체 release-test: **12,207개 통과 / 실패 0 / 기존 skip 45**, 496.765초. 탭 6개·그룹 3개 포함.
- Native Skia: lib 4,040개, 그림 placeholder 2개, direct PDF 22개 통과. lib 기존 ignored 12개.
- fmt, Native/WASM Clippy, workspace build 및 all-target Clippy, suite manifest·unit-tier 모두 통과.
- 새 편집 HWP 4개를 명시한 보안·injection 검사 20개 통과. 전체 실행에서 빈 입력으로 반환된 결과와 구분한다.
- 독립 시각 증거·변경 테스트 3개 rustfmt·문서 링크·diff whitespace 확인 완료.
- [정식 게이트와 로그](../assets/chrisryugj_20261031/README.md#최종-로컬-게이트).

제품 보정 `f85410c616`, 테스트·기록 시작점 `b8c0ef2e4b`와 이 문서의 최종 커밋을 구분한다.
검증 worktree와 기본 작업공간의 제품·변경 테스트 파일은 SHA-256으로 동일하다. 실제 사용한 기존
입력은 Git blob과 대조했고 새 편집 입력·PDF도 이 검토 기록과 같은 커밋에 포함한다.

2026-10-31 확인한 devel은 `55f40ecd99485a1c9b1f7cdc709cb6efcb9e9b29`이며 두 원 head는 변경되지 않았다.
본인 계정의 열린 PR은 0개였다. 승인 뒤 통합 PR #8395를 열었고 정확한 head의 CI·CodeQL·Render Diff·Adapter·Proptest를 확인했다.
GitHub Jobs API의 단계 기록 지연으로 정책 감사가 pending에 머물렀지만, 별도 승인받은 1회 재감사
`41108714192` attempt 2에서 success를 확인했다. 제품 CI·보안 설정·required check는 변경하지 않았다.
최신 devel과 충돌 없는 merge-tree를 확인하고 exact-head match로 머지했다.

## 통합과 후속 기록

- 통합 PR: [#8395](https://github.com/edwardkim/rhwp/pull/8395), merge `21ef5020f763e474ff652df5c2c9adf94efededc`.
- 검증 head: `298d1ecc520014c2d18cf8c9016471f586967c03`, [Full CI](https://github.com/edwardkim/rhwp/actions/runs/41104868865).
- [최종 검사 상태](../assets/chrisryugj_20261031/ci-checks-final.json), [통합 결과](../assets/chrisryugj_20261031/integration-result.json).
- 원 PR #8224·#8233 모두 MERGED다. #8222는 완료로 닫혔고 #8225는 자식 보존 범위만 반영되어 OPEN을 유지한다.
- 메인터너 일반 경로의 운영 기록으로 archive 이동·오늘할일·merge 증거만 devel에 직접 반영한다.
- 후속 댓글은 한국어 존댓말로 최종 merge·CI·검토 증거를 안내한다. #8224에는 SHA 고정 비교 이미지와 한컴 Print 링크를 포함하며, #8233은 비시각적 저장 보존 범위를 명시한다.

## Merge 후 contributor PR comment 계획

원 기여의 탭 순서 보존과 메인터너의 NBSP 충돌·Storage 예산 보정을 구분한다. 원본과 네 편집본 각 1쪽의 Native 최저 98.04650%, fresh WASM 최저 97.89252%, 같은 입력의 base 대비 악화 없음을 안내한다. 삽입 1쪽의 `insert-detail.png`(Print / Native 재열기 / WASM 편집 직후)와 `pdf/pr8224/lseg-tab-insert-2020.pdf`를 merge SHA에 고정해 연결한다. 직접 판독한 내용 순서·탭 간격과 자동 수치의 한계를 적는다. 공통 로컬 회귀·Native·lint와 동일 head의 CI 성공, 실제 통합 PR·merge SHA 및 정식 검토 기록을 연결한다.
