# rhwp ir-diff 명령 매뉴얼

## 개요

동일 문서의 HWPX 파일과 HWP 파일을 파싱하여 IR(중간표현) 차이를 자동 검출하는 디버깅 도구.
HWPX 파서가 HWP 바이너리 파서와 동일한 IR을 생성하는지 체계적으로 검증한다.

## 배경

HWPX(XML 기반)와 HWP(바이너리)는 동일한 문서를 다른 형식으로 저장한다.
한컴에서 HWPX를 HWP로 저장(또는 반대)하면 동일한 IR이 생성되어야 하지만,
파싱 과정에서 다음과 같은 불일치가 발생할 수 있다:

- UTF-16 code unit 매핑 차이 (탭 문자 8 code unit 등)
- 2× 스케일 변환 누락 (여백, 탭 위치, 줄간격 등)
- XML 속성 파싱 누락 (underline shape, fill 등)
- `<hp:switch>/<hp:case>/<hp:default>` 분기 처리 오류

## 사용법

```bash
rhwp ir-diff <파일A> <파일B> [-s <구역>] [-p <문단>] [--summary] [--max-lines <N>]
```

### 옵션

| 옵션 | 단축 | 설명 |
|------|------|------|
| `--section <번호>` | `-s` | 특정 구역만 비교 (0부터 시작) |
| `--para <번호>` | `-p` | 특정 문단만 비교 (0부터 시작) |
| `--summary` |  | 카테고리별 차이 카운트만 출력 (paragraph 헤더 + 개별 차이 라인 생략) |
| `--max-lines <N>` |  | 출력 라인 수를 N 으로 제한, 초과 시 truncation 마커 표시 (`=== 비교 완료` 라인은 항상 출력) |

#### 출력 가드 사용 예

```bash
# 카테고리별 카운트 (광범위 회귀 측정 시 유용)
rhwp ir-diff samples/hwpx/aift.hwpx samples/aift.hwp --summary

# 출력을 50 라인으로 제한 (긴 출력 처음 검사)
rhwp ir-diff samples/hwpx/aift.hwpx samples/aift.hwp --max-lines 50
```

### 사용 예시

```bash
# 전체 비교
rhwp ir-diff samples/tac-img-02.hwpx samples/tac-img-02.hwp

# 특정 문단만 비교
rhwp ir-diff samples/tac-img-02.hwpx samples/tac-img-02.hwp -s 0 -p 810

# ParaShape 차이만 필터
rhwp ir-diff samples/tac-img-02.hwpx samples/tac-img-02.hwp 2>&1 | grep "\[PS "

# TabDef 차이만 필터
rhwp ir-diff samples/tac-img-02.hwpx samples/tac-img-02.hwp 2>&1 | grep "\[TD "

# 차이 건수만 확인
rhwp ir-diff samples/tac-img-02.hwpx samples/tac-img-02.hwp 2>&1 | tail -1
```

## 비교 항목

### 문단 단위 비교

| 항목 | 설명 | 불일치 시 의미 |
|------|------|---------------|
| `text` | 문단 텍스트 | 텍스트 파싱 오류 |
| `cc` (char_count) | 문자 수 (UTF-16 code unit) | 탭/컨트롤 code unit 매핑 오류 |
| `char_offsets` | 문자별 UTF-16 위치 | LINE_SEG text_start 매핑 불일치 |
| `char_shapes` | 글자 모양 변경 위치/ID | CharShape 매핑 오류 |
| `line_segs` | 줄 레이아웃 (text_start, line_height, segment_width) | 줄바꿈/높이 불일치 |
| `controls` | 컨트롤 수 | 표/그림/글상자 파싱 누락 |
| `tab_extended` | 탭 확장 데이터 (너비, 리더, 종류) | 인라인 탭 파싱 오류 |

### ParaShape 비교

| 항목 | 설명 | 주의사항 |
|------|------|---------|
| `ml` (margin_left) | 왼쪽 여백 | HWP는 2× 스케일 저장 |
| `mr` (margin_right) | 오른쪽 여백 | HWP는 2× 스케일 저장 |
| `indent` | 들여쓰기 | HWP는 2× 스케일 저장 |
| `tab_def` (tab_def_id) | 탭 정의 참조 | 인덱스 불일치 시 탭 렌더링 오류 |
| `sb` (spacing_before) | 문단 앞 간격 | HWP는 2× 스케일 저장 |
| `sa` (spacing_after) | 문단 뒤 간격 | HWP는 2× 스케일 저장 |
| `ls` (line_spacing) | 줄간격 | Fixed/SpaceOnly/Minimum만 2× 스케일, Percent는 1× |

### TabDef 비교

| 항목 | 설명 |
|------|------|
| 탭 수 | TabItem 개수 불일치 |
| `position` | 탭 위치 (2× 스케일) |
| `tab_type` | 탭 종류 (0=왼쪽, 1=오른쪽, 2=가운데, 3=소수점) |
| `fill_type` | 채울 모양 (0=없음 ~ 11=삼중선) |

## 출력 형식

```
=== IR 비교: tac-img-02.hwpx vs tac-img-02.hwp ===

--- 문단 0.810 --- "□ 신사업 확장 계획"
  [차이] cc: A=24 vs B=108
  [차이] char_offsets[0]: A=0 vs B=0

  [PS 30] ls: 1800vs3600     ← lineSpacing 2× 스케일 불일치

  [TD 5] pos: 50152vs100304  ← TabDef position 2× 스케일 불일치

=== 비교 완료: 차이 1091 건 ===
```

- `A` = 첫 번째 파일 (보통 HWPX)
- `B` = 두 번째 파일 (보통 HWP)
- `[PS N]` = ParaShape 인덱스 N번의 차이
- `[TD N]` = TabDef 인덱스 N번의 차이

## 정상적인 차이 (무시 가능)

다음 항목은 HWPX와 HWP 간 구조적 차이로 인해 항상 다를 수 있으며, 렌더링에 영향 없음:

- **char_shapes ID 차이** (`cs[].id`): CharShape 테이블 순서가 다름
- **char_shapes pos 차이** (`cs[].pos`): 빈 문단에서 컨트롤 오프셋 차이
- **controls 수 차이**: HWPX에서 SectionDef 등이 별도 처리됨

## 디버깅 워크플로우

HWPX 렌더링 버그 발견 시 다음 순서로 진행:

1. **ir-diff로 차이 검출**
   ```bash
   rhwp ir-diff sample.hwpx sample.hwp
   ```

2. **관련 문단 상세 비교**
   ```bash
   rhwp dump sample.hwpx -s 0 -p 810
   rhwp dump sample.hwp -s 0 -p 810
   ```

3. **HWPX XML 원본 확인** — zip 해제 후 header.xml / section0.xml 검사

4. **수정 후 차이 재확인**
   ```bash
   # 수정 전후 차이 건수 비교
   rhwp ir-diff sample.hwpx sample.hwp 2>&1 | tail -1
   ```

## 발견 이력

이 도구로 발견·수정된 버그 목록:

| Task | 발견 항목 | 원인 |
|------|----------|------|
| #13 | TabDef position 2× 스케일 | HWPX HwpUnitChar 값 미변환 |
| #13 | ParaShape margin/indent 2× 스케일 | HWPX HwpUnitChar 값 미변환 |
| #13 | TabDef fill_type 매핑 오류 | HWPML DASH↔DOT 명명 반전 |
| #15 | Shape curSz width=0 | orgSz 폴백 미구현 |
| #16 | underline shape 미파싱 | HWPX charPr shape 속성 누락 |
| #17 | 탭 문자 UTF-16 code unit | 탭 1 code unit → 8 code unit |
| #18 | ParaShape lineSpacing 2× 스케일 | Fixed/SpaceOnly/Minimum 미변환 |
