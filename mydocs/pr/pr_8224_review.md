# PR #8224 리뷰 — 편집 후 탭 확장 정보 보존

## 최종 판정

**머지 보류 — 검증 진행 중.** 최신 devel에서의 재현, 실제 편집·저장·재열기 및 Native/fresh WASM 시각 검증을 완료한 뒤 판정한다.

## 접수 정보

- 작성자: `chrisryugj`, CONTRIBUTOR, Open, non-Draft. Reviewer: `edwardkim`.
- 관련 이슈: #8222. 원 head: `caabfdc2c4568f8f151c79d9d86c8df56779c52a`.
- 비교 기준: `upstream/devel` `55f40ecd99485a1c9b1f7cdc709cb6efcb9e9b29`.
- 검토 브랜치: `review/integration-20261031-chrisryugj-storage`.
- #8224 통합: `c85560c786c61c5e8a93929c22bc0e9c0936b8b1`.
- #8233 누적 통합 후보: `86f4879934d1fc903b8497d011fdd7db9dede38f`.
- 작업지시자는 helper 추가 위치 충돌을 메인터너 보정으로 해결하도록 승인했다. 최신 devel의 묶음 빈칸 복사와 원 PR의 탭 개수 helper를 모두 유지했다.
- 이번 기여자에 대한 원격 push·새 PR·댓글·머지는 아직 승인받지 않았다.

## 검토 범위와 선행 해결 이력

#8222는 실제 탭 문자와 `tab_extended`의 순서 대응이 편집 중 깨져 저장 폭이 다른 탭에 붙는 문제다. 기존 #7344 → #7628 → 통합 #7832(`bfa28a77a`)의 저장 탭 폭 소비 기능을 유지하면서, 삽입·삭제·분할·병합 때 데이터 소유를 함께 옮긴다. 새 탭에는 명시적인 자리표를 넣어 기존 TabDef 계산을 사용한다. 문서별 좌표 보정은 추가하지 않는다.

최신 devel의 #8389 묶음 빈칸 복사와 #8174/#8360의 공통 Storage 계약도 확인한다. 원 PR의 문단 병합은 직접 `collect`·`clone`·`resize`로 탭 배열을 추가하므로, 메인터너 후보에서는 문자열 순회로 개수를 세고 추가 배열 공간을 전달된 Storage로 예약한다. 별도 레이아웃 규칙은 변경하지 않는다.

## 검증 입력과 진행 기록

- `samples/lseg-05-tab.hwp`: 실제 한컴 저장본, Hancom Office 2010. 기존 독립 기준 `pdf/lseg-05-tab-hwp-2020.pdf`.
- 기본 편집 경로: WASM `insertText`, `deleteText`, `splitParagraph`, `mergeParagraph` → HWP 저장 → 재열기.
- 기본 회귀 대조: 원 PR의 5개 계약 검사를 고정 devel과 누적 후보에서 실행한다. 별도 worktree와 공용 `target/pr-review`를 사용하며 Cargo는 순차 실행한다.
- 고정 devel 결과: 1개 통과, 삽입·삭제·분할·병합 4개 실패. 원 문제는 최신 devel에서도 남아 있다.
- base WASM에서 실제 탭 문서 편집·저장·재열기를 실행했다. 텍스트와 1쪽은 보존되지만, 이것만으로 탭 소유·시각 통과를 판정하지 않는다.
- 로그와 진단 산출물: `output/pr-review/chrisryugj-20261031/`. 최종 시각 증거는 정식 경로에 보존한 뒤 종료 정리한다.

현재 원 PR CI의 성공은 참고 정보이며, 메인터너 후보의 검증 완료를 대신하지 않는다. 보정 후보의 집중 검사 5개는 통과했다. 전체 필수 회귀·lint, 새 WASM 빌드 및 편집 후 독립 Print 대조는 진행 중이다.

공용 target의 첫 후보 실행은 오래된 base rlib를 재사용해 대조 증거에서 제외했다. 후보 `src/lib.rs`의 수정 시각을 갱신해 실제 재컴파일을 확인한 뒤 5개 통과를 기록했다. 캐시 전체를 지우거나 기대값을 바꾸지 않았다.

## 사양과 실제 소비 경로

HWP 5.0 revision 1.3 표 6의 코드 9는 inline 탭이며 8개 WCHAR 슬롯을 차지한다. 원본 PARA_TEXT에서 읽은 7개 확장 word는 탭 문자 순서에 대응한다. `Paragraph` 편집 → `composer/composition.inc.rs`의 저장 줄 유효성 판정 → `layout/text_measurement.rs`의 탭 종류·폭·리더 소비 → paint 순서를 확인했다. 저장 줄이 없는 재조판 경로는 기존 규칙대로 TabDef를 사용하며, 이번 PR은 그 판정을 완화하지 않는다. 저장 후 다시 열린 문서와 편집 직후 화면을 각각 비교한다.

2026-10-31 새 한컴 2020 Print 작업 `81d841c7-2ac6-4897-8927-74fe1d6e348c`는 버전 `11.0.0.9136`, `pdf_print_method: 0`, `hancom2020_pdf_driver_one_up`으로 완료했다. 새 출력의 96 DPI 래스터는 기존 기준 PDF와 픽셀 단위로 동일해 기존 PDF를 재사용한다.
