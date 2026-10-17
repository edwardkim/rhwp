# #7751 셀 안 도형의 문단 기준 원점 — 15단계

시작 head는 `5060c4bb2`, 14단계의 실제 제품 source는 `465fd10fd`다.
14단계 전체 Rust 10,594건·Skia 4,043건과 전383쪽 점수 검증은 통과했지만,
289쪽 추가 확대 판독에서 실제 배치 차이를 발견했다. 점수로 면제하지 않고
같은 승인 범위에서 원인을 보정했다. 새 회귀 head `2ea129901`의 필수 최종 로컬 검증까지 모두 완료했다. GitHub CI와 merge는 다음 단계다.

## 독립 기준과 수정 전 결함

같은 편람 HWP·383쪽 KoPub 한컴2024 Print·96dpi를 유지한다. 입력과 PDF의
SHA·정상 Print 출처는 [14단계](task_m100_7751_stage14.md)와 같다. 기존 PDF를 재사용했다.

289쪽은 10구역28문단의 표, 4행2열 셀 문단2의 두 그룹이다. 독립 PDF의 B 경로에
CTM을 적용해 좌상단 CSS px로 변환했다.

| 개체 | 수정 전 Native | 독립 Print | 관측 |
| --- | ---: | ---: | --- |
| 위 그룹 첫 가로 화살표 x | 232.466667 | 258.76 | 26.293333px 왼쪽 |
| 아래 그룹 첫 가로 화살표 x | 230.186667 | 256.5227 | 약26.34px 왼쪽 |
| 그룹 밖 대각선 x | 약365.3 | 364.885333 | 정상, 약0.4px 차이 |

고정 devel의 실제 baseline도 같은 그룹 x를 내므로 이번 색조 변경의 신규 회귀는 아니다.
그러나 현재 #7751 보정 문서의 결함이므로 이관하지 않았다. raw99.6%와 전체 테스트 통과를
시각 완료로 간주하지 않았다. Direct PDF의 연결선 replay 미지원과 셀 앵커 결함도 구분했다.

## 원본 정보와 공통 호출 경로

원본 문단1206은 왼쪽 여백4000(1/2 HWPUNIT), resolved26.666667px다.
저장 LineSeg.column_start=2000HU와 일치한다. 들여쓰기−2136은 첫 글줄 속성이므로
문단 기준 떠 있는 개체의 원점에 더하지 않는다. 수동 메타데이터로 수용 조건을 바꾸지 않았다.

1. 전체 셀 `table_layout.rs`의 도형 호출 두 곳, 부분 셀 `table_partial.rs`의 두 곳,
   내장 표 `table_cell_content.rs::layout_embedded_table`의 호출이 원본 para_shape_id를 전달한다.
2. 공통 `layout_cell_shape_with_parent_path`가 resolved 좌우 문단 여백으로 가로 기준 원점과
   폭을 만들고, 도형의 Left/Center/Right 정렬이 그 결과를 소비한다. 본문 도형도 이미 같은
   resolved 문단 여백을 사용한다. 문서 ID·특정 여백 값·그룹 종류로 분기하지 않는다.
3. 최종 child_x를 받은 `layout_shape_object`의 Group 경로가 자식의 누적 render_tx/ty와
   결합한다. 중첩·회전 그룹의 bbox나 행렬 원점을 다시 더하지 않는다.
4. TAC는 확정된 글줄 위치, 문단이 아닌 가로 기준은 기존 셀 프레임을 유지한다.
   그림·표의 별도 위치 계약과 세로 원점·높이·흐름 예산·분할 컷은 바꾸지 않는다.
   전쪽 출력에서 x 외의 좌표·크기·내용·소유·순서가 같다는 실행 증거도 확인했다.

같은 원인으로 135쪽 일반 강조 사각형도 개선됐다. 원본은 3구역47문단 표의 셀0 문단1이다.
문단167은 좌우 여백각1000(1/2 HWPUNIT), 저장 column_start=500HU다. 문단은 Center지만
도형은 Para/Left이므로 문단 정렬로 도형 정렬을 대체하지 않는다. 독립 Print의 빨간 S 경로
bbox는 (112.198667,358.365333,66.968,43.836)px이며, 보정 Native는
(112.32,358.506667,67.04,43.72)px다.

## 새 전383쪽 검증

| 범위 | Native raw | fresh WASM raw |
| --- | ---: | ---: |
| 최저 | 93.07066 | 92.98362 |
| 135쪽 | 98.05498 | 98.03088 |
| 289쪽 | 99.74756 | 99.72209 |
| 90% 미만·누락·고정 devel 대비0.01%p 이상 하락 | 0 | 0 |

두 경로 모두14단계 대비135·289쪽만 개선됐다. 나머지381쪽 PNG는 byte-identical이다.
렌더 트리는 사각형1·그룹2와 자식선8, 총11노드의 x만 바뀌었다. 세로 좌표·크기·문구·소유·
노드 순서의 추가 차이는 없다. 전체 Native/WASM 내용·소유 대조도 내부64/32비트 usize
sentinel 대응 외에는 차이가 없으며11쪽 각주1개·미주0개를 유지했다.

135·289쪽의 양쪽 overlay를 직접 열어 강조 영역·8화살표·정상 일반선의 위치를 확인했다.
첫 그룹의 x 오차는0.373333px, 둘째는0.330633px로 줄었다.
[대표 PNG와 생산 지문](../pr/assets/pr7494_issue7751/README.md)에 연결한다.

## 정식 회귀와 정상 대조

전쪽90%·해당 쪽 양쪽98%·직접 판독 후
`tests/cases/issue_7751_handbook_completed_boundaries.rs`를 보강했다.
같은 동작은99%대289쪽 그룹을 우선하고, 다른 도형 종류·문단Center/도형Left 경계는
고유135쪽에서 보호한다. 문서 전체가98% 이상이라고 주장하지 않는다.

- 기존 QA 검사는 두 그룹의8화살표 시작 좌표±2px, 표·셀·문단 소유, 화살표 종류,
  누락·중복을 검사한다. 정상인 일반 화살표2개의 독립 Print 위치도 유지한다.
- 새135쪽 검사는 사각형의 독립 좌표·크기±2px와 원본 그림 안 포함, 문구 보존을 함께 검사한다.
- 같은 정식 파일을 frozen14 라이브러리로 컴파일(exit0)·실행하면4PASS/2FAIL(exit101)이다.
  289쪽은26.29px x 차이, 135쪽은 강조 사각형이 원본 그림의 왼쪽을 벗어나는 포함 관계에서
  실패했다. frozen15에서는 컴파일exit0·6PASS/0FAIL(exit0)이다. 빌드 실패를 재현으로 세지 않았다.
- 기존 중첩 글상자·TAC6건, 그룹 수준 roundtrip2건, 그룹 그림 crop2건도 모두 통과했다.
  합성 계약의 통과를 독립 한컴 출력과의 시각 일치로 확대하지 않는다.
- 최종 head의 focused9묶음은37PASS, Native Skia lib는4,043PASS/0FAIL(기존ignore12)다.
  전체 nextest는10,595PASS/0FAIL(기존skip45), 실행743.421초로 완료됐다.
  fmt·Native/WASM/workspace all-target Clippy·workspace build·manifest·unit-tier도 모두exit0이다.
  필수19단계와 지원 검사3단계·시각/브라우저33단계가 각각 최종 종료됐다.

## 실제 생산 출처와 실행 교정

Native bin/lib는 공용 target/pr-review·jobs8의 release-test 빌드로 각각exit0이었다.
첫 source commit `cef058e45` 뒤 끝난 cargo fmt의 tuple 줄바꿈만 build 시작 전에 적용됐다.
실제 producer는 그 commit+`stage15-native-producer-format.diff`다. 세 생산 파일 SHA는
`stage15-native-producer-sources.json`에 고정했고, 후속 `542a8bcbf`가 같은 byte를 포함한다.
Native frozen bin/rlib·명령·해시는 `stage15-native-provenance.json`에 보존했다.

fresh WASM은 `542a8bcbf`에서 jobs8·codegen16·no-opt host 경로로 빌드(exit0)했다.
public JS/WASM 동기화 해시를 확인한 뒤 이번 wrapper가 생성한 tracked JS만 복구했다.
실제 frozen package와 출처는 `stage15-fresh-wasm-provenance.json`에 남겼다.
후속 정식 테스트·문서 변경은 제품 코드와 구분한다.

첫 선택 Native 비교 명령은9PNG대383PDF에 --pages를 빠뜨려 누락 guard가 거부했다.
명령을 고쳐 같은 새9PNG로 성공했다. 이 실행 오류를 제품 결함이나 회귀 실패로 세지 않았다.

기존 Direct PDF는289쪽 Line 화살표를 손실 없이 replay할 수 없다는 명시적 오류(exit1)를 냈다.
63쪽 Path 연결선·130쪽 SVG payload 거부와 함께 지원 한계로 기록하며 guard를 완화하지 않는다.
독립 Print 색조5문서의 Native/fresh WASM/실제 Direct PDF는 전쪽 raw100%였다.
실제 Canvas·DOM 브라우저8건, Studio2,023건·Python156건·TypeScript도 새 head에서 통과했다.

TSV·전쪽 PNG·원본/PDF 해시·명령·오류코드·직접 판독·before/after·33PNG 지문은 ignored
`output/pr-review/pr7494-20261016/stage15-*`와 `logs/stage15-*`에 보존한다.
필수 로컬 검증을 완료했으며 승인된 통합 PR·같은 PR의 archive review/오늘할일 trailing·CI·merge·
후속 처리로 이어간다. 실제 결과가 나오기 전에 CI·merge 완료를 기록하지 않는다.
