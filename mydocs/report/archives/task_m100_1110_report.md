# Task #1110 최종 보고서 — exam_social.hwpx HWP 저장 파일손상 판정 제거

- 이슈: [#1110](https://github.com/edwardkim/rhwp/issues/1110)
- 브랜치: `local/task1110`
- 마일스톤: M100
- 작성일: 2026-05-25

## 1. 작업 결과

`samples/hwpx/exam_social.hwpx`를 HWP로 저장했을 때 한컴 에디터에서 발생하던 파일손상/문서 변조
판정을 제거했다.

이번 이슈는 다음 범위까지 성공으로 판정한다.

```text
1. samples/hwpx/exam_social-p1.hwpx 저장본이 한컴 에디터에서 열린다.
2. samples/hwpx/exam_social.hwpx 저장본의 한컴2020 파일손상/변조 판정이 제거된다.
3. rhwp-studio reload는 정상이다.
4. HWPX 렌더링에서 구현한 문단번호/머리말/꼬리말 계약을 HWP 저장 경로에도 반영했다.
```

작업지시자 최종 결정:

```text
현재 진행된 것까지만 이번 타스크는 성공으로 판정한다.
미해결된 문제는 새로운 이슈로 등록해서 별도 공략한다.
```

## 2. 해결한 문제

정답 HWP와 생성 HWP를 비교해 다음 저장 계약을 보강했다.

```text
1. AutoNumber 뒤 fixed-width space와 PARA_RANGE_TAG 계약
2. HWPX 문단번호/쪽번호 관련 placeholder 저장 계약
3. 머리말/꼬리말 문단 스타일 저장 계약
4. 첫 문단 control code 순서 계약
5. 본문 fixed-width space의 HWP5 control mask materialization
```

중요한 전환점:

```text
Stage24:
  한컴2020의 파일손상/문서 변조 판정 제거

Stage28:
  본문 fixed-width space를 HWP5 fixed blank control contract에 맞춰 재검증
  rhwp reload와 wasm 빌드 성공
```

## 3. 미해결 분리

다음 문제는 #1110 범위에서 분리한다.

```text
현상:
  samples/hwpx/exam_social.hwpx 저장본을 한컴 에디터에서 열면
  3페이지 홀수쪽 머리말의 글상자 내부 쪽번호가 다음 줄로 밀리며
  머리말 영역 내 표와 글상자 높이가 과도하게 커진다.

관찰:
  rhwp-studio에서는 정상 조판된다.
  한컴 에디터에서 3페이지 머리말 글상자 안의 쪽번호 앞 공백/줄바꿈 상태를 수동으로 정리하면
  정상 배치로 돌아온다.
  정답 HWP와 생성 HWP의 직접적인 master page AutoNumber 문단 주변 record는 byte 수준으로 일치한다.

해석:
  rhwp-studio 렌더러가 무시하거나 의미 없이 취급하는 저장 전용 record/attribute를
  한컴 에디터가 글상자 내부 줄나눔 또는 머리말 영역 계산에 사용하고 있을 가능성이 높다.
```

따라서 후속 이슈의 핵심은 글상자 자체의 좌표 보정이 아니라, 정답 HWP와 생성 HWP의 남은
storage-only contract 차이를 좁히는 것이다.

## 4. 산출물

계획서:

```text
mydocs/plans/task_m100_1110.md
```

작업 기록:

```text
mydocs/working/task_m100_1110_stage1.md
```

주요 판정 산출물:

```text
output/poc/hwpx2hwp/task1110/stage24_first_para_control_code_order/
output/poc/hwpx2hwp/task1110/stage28_body_fwspace_contract/
```

## 5. 검증

마지막 확인:

```text
docker compose --env-file .env.docker run --rm wasm
```

결과:

```text
success
pkg/ WASM 산출물 생성 완료
```

Stage28 기준 검증:

```text
cargo fmt --check
cargo test hwp5_save_fwspace_marks_fixed_blank_control --lib
cargo build --bin rhwp
target/debug/rhwp info output/poc/hwpx2hwp/task1110/stage28_body_fwspace_contract/exam_social-p1-stage28.hwp
target/debug/rhwp info output/poc/hwpx2hwp/task1110/stage28_body_fwspace_contract/exam_social-stage28.hwp
```

결과:

```text
success
```

## 6. 후속 이슈

새 이슈로 분리할 항목:

```text
[hwpx2hwp] exam_social.hwpx 저장본의 홀수쪽 머리말 글상자 쪽번호 줄바꿈/높이 증가 수정
```

등록된 후속 이슈:

```text
https://github.com/edwardkim/rhwp/issues/1113
```

후속 이슈는 #1110에서 제거된 파일손상/변조 판정과 별개로, 한컴 에디터의 홀수쪽 머리말 글상자
조판 차이만 다룬다.
