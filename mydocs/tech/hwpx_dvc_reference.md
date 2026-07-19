---
kind: reference
status: active
canonical: mydocs/tech/hwpx_dvc_reference.md
last_verified: 2026-07-17
---

# 한컴 DVC (Document Validation Checker) 참조 가이드

- **작성일**: 2026-04-17
- **대상 프로젝트**: [hancom-io/dvc](https://github.com/hancom-io/dvc)
- **원본 저장소**: <https://github.com/hancom-io/dvc>
- **라이선스**: Apache License 2.0 (© 2022 Hancom Inc.)
- **관련 문서**: `mydocs/tech/hwpx_hancom_reference.md` (OWPML 모델 참조 가이드)

## 1. 프로젝트 정체

**HWPX 문서 적합성 검증기 (Document Validation Checker, DVC)**. 한컴이 2022년 Apache 2.0으로 공개한 C++ DLL 프로젝트. OWPML 모델을 사용해 HWPX 문서가 **정해진 서식 규칙(JSON)을 준수하는가**를 검증한다.

### 사용 시나리오

- 공공기관·학교·출판사 등이 문서 서식 규정을 JSON으로 정의
- 작성된 HWPX 파일이 규정을 따르는지 일괄 검증
- 예: "폰트는 바탕·굴림·맑은 고딕만", "줄간격 160% 이상", "특수문자 유니코드 범위 제한"

### 빌드 환경

- Windows 10 + Visual Studio 2017 + C++
- 의존성: `hwpx-owpml-model` (Apache 2.0) + `jsoncpp` (MIT)
- **리눅스·macOS 지원 불완전** — `OS_LINUX` 매크로 분기 있으나 본 빌드는 Windows만

## 2. 검증 대상 영역

`README.md` 기준 10개 영역 + `Checker.h`의 9개 `Check*()` 메서드:

| 영역 | JSON 키 | 체크 항목 |
|---|---|---|
| **글자모양** | `charshape` | 폰트, 크기, 색상, 굵기·기울임·밑줄·취소선·외곽선, 비율, 자간, 배경 테두리 — 40개 속성 |
| **문단모양** | `parashape` | 정렬, 여백, 들여쓰기, 줄간격, 단락 전후 간격, 탭, 페이지 넘김, 줄바꿈 규칙 |
| **표** | `table` | 테두리 4변, 셀 배경, treatAsChar, 표-속-표 허용 여부 |
| **특수문자** | `specialcharacter` | 허용 유니코드 범위 (min/max) |
| **테두리/글머리표** | `outlineshape`, `bullet` | 10 레벨별 번호 형식·글머리표 모양 |
| **문단번호** | `paranumbullet` | 레벨별 numbertype, numbershape |
| **스타일** | `style` | 사용 허용 여부 (permission) |
| **하이퍼링크** | `hyperlink` | 사용 허용 여부 |
| **매크로** | `macro` | 사용 허용 여부 |

## 3. 파일 구조

```
dvc/
├─ Checker.cpp (47KB)       — 검증 오케스트레이터. 9개 Check*() 메서드
├─ Checker.h                 — 인터페이스
├─ CommandParser.cpp/.h     — CLI 옵션 파싱
├─ DVCDefine.h              — 플랫폼·문자열 매크로 (Linux/Windows 분기)
├─ Factory.cpp/.h           — DVC 인스턴스 생성 팩토리
├─ Util.cpp/.h              — 공통 유틸
├─ Source/                   — 검증 로직 본체
│  ├─ CheckList.cpp/.h      — JSON 규칙 → 내부 체크리스트 구조체
│  ├─ OWPMLReader.cpp/.h    — OWPML 객체 → R* 검증용 IR 변환
│  ├─ JsonModel.h (591줄)   — errorCode 정의 + JSON 키 매크로
│  ├─ DVCModule.cpp/.h      — DLL 메인 모듈
│  ├─ DVCOutputJson.cpp/.h  — JSON 출력 직렬화
│  ├─ DVCErrorInfo.cpp/.h   — 오류 정보 구조체
│  ├─ ReaderUtil.cpp/.h     — OWPML 순회 유틸
│  ├─ C*.cpp/.h              — 영역별 체크 로직 (9개)
│  │    CCharShape, CParaShape, CTable, CSpecialCharacter,
│  │    COutlineShape, CBullet, CParaNumBullet, CStyle,
│  │    CHyperlink, CMacro
│  └─ R*.cpp/.h              — 검증용 중간 표현 (rhwp IR 대응)
│       RCharShape, RParaShape, RTable, RBullets, ROutlineShape
├─ export/
│  ├─ export.h              — DVC::IDVC 공개 API
│  └─ ExportInterface.h     — IDVCOutput 인터페이스
└─ sample/
   ├─ jsonFullSpec.json (646줄) — 전체 체크 가능 스펙 카탈로그
   └─ test.json (190줄)          — 실사용 예시
```

## 4. 동작 흐름

```
입력:
  - 검증 대상 HWPX 파일 경로
  - 검증 규칙 JSON (test.json 또는 jsonFullSpec.json 일부)

처리:
  1. OWPML 모델로 HWPX 파싱
  2. OWPMLReader가 OWPML 객체 → R* 중간 표현으로 변환
  3. CheckList가 JSON 규칙 → 내부 구조체로 파싱
  4. Checker의 Check*() 메서드들이 R*와 CheckList를 비교
  5. 위반 시 ErrorInfo 수집 (errorCode + 위치: pageNo/lineNo/tableID/col/row)

출력:
  - JSON 또는 XML (Default: JSON)
  - Console 또는 파일
  - 옵션: simple/all, default/alloption/table/tabledetail/shape/style/hyperlink
```

### 공개 API (`export/export.h`)

```cpp
DVC::IDVC* dvc = DVC::createDVC();
dvc->setCommand(argc, argv);   // CLI 옵션
dvc->doValidationCheck();      // 검증 실행
dvc->output();                 // 결과 출력
DVC::deleteDVC(dvc);
```

### CLI 옵션 요약

| 옵션 | 설명 |
|---|---|
| `-j` / `-x` | 출력 형식 JSON / XML |
| `-c` / `--file=PATH` | Console / 파일 출력 |
| `-s` / `-a` | simple (첫 오류 중단) / all (전체 오류) |
| `-d` / `-o` | default / alloption |
| `-t` / `-i` | table (표 단위) / tabledetail (셀 단위) |
| `-p` / `-y` / `-k` | shape / style / hyperlink 출력 |

## 5. errorCode 체계 (`JsonModel.h`)

영역별 1000번 단위 블록:

```cpp
JID_CHAR_SHAPE        = 1000    // 글자모양
JID_PARA_SHAPE        = 2000    // 문단모양  (추정)
JID_TABLE             = 3000    // 표
JID_SPECIAL_CHARACTER = 4000    // 특수문자
JID_OUTLINE_SHAPE     = 5000    // 개요
JID_BULLET            = 6000    // 글머리표
JID_PARA_NUM_BULLET   = 7000    // 문단번호
JID_STYLE             = 8000    // 스타일
JID_HYPERLINK         = 9000    // 하이퍼링크
JID_MACRO             = 10000   // 매크로
```

각 블록 내 세부 코드는 `+1, +2, ...` 로 증가. 총 **수백 개의 고유 errorCode**.

### 출력 샘플 (README)

```json
{
  "charIDRef": 6,
  "errorCode": 1005,
  "isInTable": false,
  "isInTableInTable": false,
  "lineNo": 4,
  "pageNo": 2,
  "paraPrIDRef": 0,
  "tableCol": 0,
  "tableID": 0,
  "tableRow": 0,
  "text": ""
}
```

## 6. 검증 규칙 JSON 스키마

### `test.json` 발췌 (실사용 예)

```json
{
  "charshape": {
    "langtype": "대표",
    "font": ["바탕", "바탕체", "돋움", "돋움체", "굴림",
             "굴림체", "궁서", "궁서체", "맑은 고딕"],
    "ratio": 100,
    "spacing": 0
  },
  "parashape": {
    "spacing-paraup": 0,
    "spacing-parabottom": 0,
    "linespacing": 0,
    "linespacingvalue": 160,
    "indent": 0,
    "outdent": 0
  },
  "table": {
    "border": [
      {"position": 1, "bordertype": 1, "size": 0.12, "color": 0},
      {"position": 2, "bordertype": 1, "size": 0.12, "color": 0},
      {"position": 3, "bordertype": 1, "size": 0.12, "color": 0},
      {"position": 4, "bordertype": 1, "size": 0.12, "color": 0}
    ],
    "treatAsChar": true,
    "table-in-table": false
  },
  "specialcharacter": {
    "minimum": 32,
    "maximum": 1048575
  },
  "outlineshape": { "leveltype": [...] },
  "bullet": { "bulletshapes": "□○-•*" },
  "style": { "permission": false },
  "hyperlink": { "permission": false },
  "macro": { "permission": false }
}
```

## 7. rhwp 관점의 가치

### 단기 — 본 타스크 #182에서의 활용

**보조 게이트**: Stage 5 완료 시점에 rhwp serialize 출력을 DVC로 돌려서 기본 서식 규칙 통과 여부 확인. 한컴2020 수동 오픈과 병행하는 또 하나의 보조 증거.

### 중기 — 별도 이슈로 Rust 포팅

**`rhwp validate` CLI 신설** (후속 이슈로 등록 예정):

1. DVC와 **동일한 JSON 규칙 스키마** 채택 → 기존 DVC 규칙 자산 그대로 활용
2. DVC와 **호환되는 errorCode 체계** → 기존 DVC 사용자의 출력 파서 재사용 가능
3. **rhwp IR 직접 사용**으로 OWPML 중간 층 제거 → 포팅 공수 ~60%로 단축
4. **HWP 바이너리도 동일 IR 경로로 검증 가능** → DVC 초월 (HWP/HWPX 통합 검증)
5. **WASM 빌드 가능** → 브라우저·rhwp-studio 실시간 검증
6. **리눅스·macOS·CI 자동화 가능** → 공공기관·학교의 일괄 검증 현실화

### 장기 — 생태계 영향

- DVC는 Windows 전용 → CI 파이프라인 자동화 불가
- Rust 포팅판이 **크로스-OS CI 자동화가 용이한 드문 HWPX 검증 도구**가 됨
- 공공기관·학교·출판사가 GitHub Actions·GitLab CI에서 바로 쓸 수 있음
- DVC 규칙 JSON 자산이 Windows 밖으로 이식되는 첫 경로

## 8. rhwp validate 설계 방향 (후속 이슈 청사진)

### 기본 명령

```bash
rhwp validate --rules test.json sample.hwpx
rhwp validate --rules test.json sample.hwp       # HWP 바이너리도 지원 (DVC 초월)
rhwp validate --rules test.json --format json --output result.json sample.hwpx
rhwp validate --rules test.json --level all --output-all sample.hwpx
```

### 출력 호환

기본 JSON 출력은 DVC와 **필드 이름·구조 동일**:
```json
{
  "charIDRef": 6,
  "errorCode": 1005,
  "lineNo": 4,
  ...
}
```

추가 필드는 `rhwp_*` 접두어로 구분 (예: `rhwp_source_format: "hwp"`).

### errorCode 체계

- **1000~10999**: DVC 호환 영역 (DVC의 `JID_*` 그대로)
- **11000~**: rhwp 고유 영역 (3-way BinData 정합, charPrIDRef 매핑 등 rhwp가 추가로 검증할 수 있는 구조 정합성)

### 검증 층 분리

| 층 | DVC | rhwp validate |
|---|---|---|
| 서식 규칙 검증 | ✅ | ✅ (DVC 호환) |
| 구조 정합성 (3-way BinData 등) | ❌ | ✅ (rhwp 고유) |
| HWP 바이너리 | ❌ | ✅ |
| WASM | ❌ | ✅ |

## 9. 포팅 공수 추정

| 범위 | 공수 | 산출물 |
|---|---|---|
| 최소 MVP (글자모양 + 문단모양) | 5~7일 | `rhwp validate --rules spec.json file.hwpx` 기본 동작 |
| 핵심 7개 영역 (표·스타일·하이퍼링크·매크로 포함) | 15~20일 | 현 DVC 수준 달성 |
| HWP 바이너리 지원 + WASM 빌드 | +5~7일 | DVC 초월 |

합계 약 **3~4주** (전담 기준). rhwp IR 재사용 효과로 DVC 원본 공수의 60% 수준.

## 10. 리스크·주의사항

### 스펙 해석 차이

DVC는 "한컴이 원하는 해석"의 정본. rhwp 포팅판이 같은 규칙 JSON·같은 HWPX로 **다른 결과**를 내면 "rhwp 검증은 부정확"이라는 인식 확산 위험.

**대조 테스트 필수**:
- Windows VM에서 DVC 실행 → 결과 JSON 캡처
- 같은 입력을 rhwp validate로 실행 → 결과 JSON 비교
- 필드 단위 일치율 ≥ 99% 유지

### errorCode 충돌 대응

한컴이 DVC 업데이트 시 새 errorCode 추가 가능. rhwp는 **호환 영역(1000~10999)은 동결**하고 신규는 rhwp 고유 영역(11000+)에만 추가. DVC 버전 태깅:

```bash
rhwp validate --dvc-version 1.0 ...   # 초기 호환
rhwp validate --dvc-version 2.0 ...   # 한컴 업데이트 반영
```

### Apache 2.0 파생물 표기

각 Rust 파일 헤더:

```rust
//! Ported from hancom-io/dvc (Apache License 2.0, © 2022 Hancom Inc.)
//! https://github.com/hancom-io/dvc
//! See mydocs/tech/hwpx_dvc_reference.md for details.
```

## 11. 시사점 요약

### rhwp에 주는 가치

1. **검증 규칙 스키마 정답**: `jsonFullSpec.json` 646줄 + `JsonModel.h` 591줄 = 한컴이 공식적으로 정의한 "검증 대상 속성 목록". OWPML 태그 목록(292개)과 함께 rhwp의 커버리지 판정 기준.
2. **errorCode 체계 호환**: DVC 사용자의 출력 파서·자동화 스크립트 재사용 가능.
3. **후속 이슈의 청사진**: `rhwp validate` CLI 설계의 권위 있는 레퍼런스.

### 본 #182에서의 역할 (제한적)

1. Stage 5 보조 게이트 (Windows VM에서 DVC 수동 검증)
2. 기술문서에 존재 명시 → 향후 포팅 이슈 착수 시 시작점

### 대체되지 않는 rhwp 영역

| 영역 | DVC | rhwp |
|---|---|---|
| 서식 규칙 검증 | ✅ Windows만 | 향후 포팅 후 ✅ (리눅스·WASM 포함) |
| 구조 정합성 검증 | ❌ | ✅ (rhwp 고유) |
| 편집·라운드트립 | ❌ | ✅ |
| 렌더링 | ❌ | ✅ |
| HWP 바이너리 | ❌ | ✅ |

## 12. 관련 자료

- **프로젝트 URL**: https://github.com/hancom-io/dvc
- **의존 프로젝트**: [hancom-io/hwpx-owpml-model](https://github.com/hancom-io/hwpx-owpml-model)
- **rhwp 관련 문서**:
  - `mydocs/tech/hwpx_hancom_reference.md` — OWPML 모델 참조 가이드
  - `mydocs/tech/hwp_hwpx_ir_differences.md` — HWP↔HWPX IR 차이점
  - `mydocs/plans/task_m100_182.md` — HWPX Serializer 완성 수행계획
  - Discussion [#184](https://github.com/edwardkim/rhwp/discussions/184) — LLM 기반 HWPX 생성의 함정 (검증 도구 필요성 언급)
