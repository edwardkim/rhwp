# PR #1923 검토 — Issue #1915: HWP3-origin HWP5 저장 시 page_def(용지·여백) 소실 수정

- 작성일: 2026-07-05 / 검토자: Claude (메인테이너 대행 검토)
- PR: planet6897 → devel / MERGEABLE, CI 9 pass / 시간순 1번째 (#1923 00:16 → #1924 00:20)
- 연결 이슈: #1915

## 1. PR 요약

HWP3 원본을 HWP5로 저장하면 섹션 page_def가 0으로 기록되어 재로드 시 용지 0×0이 되는
데이터 손실 수정. 근본 원인은 IR 계약 공백 — HWP3 파서는 `section.section_def`만 채우고
첫 문단에 `Control::SectionDef`를 넣지 않는데, HWP5 직렬화기는 그 컨트롤이 있어야
secd/PAGE_DEF 계열 레코드를 방출한다. hwpdocs 10k 서베이 hwp5-roundtrip IR_DIFF 53건 중
**41건이 이 클래스** (전건 HWP3-origin).

수정: `serialize_section` 진입에서 첫 문단에 SectionDef가 없으면 삽입한 **사본**으로
직렬화 (+34줄) + 신규 게이트 `tests/issue_1915_hwp3_pagedef.rs` + fixture 승격(2.4KB).

## 2. 코드 검토

- **어댑터와 동일 계약 검증**: hwpx_to_hwp 어댑터의 `insert_section_def_control`(한컴 인식
  검증 경로)과 같은 방식(controls.insert(0, SectionDef), 컨트롤 문자 무조작) — 검증된
  계약의 직렬화기 진입 적용이라 신뢰 가능.
- **가드 3중**: ① raw_stream 보존 경로는 상류 조기 반환으로 불변 ② 기존 SectionDef 보유
  IR(HWP5/HWPX-origin) 불변 ③ `has_real_page_def`(용지 >0)일 때만 보강 — 합성/부분 IR
  (유닛테스트 fixture)에 무의미한 secd를 주입하지 않음. 첫 문단만 클론(IR 원본 불변).
- 잔여 관찰(비차단): 41건 중 3건은 diff=0인데 쪽수 +1 — HWP3원본↔변환본 렌더 경로
  이원화(#1637 계열)의 별개 축으로 컨트리뷰터가 분리 기록. 용지 0×0 붕괴 대비 순개선.

## 3. 게이트 결과 (devel `5d5a635e` + #1923 + #1924 결합)

| 게이트 | 결과 |
|---|---|
| GitHub CI | 9 pass / 1 skip |
| cargo fmt --check | 통과 |
| cargo clippy --profile release-test --all-targets | 경고 0 |
| cargo test --profile release-test --tests (신규 핀 + roundtrip baseline 포함) | **2,878 통과 / 실패 0** |
| fixture 단독 재현 | CLI hwp5-roundtrip은 #1922 스니핑으로 **FORMAT_SKIP**(HWP3 실체, 설계 의도) — 수정 경로는 자기 핀(parse→serialize→reparse)이 직접 검증 |

- OVR/시각 판정 미적용 — 직렬화기 전용(렌더 경로 무접촉). 리팩토링 v2 §1 금지 목록과도
  무관(분기 접촉 없음, HWP3-origin 예외 경로 신설이 아니라 IR 계약 폴백).

## 4. 판단 (작업지시자 승인 대기)

- 수정 건전, 서베이 41건 해소 + 최소 fixture 게이트. #1924와 파일 겹침 0, 독립 머지 가능.
- 이슈 #1915: 본문에 close 선언 없음 — 잔여 3건(쪽수 +1)은 별개 축으로 기록되어 있어
  close 여부는 작업지시자 판단.
