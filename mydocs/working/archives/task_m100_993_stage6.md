# Task M100-993 Stage 6 Working Note

## 1. 목적

`mel-001.hwpx`를 HWP로 저장할 때 2페이지 첫 1x1 표의 셀 배경이 검게 나오고,
한컴 에디터에서 파일손상 판정이 발생하는 원인을 셀 HWP5 contract 관점에서 분리한다.

## 2. 생성 파일

```text
output/poc/hwpx2hwp/task993/stage6_cell_contract_probe/mel-001-current.hwp
output/poc/hwpx2hwp/task993/stage6_cell_contract_probe/mel-001-gradient-fixed.hwp
output/poc/hwpx2hwp/task993/stage6_cell_contract_probe/tb-org-02-gradient-fixed.hwp
```

## 3. 판정표

| file | 한컴 판정 유형 | 1x1 표 배경 | 표/셀 배치 | 마지막 페이지 출력 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|---|
| `output/poc/hwpx2hwp/task993/stage6_cell_contract_probe/mel-001-current.hwp` | 성공 | 실패 | 실패 | 성공 | 성공 | 기존 adapter 기준 |
| `output/poc/hwpx2hwp/task993/stage6_cell_contract_probe/mel-001-gradient-fixed.hwp` | 성공 | 성공 | 실패 | 성공 | 성공 | gradient BorderFill HWP5 contract 후보 |
| `output/poc/hwpx2hwp/task993/stage6_cell_contract_probe/tb-org-02-gradient-fixed.hwp` | 성공 | - | 한컴에서 셀내 텍스트가 한글자씩 줄나눔되어 셀 높이 증가 | 성공 | 정상 | 셀 세로 정렬 guard |

## 4. 핵심 관찰

문제가 되는 2페이지 첫 1x1 표는 section0 `body_order=17`이다.

정답지와 기존 generated HWP의 target table/cell 비교 결과:

```text
CTRL_HEADER(Table): payload hash 동일
TABLE: payload hash 동일
Cell LIST_HEADER first 34 bytes: 동일
Cell borderFillIDRef: 7로 동일
Cell text/pchar/line_seg: 동일
```

따라서 1x1 표의 검은 배경은 셀 record의 borderFill 참조값이 틀린 문제가 아니라,
참조 대상인 DocInfo `BORDER_FILL #7` payload가 HWP5 contract에 맞지 않게 저장되는 문제로 좁혀졌다.

## 5. BorderFill #7 비교

`mel-001.hwpx`의 `hh:borderFill id="7"`은 gradient fill이다.

```xml
<hc:gradation type="LINEAR" angle="0" centerX="50" centerY="50" step="50" colorNum="2" stepCenter="50" alpha="0">
  <hc:color value="#FFFFFF"/>
  <hc:color value="#D6EAFE"/>
</hc:gradation>
```

정답지/기존 생성/수정 후보의 7번째 `BORDER_FILL` 차이:

| file | BORDER_FILL #7 size | payload hash |
|---|---:|---|
| oracle `samples/hwpx/hancom-hwp/mel-001.hwp` | 71 | `blake3:8e78780657cb4b8a146d1a5d90909a3e5d9c5bff2781e703ea9e09561357eaa9` |
| current `mel-001-current.hwp` | 58 | `blake3:73bd698041b846c9a02cca32bb1047d3100300e70b82771cdf64ecc7ca20607f` |
| fixed `mel-001-gradient-fixed.hwp` | 71 | `blake3:38fa5193cbf39c53645f6af29bb3932e7893758060e021dc9dd51a6e3a827b2d` |

기존 생성 파일은 gradient를 HWP5 DocInfo binary 형식보다 짧게 저장했다.
수정 후보는 gradient payload 길이를 정답지와 같은 71바이트로 맞춘다.

## 6. 적용한 구현 후보

```text
1. HWPX gradient type 문자열을 HWP5 gradient kind로 매핑
   - LINEAR -> 1
   - RADIAL -> 2
   - CONICAL -> 3
   - SQUARE -> 4

2. HWPX gradation의 step 값을 GradientFill.blur로 매핑
3. HWPX gradation의 stepCenter 값을 GradientFill.step_center로 보존
4. HWPX gradation alpha를 Fill.alpha로 보존
5. DocInfo BORDER_FILL gradient serialization을 HWP5 parser contract와 같은 필드 폭으로 저장
   - kind: u8
   - angle/center_x/center_y/blur/count: u32
   - colors: ColorRef 배열
   - additional_size=1
   - step_center: u8
   - alpha: u8
```

## 7. 검증

```text
cargo check
```

결과:

```text
success
```

`mel-001-gradient-fixed.hwp`는 rhwp 기준 20페이지로 로드되며, 2페이지의 표/문단 목록도 로드된다.

## 8. 판정 해석

Stage 6 판정으로 `mel-001`의 2페이지 첫 1x1 표 배경 문제는 gradient BorderFill HWP5 contract
문제로 확정한다.

```text
mel-001-current.hwp:
  - 한컴에서 파일손상 없이 열림
  - 1x1 표 배경 실패
  - 표/셀 배치 실패

mel-001-gradient-fixed.hwp:
  - 한컴에서 파일손상 없이 열림
  - 1x1 표 배경 성공
  - 표/셀 배치 실패는 잔존
```

따라서 이번 수정 후보는 다음 축을 해결했다.

```text
HWPX gradation -> HWP5 BORDER_FILL gradient payload
```

아직 해결하지 않은 축:

```text
mel-001 전체 표/셀 배치
tb-org-02의 한컴 표시에서 셀내 줄나눔 폭 산정 차이
```

특히 `tb-org-02-gradient-fixed.hwp`는 rhwp-studio에서는 정상이나 한컴 에디터에서는 같은 셀의 텍스트를
1글자씩 줄나눔한다. 정답은 2글자씩 줄나눔되는 형태다. 한컴이 1글자씩 줄을 끊으면 셀 높이가 증가하고,
그 결과 표 전체 높이와 페이지 배치가 틀어진다. 이 문제는 gradient BorderFill과 독립된 셀 내부 줄나눔
폭 산정 contract 문제로 분리한다.

## 9. 다음 단계

다음 단계는 두 갈래로 분리한다.

```text
1. gradient BorderFill HWP5 contract 수정은 유지한다.
2. mel-001 표/셀 배치 실패는 별도 probe로 계속 추적한다.
3. tb-org-02 한컴 표시 차이는 셀 내부 문단 폭, char shape, line segment, 문자 배치/줄나눔 contract를
   정답지와 비교해 별도 stage로 분리한다.
```

## 10. tb-org-02 문제 재정의

`tb-org-02`의 남은 문제는 다음과 같이 재정의한다.

```text
현상:
  생성 HWP를 한컴 에디터에서 열면 조직도 셀 내부 텍스트가 한글자씩 줄나눔된다.

정답:
  같은 셀에서 텍스트가 2글자씩 줄나눔되어야 한다.

결과:
  한글자씩 줄나눔되면 셀 높이가 증가하고, 표 전체 높이가 커져 페이지 배치가 틀어진다.

해결 방향:
  셀 높이를 사후 보정하지 않는다.
  한컴의 셀 내부 줄나눔 폭 산정이 정답지와 같아지도록, 해당 셀의 HWP5 contract를 핀셋 비교한다.
```
