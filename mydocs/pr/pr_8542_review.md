# PR #8542 검토 — RowBreak 저장 재시작과 이어받기 원점

## 최종 판정

**머지 보류 — 메인테이너 독립 검증 진행 중.**

원 head의 Full CI는 성공했다. 최신 devel의 재현 여부, 같은 조건의 Native/fresh WASM 전쪽 비교,
분할·이어받기 반례와 필수 편람 검증을 완료한 뒤 판정을 갱신한다. 원격 push·댓글·merge는 수행하지 않았다.

## 접수와 검토 경로

| 항목 | 값 |
| --- | --- |
| PR / 작성자 | #8542 / johndoekim, 기존 merged PR 21건 |
| 관련 이슈 | #8539, #8198의 겹침·잘림 보정 뒤 남은 쪽 경계 차이 |
| 원 head | `dc7553366f6c85035d510eaeb50eff2f25d83242` |
| 원 제품 수정 | `1543ae62fe07137da39bbaa88aa82884cc084bf2` |
| 검토 기준 devel | `ece9c3383bb2c73e0f055459ede727a9d3101ac6` |
| 로컬 병합 시뮬레이션 | `5df48e77e7fc249feb6eacda74263ed96375cd36`, tree `1e239d728ed6774aabf00e8f3f15623fef335eae`, 충돌 없음 |
| 작업 브랜치 | `review/pr-8542-rowbreak-stored-restart` |
| reviewer | `edwardkim`, 지정 후 REST API 재확인 |
| 원격 상태 | Open, non-Draft, devel 대상. head와 mergeability는 원격 조치 직전 재확인 |

base route: `maintainer_general.md`.
modifiers: `intake_and_review.md`, `local_validation.md`, `visual_fixture_evidence.md`,
`rework_and_exceptions.md`(오래된 기준), `post_merge.md`.
loaded documents: `pr_review_workflow.md`, `pr_review/README.md`와 위 기본·보조 문서.
공통 빌드·GitHub 운영·문서 절차와 Visual Sweep/출력 백엔드 절차를 함께 적용한다.

첫 기여자 경로는 비해당이다. 원 기여 커밋을 보존하고, 로컬 병합 커밋은 검토 시뮬레이션으로만 사용한다.

## 범위와 선행 해결 근거

국가법령정보센터 공개 HWP의 18행×3열 RowBreak 표에서 두 경계의 줄 소속을 확인한다.
독립 Hwp 2020 PDF는 1→2쪽의 9행을 `[2,3,3]`, 2→3쪽의 16행을 `[10,8,10]`으로 나눈다.
대응 원본·PDF는 원 PR의 `samples/issue8539/`에 이미 커밋됐으며 실행 파일과 SHA-256이 일치한다.

- HWP: `53d7763a0e5c9860e62c8dc3b20cfb515aac6f6eceeeef01ff0fb1abbdfe8fef`.
- PDF: `a54a96205d1916b44d98a71caf6114d47ddb34ebfa9f34263228f875e90e5da6`, 3쪽.
- #8198의 최초 겹침·잘림과 이번 쪽 경계 위치 차이를 구분한다. #8539 본문과
  [메인테이너 의견](https://github.com/edwardkim/rhwp/issues/8198#issuecomment-6579028228)을 대조했다.
- #7870의 원 기여 `717679359e6e128f26610727cf7c04d2f8ac6d5f` → 통합 `bf4fc67e54b1cf8a370de1780cf34898faf19fee`는
  저장 재시작의 흡수와 용량 컷이 같은 점유 높이를 소비하도록 중첩 표 atom의 이송 여백을 보정했다.
  이번 수정은 끝 줄간격과 이어받기 원점의 남은 차이를 다룬다.
- #7965 → 통합 #8148, merge `2e087a5b4bc1515ad877e8020b30114639a59392`의 저장 프레임 소유 계약은 유지해야 한다.
  [기존 검토](archives/pr_7965_review.md)와 실제 source-frame 호출 경로를 대조한다.

## 검증 계획과 CI 재사용

원 head `dc7553366f6c85035d510eaeb50eff2f25d83242`의
[Full CI](https://github.com/edwardkim/rhwp/actions/runs/41643028246)는 성공했다.
CodeQL·Render Diff·Adapter·Proptest 및 Full 정책 집계도 성공 상태를 조회했다.
현재 source/test/fixture/workflow 보정을 추가하지 않았으므로, 정확한 기존 head의 광범위 전체 회귀를
중복 실행하지 않는 정본 3.2.2를 적용한다. 집중 회귀·실제 Native/fresh WASM과 최신 base 비교는 별도로 수행한다.
보정이 필요해지면 이 재사용 판정과 필요한 새 head 검증 범위를 다시 정한다.

실행 로그·중간 산출물은 기본 작업공간 `output/pr-review/pr8542/`, 공용 Cargo target은
`/home/edward/mygithub/rhwp/target/pr-review`다. 검증 완료 수치와 직접 판독 결과는 실행 뒤 기록한다.

## 원격 승인과 후속 처리

현재 승인은 접수·로컬 검토 범위다. 검증한 후보·본문·댓글 초안과 정확한 실행 범위를 준비한 뒤
원격 push·merge·후속 댓글 및 운영 기록 반영 승인을 별도로 요청한다.
시각 증적은 [Visual Sweep 정본](../manual/verification/visual_sweep_guide.md#github-merge-comment)에 따라
직접 판독한 PNG와 전쪽 TSV를 보존한다. merge 뒤 댓글은 실제 merge SHA의 안정 URL을 사용한다.

## 독립 검토 중간 결과와 보완 범위

2026-11-05에 현재 devel과 원 기여를 병합한 실행 후보에서 다음 결과를 확인했다.

- 원 사례의 Native 3쪽은 모두 raw 100.00000%, fresh WASM Canvas는 100.00000 / 100.00000 / 99.99607%다.
  기준 devel의 Native 최저 57.29471%, WASM 최저 57.33608%에서 개선됐다.
- 실제 `[2,3,3]`·`[10,8,10]` 컷과 앞뒤 줄·행, 마지막 내용·표 하단을 양쪽 3쪽의 review PNG에서 판독했다.
- 반례 6개 193쪽씩의 Native·Canvas PNG가 모두 기준과 같고, Native 페이지 구조와 렌더 트리도 같다.
- 공개 샘플 1,494개 비교에서 정상적으로 열린 1,490개 중 이번 문서 1개만 구조가 바뀌었다.
  나머지 1,489개는 동일하다. 암호 입력이 필요한 3개와 90초 제한에 도달한 대형 표 1개는 양쪽 동일 실패다.
- 필수 편람 Native 383쪽의 최저 raw는 99.96465%, 쪽수는 383=383, #8389 기준 하락은 0건이다.

제품 로직과 원 기여의 assertion은 보정할 결함을 찾지 못했다. 다만 신규 회귀의 첫머리 측정 근거와
정답지 쪽수 원장 등록을 보강한다. 독립 자료에서 확인한 3쪽을 사용하며 기존 기대값·허용치는 바꾸지 않는다.
Rust test 주석·baseline 변경이 생기므로 초기 exact-head CI 재사용만으로 새 후보를 완료 처리하지 않고,
현행 4.3의 lint·집중/전체 회귀·Native 검사를 새 후보에서 순서대로 실행한다. 실행한 제품 source tree는
`ceaa7c40545ca8036c71c880b8d4a8c447435a4e`이며 이 보완에서 변경하지 않는다.
