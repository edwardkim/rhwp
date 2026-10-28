# Task m100 #1891 Stage 7

## 목표

Stage 6에서 닫은 HWP5-origin HWPX export/reparse 자기정합과 별개로, 공식 PDF 기준
원본 HWP/HWPX 렌더 쪽수 일치 여부를 확인하고 남은 차이를 수정한다.

## 초기 쪽수 비교

| 샘플 | 기준 PDF | HWP | HWPX |
| --- | ---: | ---: | ---: |
| `76076_regulatory_analysis` | 82 | 81 | 82 |
| `80168_regulatory_analysis` | 157 | 150 | 149 |
| `80250_regulatory_analysis` | 17 | 16 | 16 |
| `86712_regulatory_analysis` | 65 | 64 | 65 |

## Stage 7 정리 결과

### 적용 유지

- 빈 host 문단의 `TopAndBottom` + `RowBreak` 표가 렌더 단계에서는 `table.common.height`
  선언 높이를 하한으로 그려지는데, 페이지네이터가 셀 내용 측정치만으로 fit 을 판단하면
  현재 쪽 하단에서 표가 위로 clamp 되어 앞 내용과 겹친다.
- 이 보정은 파일명/페이지 번호가 아니라 문서 속성에 근거한다.
  - `table.common.treat_as_char == false`
  - `text_wrap = TopAndBottom`
  - `vert_rel_to = Para`
  - `TablePageBreak::RowBreak`
  - host 문단에 가시 텍스트 없음
  - `table.common.height` 선언 높이가 존재
- HWP5 CFB라도 문서 전체에 실제 저장 `LineSeg`가 있는 경우에는 저장 vpos 흐름을
  신뢰해 선언 높이 기준 이월을 허용한다.
- 문서 전체에 실제 저장 `LineSeg`가 없으면 재조판 흐름으로 보고, 셀 내용 측정치로는
  현재 쪽에 들어가지만 선언 높이로는 현재 쪽 하단과 겹치는 경우만 이월한다.

### 현재 쪽수 비교

| 샘플 | 기준 PDF | HWP | HWPX | 상태 |
| --- | ---: | ---: | ---: | --- |
| `76076_regulatory_analysis` | 82 | 82 | 82 | 통과 |
| `80168_regulatory_analysis` | 157 | 151 | 151 | 미통과 |
| `80250_regulatory_analysis` | 17 | 16 | 16 | 미통과 |
| `86712_regulatory_analysis` | 65 | 65 | 65 | 통과 |

### 정리/제외

- `RHWP_TABLE_DRIFT`용 임시 `TABLE_DECLARED_PUSH` 로그는 제거했다.
- `height_measurer`의 빈 문단 fallback 을 `CharShape`/`ParaShape` 기반으로 바꾸는 중간
  시도는 페이지 수 개선 효과가 없어 코드에서 제외했다. 이 관찰은 남은 80168/80250
  원인 분석 후보로만 둔다.
- #1906은 이번 #1891 PDF 기준 쪽수 검증 범위와 무관하므로 비교/판정 대상에서 제외한다.
- #1919는 #1898 PR이며 현재 작업과 파일 단위로 `src/renderer/layout.rs`만 겹친다.
  임시 index 기반 `git apply --check --3way` 결과 현재 변경 위에 patch 적용 가능했다.
  단, #1891 Stage 7을 먼저 커밋한 뒤 cherry-pick 검증하는 순서가 안전하다.

## 남은 원인 후보

- `80168_regulatory_analysis`는 한컴 기준 PDF의 90쪽처럼 큰 표의 잔여 행만 있는 쪽과
  153쪽처럼 거의 빈 구분 쪽이 존재한다. rhwp는 해당 반복 규제 블록의 큰 표/후속
  쪽나누기 구간을 더 압축해 총 6쪽이 부족하다.
- `80168` 후반부 예시:
  - PDF 151~152: `<토지보상법>`/`<도시정비법>` 법령 인용 표가 두 쪽으로 흐르고,
    PDF 153은 쪽 번호만 있는 빈 구분 쪽이다.
  - rhwp 146~148: `pi=1333`, `pi=1336` 법령 인용 표 뒤 `pi=1346 [쪽나누기]`
    구간이 더 압축되어 빈 구분 쪽이 생성되지 않는다.
- 다음 개선은 대형 `RowBreak` 표의 행/셀 높이와 후속 `[쪽나누기]` 처리 관계를 문서
  속성 기반으로 확인해야 한다.

## 진행 원칙

- 특정 파일명, 페이지 번호, PR/issue 번호, 임의 계수로 분기하지 않는다.
- #1906은 이번 #1891 PDF 기준 쪽수 검증 범위와 무관하므로 비교/판정 대상에서 제외한다.
- 보정이 필요하면 입력 문서에서 읽을 수 있는 `LineSeg`, `ParaShape`, `CharShape`,
  표/셀 속성, control 속성, section/page 속성 또는 공개 스펙 필드에 근거한다.
- 가장 큰 차이인 `80168_regulatory_analysis`부터 페이지 경계가 앞당겨지는 위치를
  확인한다.

## 검증 계획

- `pdfinfo`로 기준 PDF 쪽수를 확인한다.
- `rhwp dump-pages`로 원본 HWP/HWPX 쪽수를 확인한다.
- 필요 시 PDF/PNG 렌더와 render tree를 사용해 페이지 경계 차이를 좁힌다.
