# Task M100-1097 Stage 1 작업 기록

## 1. 목표

Issue #1097:

```text
[hwpx2hwp] aift 목차 마커와 페이지 표기 저장 누락
```

`samples/hwpx/aift.hwpx`를 HWP로 저장했을 때 목차 영역에서 마커와 `(페이지 표기)`가
한컴 에디터에서 정상 출력되지 않고 마지막 `)`만 남는 문제를 추적한다.

## 2. 판정 파일

| file | 한컴 판정 유형 | 목차 마커/`(페이지 표기)` 출력 | 메모 출력 | 표/페이지 배치 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task1097/stage1_toc_marker_trace/fixed_aift.hwp` | 성공 | 성공 | 성공 | 성공 | 성공 | HWP5 inline tab payload 수정 후보 |

## 3. 핵심 원인

rhwp 텍스트 추출 결과에는 `(페이지 표기)` 문자열이 존재했다.
따라서 이 문제는 IR에서 문자열이 사라진 것이 아니라, 한컴 에디터가 HWP5 `PARA_TEXT` 안의
인라인 탭 확장 payload를 다르게 해석하면서 뒤쪽 문자열을 비정상 처리하는 문제로 판단했다.

문제가 되는 HWPX 원본 구조:

```xml
<hp:tab width="17283" leader="3" type="2"/>
```

이 값은 HWP5 `PARA_TEXT`의 탭 확장 7-word payload에서 다음처럼 저장되어야 한다.

```text
[17283, 0, 0x0203, 0, 0, 0, 9]
```

기존 구현은 다음처럼 저장했다.

```text
[17283, 3, 0x0200, 0x20, 0x20, 0x20, 9]
```

즉 `leader`를 별도 word에 저장하고, 사용하지 않는 필드에 space 값을 채우고 있었다.
정답 HWP는 `type`과 `leader`를 하나의 word에 합쳐 `(type << 8) | leader`로 저장한다.

## 4. 정답지 비교

첫 목차 항목 `1-1. 개발 대상 기술·제품의 개요`의 `PARA_TEXT` 비교:

| file | PARA_TEXT hash | 탭 확장 payload |
|---|---|---|
| oracle `samples/aift.hwp` | `a1a69bbea59efa2a` | `83 43 00 00 03 02 00 00 00 00 00 00 09 00` |
| generated before | `f03ea4e314c18f79` | `83 43 03 00 00 02 20 00 20 00 20 00 09 00` |
| fixed candidate | `a1a69bbea59efa2a` | `83 43 00 00 03 02 00 00 00 00 00 00 09 00` |

수정 후보의 `PARA_TEXT` payload는 정답지와 동일해졌다.

Trace 산출물:

```text
output/poc/hwpx2hwp/task1097/stage1_toc_marker_trace/oracle_toc_dev_target_anchor_trace.md
output/poc/hwpx2hwp/task1097/stage1_toc_marker_trace/generated_toc_dev_target_anchor_trace.md
output/poc/hwpx2hwp/task1097/stage1_toc_marker_trace/fixed_toc_dev_target_anchor_trace.md
```

## 5. 구현 내용

수정 위치:

```text
src/parser/hwpx/section.rs
```

적용한 contract:

```text
1. width는 ext[0]에 저장한다.
2. leader는 별도 word에 저장하지 않는다.
3. type과 leader를 ext[2]에 `(type << 8) | leader`로 저장한다.
4. ext[3], ext[4], ext[5]는 0으로 둔다.
5. ext[6]은 HWP5 인라인 탭 종료 marker인 9를 저장한다.
```

추가 테스트:

```text
test_parse_hwpx_tab_extension_uses_hwp5_inline_format
```

## 6. 검증

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

## 7. 시각 판정

작업지시자가 다음 파일을 한컴 에디터에서 시각 판정했다.

```text
output/poc/hwpx2hwp/task1097/stage1_toc_marker_trace/fixed_aift.hwp
```

판정 결과:

```text
시각 판정 통과
```

확인 의미:

```text
1. 목차에서 마커와 `(페이지 표기)`가 정답 HWP처럼 출력된다.
2. 마지막 `)`만 남는 현상이 사라졌다.
3. #1092 메모 출력이 유지된다.
4. #1094 표/페이지 배치가 유지된다.
```
