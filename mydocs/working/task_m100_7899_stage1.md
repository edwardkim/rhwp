# #7899 — 구역 끝 목록 보존 구현·집중 검사

상위 계획: [수행](../plans/task_m100_7899.md), [구현](../plans/task_m100_7899_impl.md). 메인터너의 2026-10-14 “계획 승인·구현 진행”에 따른 단계 1 기록이다. 최종 PR·시각 통과 보고가 아니다.

## 구현과 실제 소비 경로

| 계약 | 생산 → 저장 소비 |
| --- | --- |
| 구역 끝 목록의 독립 소유 | `parser/body_text.rs::parse_body_text_section`이 마지막 root 문단 뒤 level 1 LIST_HEADER/MEMO_LIST부터 원본 순서의 레코드를 `Section.trailing_lists`에 보관한다. 본문 문단 파싱 범위에서 제외한다. `serializer/body_text.rs::serialize_section_inner`는 본문을 새로 쓴 뒤 이 범위를 한 번만 출력하며 typed master/memo 생성과 동시에 쓰지 않는다. |
| 원본 재사용의 수명 | `model/raw_provenance.rs::section_model_digest`는 tail 원본도 포함한다. 로드 fixup 뒤 `seal_body_raw_provenance`가 현재 바탕쪽·메모 참조 투영을 봉인한다. `serializer/cfb_writer.rs::serialize_hwp_inner`는 공개 모델 변경도 검사한 뒤 일반·비밀번호·보고서 저장 경로로 들어간다. 본문 글자·캐럿/범위 이동은 봉인 대상에서 제외한다. |
| 낡은 목록의 강제 재생 방지 | 바탕쪽 모델 또는 MEMO 필드 자체가 달라지면 `UnsupportedInput`을 반환한다. 아직 모델링하지 못한 원본 목록을 삭제하거나 수정 내용을 무시해 성공시키지 않는다. 새 메모 UI·Native 메모 목록 자체 편집은 완료 범위가 아니다. |
| Native 메모 정체성 | 원본 본문 시작 슬롯은 `%%me`인데 CTRL_HEADER는 `%unk`일 수 있다. `control_char_code_and_id → field_end_marker`가 시작/끝의 메모 식별을 공유하고 CTRL_HEADER 원본 정체성과 번호·command는 보존한다. 단순 tail 복원만으로는 한컴 재저장 때 메모가 소실된 실제 반례에서 발견한 추가 수정이다. |
| 원본 없는 생성 경로 | `trailing_lists=None`인 HWPX는 기존 memo materialization·master-page writer를 그대로 소비한다. HWPX 다중 secd의 물리 분리에서는 마지막 조각에만 tail을 전달한다. Native HWP5는 그 분리 분기 대상이 아니며 그 경로의 시각 검증을 주장하지 않는다. |

HWP3/HML 및 기존 합성 Section의 23개 명시적 초기화에는 `None`만 추가했다. 기존 source-side 검사 수·기대값·조판 조건은 바꾸지 않았다. `is_extension`·페이지네이션·renderer는 수정하지 않았다.

## 집중 실행 결과

정식 원본: `tests/cases/issue_7899_section_tail.rs`. 기존 공개 HWP/HWPX를 재사용하며 새 렌더링 fixture/golden을 추가하지 않는다.

- 기준 production source `b3eaf7b10b15a2d40810d42acdcf4de6c1ba026b`에서 같은 8개 보존 검사를 실행: **7 FAIL / 정상 대조 1 PASS**. 목록 유실·잘못된 메모 시작 슬롯·낡은 모델 저장을 실제 assertion으로 검출했다. 빌드 오류나 0 test 실행을 실패 재현으로 세지 않았다. 보정 source는 backup hash 대조 후 모두 복원했다.
- 보정 후 같은 8개 모두 PASS. HWPX→HWP 생성 정상 대조를 더한 최종 **9개 모두 PASS**. Native 메모 1·2의 번호·본문, 4/5/6/7 바탕쪽의 tag/level/payload/순서, 재편집, 다른 구역 소유, 구역 정의 보강, unsupported mutation을 검사했다.
- 기존 `issue_1391_memo_field_roundtrip` 2개 PASS: HWPX 메모 본문/parameters와 일반 hyperlink 대조.
- 새 Native CLI의 10개 입력에서 미편집/편집/재편집 저장 PASS. 미편집 구역은 전 바이트 동일, 편집한 구역만 변하며 tail 전체가 원본과 동일하다. 교육과정 샘플은 공개 main blob에서 확보한 별도 로컬 입력이다. devel에 존재한다고 보고하지 않는다.
- fmt·diff check PASS. 고정 base의 manifest policy 및 source unit tier policy PASS. 정책 결과는 전체 Rust 검사·Clippy 통과를 의미하지 않는다.

명령: `CARGO_TARGET_DIR=/home/edward/mygithub/rhwp/target/pr-review node scripts/run-rust-test.mjs issue_7899_section_tail -- --cargo-profile release-test`. 검사 로그·독립 레코드 비교·일시 source backup은 ignored `/home/edward/mygithub/rhwp/output/pr-review/issue7899-20261014/`에 보존한다. `focused-before.log`, `before-contract-status.json`, `focused-after-9.log`, `focused-hwpx-memo.log`, `candidate-final-native/record-comparison.json`이 원시 증거다. 파생 suite는 제출하지 않는다.

## 독립 실행의 중간 결과와 남은 범위

구역 끝 목록만 복구한 중간 Native 저장본은 한컴 2020에서 math_8 PDF 인쇄와 메모 문서 열기가 성공했지만, 메모 문서 재저장에서는 번호 목록이 사라졌다. 동일한 원본을 재저장한 대조는 번호 1을 유지해 환경 문제로 상쇄할 수 없었다. %unk/%%me 시작 슬롯 차이를 찾아 위 보정과 정식 검사를 추가했다. 최종 보정본의 한컴 재저장 확인은 단계 2에서 별도로 기록한다.

중간 math_8 편집본의 1쪽 독립 한컴 Print 대비 Native 실루엣은 설치 글꼴 전체 임베딩으로 **55.19024%**다. 수식 누락·간격 차이가 직접 보인다. 같은 저장본을 고정 base CLI와 중간 보정 CLI로 출력한 SVG는 byte 동일(`1c98f3de02f60ff5a8ca220a142ad44671acb0e017fcb1fb485bb07059cee6e5`)하여 이 화면 차이를 이번 수정의 새 회귀로 분류하지 않는다. **기존 차이라는 사실은 90% gate 면제나 PR 제출 승인도 아니다.** Docker fresh WASM·최종 Native 검증은 남아 있다. 임베딩 없는 52.72554% 실행은 환경 진단으로만 보존한다.

이 단계로 구현·집중 보존 계약을 확정한다. 다음은 승인된 계획 내 단계 2의 Docker WASM 저장 계약, 최종 한컴 열기/메모 보존과 영향 출력 검증이다. 전체 CI 성격의 검증, 원격 push·댓글·PR 생성·close는 수행하지 않았다.
