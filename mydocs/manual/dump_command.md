---
kind: guide
status: active
canonical: mydocs/manual/cli_commands.md
last_verified: 2026-07-16
---

# rhwp dump 명령 매뉴얼

## 개요

HWP 문서의 조판부호(컨트롤) 구조를 텍스트로 출력하는 디버깅 도구.
문서 내 모든 문단, 도형, 표, 머리말/꼬리말 등의 속성을 상세히 표시한다.

## 사용법

```bash
rhwp dump <파일.hwp> [--section <번호>] [--para <번호>]
```

### 옵션

| 옵션 | 단축 | 설명 |
|------|------|------|
| `--section <번호>` | `-s` | 특정 구역만 출력 (0부터 시작) |
| `--para <번호>` | `-p` | 특정 문단만 출력 (0부터 시작) |

### 사용 예시

```bash
# 전체 문서 덤프
rhwp dump samples/basic/KTX.hwp

# 구역 0만 출력
rhwp dump samples/basic/KTX.hwp --section 0

# 문단 0만 출력
rhwp dump samples/basic/KTX.hwp --para 0

# 구역 0의 문단 3만 출력
rhwp dump samples/basic/KTX.hwp -s 0 -p 3
```

## 출력 형식

### 구역 헤더

```
=== 구역 0 ===
  용지: 210.0mm × 297.0mm (59528×84188 HU), 가로
  여백: 좌=8.0 우=8.0 상=5.0 하=5.0 mm
```

- 용지 크기: mm 단위와 HWPUNIT 단위 병기
- 용지 방향: 세로/가로
- 여백: 좌/우/상/하 mm 단위

### 문단 헤더

```
--- 문단 0.3 --- cc=17, text_len=0, controls=2 [단나누기]
  텍스트: (빈 문단)
```

| 필드 | 설명 |
|------|------|
| `0.3` | 구역번호.문단번호 |
| `cc` | char_count (제어문자 포함 총 문자 수) |
| `text_len` | 실제 텍스트 글자 수 |
| `controls` | 컨트롤 개수 |
| `[단나누기]` 등 | 문단 break 종류 (있을 경우) |

break 종류:
- `[구역나누기]` — Section break
- `[다단나누기]` — MultiColumn break (단 수 변경)
- `[쪽나누기]` — Page break
- `[단나누기]` — Column break

### 컨트롤 출력

#### 단정의 (ColumnDef)

```
  [1] 단정의: 2단, 유형=일반, 간격=5.0mm(1417), 같은너비=false
  [1]   단너비: [48.4mm, 2.1mm]
  [1]   구분선: type=1, width=3, color=0x00000000
```

#### 도형 (Shape)

```
  [3]   [직선] start=(0,79) end=(54356,0)
    선: color=0x00787878, width=567, style=0x4a0000
    크기: 127.0mm × 0.0mm (36000×3 HU)
    위치: 가로=용지 오프셋=8.2mm(2331), 세로=용지 오프셋=19.5mm(5532)
    배치: 위아래, 글자처럼=false, z=65
    요소: orig=54356×79, curr=36000×3, scale=(0.662,0.038), offset=(0,0), eff=84.1mm×0.0mm
```

**도형 공통 속성:**

| 항목 | 설명 |
|------|------|
| 크기 | CommonObjAttr의 width×height (mm + HWPUNIT) |
| 위치 가로 | HorzRelTo (용지/쪽/단/문단) + horizontal_offset |
| 위치 세로 | VertRelTo (용지/쪽/문단) + vertical_offset |
| 배치 | TextWrap 종류 |
| 글자처럼 | treat_as_char (인라인 배치 여부) |
| z | z-order |

**도형 요소 속성 (scale/offset이 기본값이 아닐 때만 출력):**

| 항목 | 설명 |
|------|------|
| orig | ShapeComponentAttr의 original_width × original_height |
| curr | current_width × current_height |
| scale | render_sx, render_sy (렌더링 스케일) |
| offset | render_tx, render_ty (렌더링 오프셋) |
| eff | curr × scale 결과 (실효 크기, mm) |

**변환 속성 (뒤집기/회전이 있을 때만 출력):**

```
    변환: 뒤집기=(false,false), 회전=67
```

**도형 종류별 추가 정보:**

| 종류 | 추가 출력 |
|------|----------|
| 직선 | start/end 좌표, 선 색상/굵기/스타일 |
| 사각형 | 모서리 곡률, 선 속성, 글상자 텍스트 |
| 타원 | (공통 속성만) |
| 호 | (공통 속성만) |
| 다각형 | 꼭짓점 수 |
| 곡선 | 제어점 수 |
| 묶음 | 자식 개수 + 재귀 출력 |
| 그림 | bin_data_id |

#### 표 (Table)

```
  [0] 표: 3행×11열, 셀=28, 쪽나눔=None
```

#### 머리말/꼬리말

```
  [2] 꼬리말: "※ KTX 소요 시간과 운임은..."
  [0] 머리말: "제목 텍스트"
```

#### 그림 (Picture)

```
  [0] 그림: bin_data_id=1
    크기: 50.0mm × 30.0mm (14173×8504 HU)
    위치: ...
```

#### 기타 컨트롤

```
  [0] 구역정의: 용지 210.0×297.0mm, 가로
  [0] 자동번호: type=Footnote, number=1
  [0] 책갈피: "bookmark_name"
  [0] 하이퍼링크: "https://..."
  [0] 필드: ClickHere "필드명령"
  [0] 감추기: header=true, footer=false, border=false, fill=false
```

## 위치 기준 참조

### 가로 위치 기준 (HorzRelTo)

| 값 | 표시 | 설명 |
|----|------|------|
| Paper | 용지 | 용지 왼쪽 끝 기준 절대 좌표 |
| Page | 쪽 | 본문 영역(여백 제외) 기준 |
| Column | 단 | 현재 단 영역 기준 |
| Para | 문단 | 현재 문단 기준 |

### 세로 위치 기준 (VertRelTo)

| 값 | 표시 | 설명 |
|----|------|------|
| Paper | 용지 | 용지 위쪽 끝 기준 절대 좌표 |
| Page | 쪽 | 본문 영역(여백 제외) 기준 |
| Para | 문단 | 현재 문단 기준 |

### 배치 방식 (TextWrap / TextFlowMethod)

HWP 바이너리 포맷은 4개 값만 사용한다 (hwplib 기준):

| 값 | 표시 | HWP 바이너리 | 설명 |
|----|------|-------------|------|
| Square | 어울림 | 0 | 사각형 영역으로 텍스트 감싸기 (FitWithText) |
| TopAndBottom | 자리차지 | 1 | 개체 위/아래로만 텍스트 배치 (TakePlace) |
| BehindText | 글뒤로 | 2 | 텍스트 뒤에 배치 |
| InFrontOfText | 글앞으로 | 3 | 텍스트 앞에 배치 |

HWPX 포맷은 추가로 Tight, Through 값도 사용한다.

## 단위 환산

| 변환 | 공식 |
|------|------|
| HWPUNIT → mm | `hu × 25.4 / 7200` |
| mm → HWPUNIT | `mm × 7200 / 25.4` |
| HWPUNIT → px (96DPI) | `hu × 96 / 7200` |

참고: 1인치 = 7200 HWPUNIT = 25.4mm = 96px (96DPI 기준)

## Docker 환경에서 실행

```bash
docker compose --env-file /dev/null run --rm dev cargo run -- dump samples/basic/KTX.hwp
```
