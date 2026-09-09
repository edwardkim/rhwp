# #6856 재착수 Stage 5 — 한컴 손상 판정과 0문단 거부

- 날짜: 2026-09-08, Issue: #6856, 브랜치: `task_m100_6856_baseline`.
- 메인테이너 지시: **“한컴에서와 동일하게 손상된 처리로 판정해야 합니다.”**
- 결정 기록: `e6fdd47f8`, [계획 §6.5](../plans/task_m100_6856_restart_impl.md#65-한컴-손상-판정에-따른-0문단-처리-확정).
- 선행 관찰: [Stage 4](task_m100_6856_restart_stage4.md).

## 구현

HWP 사각형 SHAPE_COMPONENT의 직접 소유 LIST_HEADER가 문단 수 0을 선언하면
`BodyTextError::DrawingTextStructure`를 반환한다. 기존 문서 열기 오류 전달 경로를 사용하여
빈 섹션이나 일반 사각형으로 복구하지 않는다. 일반/lenient CFB 경로에 같은 계약을 적용한다.

| 상태 | 결과 |
| --- | --- |
| 내부 목록 자체가 없음 | 일반 사각형으로 읽기 허용 |
| 목록 안에 빈 문단 1개가 있음 | 글상자로 읽기 허용 |
| 사각형 소유 목록이 문단 수 0을 선언 | 손상 입력으로 거부 |

범위는 확인된 **HWP 사각형의 소유 목록**이다. HWPX·표·캡션이나 모든 포맷의 0 개수 필드에
일괄 적용하지 않았다. 모델 객체 자체의 구조 질의와 외부 파일의 유효성 검사는 별개다.
따라서 메모리상의 `Some(TextBox)` 구조 식별 시험은 유지하며, 그것으로 0문단 HWP 입력을
허용하는 것은 아니다.

### 문단 수 필드 폭 정정

선행 검증기는 하위 16비트만 읽고 있었다. 반면 `src/parser/control/shape.rs`의 그리기 목록 해석과
`src/serializer/control.rs`의 `serialize_text_box_if_present`는 UINT32를 사용한다.
0 판정을 도입할 때 65,536 같은 양수를 0으로 오인하지 않도록 검증기도 같은 32비트 필드를 읽게 했다.
공통 LIST_HEADER의 일반 규격을 바꾼 것이 아니라 해당 그리기 reader/writer와 일치시킨 것이다.
양수 65,536 선언에 실제 문단이 없는 시험은 '0문단'이 아니라 '65,536개 중 누락' 오류로 판정한다.

## 검증 증적

- 수정 전 0문단 전용 시험 2개 실패: section·문서 열기가 손상 입력을 허용했다.
- 수정 후 결과는 아래 실행 완료 기록에 남긴다.
- 새 시험은 `tests/cases/issue_6856_drawing_structure_errors.rs`에 추가했다.
  직접/그룹 소유 0문단, 일반/lenient CFB, DocumentCore 오류 전달, UINT32 개수 보존을 검사한다.
- 정상 대조군은 기존 소유권 시험의 일반 사각형·빈 글상자와 실제 편람/A4 쌍을 재사용한다.
  새 정상 샘플이나 10k 전수 계측을 만들지 않는다.

### 메인테이너가 확인한 실제 시험본

- `output/6856/zero-paragraph/a4-zero-paragraph.hwp`
- SHA256: `075b6fe46ddf5926505e98e49d92c14692fb3130e08db182f3b4de82a1b87e76`.
- 파일을 수정하거나 재저장하지 않고 새 코드의 문서 파서와 `DocumentCore::from_bytes`로 확인한다.
- 로컬 확인 도구: `output/6856/zero-paragraph/check-rejection.rs`.

## 실행 완료 기록

- 최종 release-test 라이브러리 빌드 통과.
- 최종 라이브러리에 재링크한 집중 시험 **30개 통과**:
  구조 오류 11, 빈 문단 1, 소유권 3, 구조 식별 5, 출력 부호 4, 기존 #6852 선 회귀 6.
- 메인테이너 확인 시험본은 다음 구조 오류를 반환했다:
  `rectangle at level 2 declares 0 paragraphs, found 0`.
  `parse_document`가 `DrawingTextStructure` 오류를 반환하고 `DocumentCore::from_bytes`도 거부했다.
- 정상 편람 HWP/HWPX 모두 본문 내부 영역 136개 + 바탕쪽 43개 = **179개 유지**.
  사각형 기하 중 일반 사각형 60개, 내부 영역 있는 사각형 177개를 유지했다.
- 정상 A4 HWP/HWPX 모두 일반 사각형 1개·글상자 2개 유지. 원본/손상 시험본을 재저장하지 않았다.
- 위 시험은 focused rustc 경로이며 generated integration suite 전체 실행은 아니다.
- `cargo clippy --locked -p rhwp --lib --profile release-test --target-dir
  /home/edward/mygithub/rhwp-6812-review-target -- -D warnings` 통과.
- 변경 Rust 파일의 rustfmt 검사와 `git diff --check` 통과.

## 다음 절차

0문단 처리 방침은 더 이상 승인 대기 사항이 아니다. 본 절편의 결과를 승인받은 뒤
남은 통합 검증과 최종 보고·PR 준비 절차로 진행한다. 로컬 집중 검증은 전체 Rust
native/WASM/workspace 게이트를 대체하지 않는다. 실행 중인 Studio WASM은 별도 빌드·교체가 필요하며
이번 절편에서 원격 push·PR 생성·이슈 종료를 수행하지 않는다.
