# Task M100-1097 완료 보고서

## 1. 이슈

GitHub Issue #1097:

```text
[hwpx2hwp] aift 목차 마커와 페이지 표기 저장 누락
```

## 2. 목표

`samples/hwpx/aift.hwpx`를 HWP로 저장했을 때 한컴 에디터 목차 영역에서 마커와
`(페이지 표기)`가 정상 출력되지 않고 마지막 `)`만 출력되는 문제를 해결한다.

비교 기준:

```text
source: samples/hwpx/aift.hwpx
oracle: samples/aift.hwp
```

선행 성공 조건:

```text
#1092 메모 컨트롤 저장 성공 유지
#1094 표 높이/페이지 배치 성공 유지
```

## 3. 문제

rhwp 텍스트 추출 결과에는 `(페이지 표기)` 문자열이 존재했다. 따라서 문제는 문자열 누락이 아니라
한컴 에디터가 저장된 HWP5 `PARA_TEXT`의 인라인 탭 확장 payload를 다르게 해석하면서 뒤쪽 문자열을
비정상 처리하는 형태였다.

문제가 되는 HWPX 원본:

```xml
<hp:tab width="17283" leader="3" type="2"/>
```

## 4. 원인

기존 구현은 HWPX `leader`와 `type`을 HWP5 인라인 탭 확장 필드에 잘못 배치했다.

기존 생성 payload:

```text
[17283, 3, 0x0200, 0x20, 0x20, 0x20, 9]
```

정답 HWP payload:

```text
[17283, 0, 0x0203, 0, 0, 0, 9]
```

즉 HWP5 contract에서는 `leader`를 별도 word에 저장하지 않고, `type`과 `leader`를 하나의 word에
합쳐 `(type << 8) | leader`로 저장해야 한다.

## 5. 수정

수정 파일:

```text
src/parser/hwpx/section.rs
```

적용 내용:

```text
1. HWPX hp:tab@width를 ext[0]에 저장
2. hp:tab@leader를 별도 word에 저장하지 않음
3. hp:tab@type과 leader를 ext[2]에 `(type << 8) | leader`로 저장
4. ext[3], ext[4], ext[5]는 0으로 유지
5. ext[6]은 HWP5 inline tab 종료 marker인 9로 유지
```

추가 테스트:

```text
test_parse_hwpx_tab_extension_uses_hwp5_inline_format
```

## 6. 산출물

작업 기록:

```text
mydocs/working/task_m100_1097_stage1.md
```

판정 파일:

```text
output/poc/hwpx2hwp/task1097/stage1_toc_marker_trace/fixed_aift.hwp
```

핵심 trace:

```text
output/poc/hwpx2hwp/task1097/stage1_toc_marker_trace/oracle_toc_dev_target_anchor_trace.md
output/poc/hwpx2hwp/task1097/stage1_toc_marker_trace/generated_toc_dev_target_anchor_trace.md
output/poc/hwpx2hwp/task1097/stage1_toc_marker_trace/fixed_toc_dev_target_anchor_trace.md
```

## 7. 검증

실행한 검증:

```text
cargo test test_parse_hwpx_tab_extension_uses_hwp5_inline_format
cargo fmt --check
cargo check
cargo build --bin rhwp
git diff --check
```

결과:

```text
success
```

정답지/기존 생성/수정 후보의 첫 목차 항목 `PARA_TEXT` 비교:

| file | PARA_TEXT hash | 탭 확장 payload |
|---|---|---|
| oracle `samples/aift.hwp` | `a1a69bbea59efa2a` | `83 43 00 00 03 02 00 00 00 00 00 00 09 00` |
| generated before | `f03ea4e314c18f79` | `83 43 03 00 00 02 20 00 20 00 20 00 09 00` |
| fixed candidate | `a1a69bbea59efa2a` | `83 43 00 00 03 02 00 00 00 00 00 00 09 00` |

## 8. 시각 판정

작업지시자 판정:

```text
시각 판정 통과
```

확인 의미:

```text
1. 목차 마커와 `(페이지 표기)`가 정답 HWP처럼 출력됨
2. 마지막 `)`만 남는 현상이 사라짐
3. #1092 메모 출력 유지
4. #1094 표/페이지 배치 유지
```

## 9. 결론

#1097은 HWPX `hp:tab`의 `leader/type`을 HWP5 인라인 탭 확장 payload로 잘못 materialize하던 문제였다.

해결의 핵심은 다음 contract다.

```text
hp:tab@type + hp:tab@leader -> tab_extended[2] = (type << 8) | leader
```

Stage 1 판정으로 `aift.hwpx` 목차의 마커와 `(페이지 표기)` 저장 누락 문제는 완료 처리할 수 있다.
