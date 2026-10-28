# 전체 회귀 실패의 독립 출력 자격

제품 `d1d3091c5`의 Native/fresh Docker WASM으로23입력37쪽/backend를 확인했다.
[입력·독립 PDF·SHA·페이지·최저값](inputs.tsv)과 양쪽 전체 TSV 원문을 그대로 보존한다.
Native 최저97.90355%, WASM 최저97.75192%이며 누락·쪽수 차이·90% 미만은 없다.
실행 경로는 compat2022·96dpi·PNG·full font embedding이다.

`stack-hwp`/`stack-hwpx`는 그림의 실제 x원점을 독립 Print와 대조했다. 기존±2px 검사를
완화하지 않고 실제 paint 원점과 hit 원점이 같은 source-aware 문단 offset을 소비하게 했다.
±2px 검사 대상5쪽은 HWP/HWPX 각각 Native/fresh WASM raw가 모두100%다.
나머지는 공백의 실제 TextRun 점유폭, 포함/교차 그림의 단일 소유, 빈 셀/병합 행/숨은 경계의
표 구조와 HWP/HWPX 왕복을 검증한다. 새 인식 계약과 모순되는 과거 negative 기대를
정상 출력·소유·편집 보존의 독립 관계로 바꿨다. 원래 crossing/open clip 반례는 거절한다.
검사 원 해결 PR/commit과 수정 전후 실행은 [stage38](../../../../../working/task_m100_7998_stage38.md)에 연결한다.

저장 입력은 `inputs/`에 있으며 원래 PDF/독립 Print는 저장소의 원본 경로를 재사용한다.
그림 원점은 원본 저장 HWP/HWPX 각각8쪽 전체를 독립 한컴2020 Print와 비교했다.
나머지 PDF import·저장 왕복은 원본 PDF의 독립 PDFium 출력을 기준으로 각각1쪽을 대조했다.
이 범위를 한컴 편집 후 전체 Print의 대체 증거로 표시하지 않는다.

`positioned-word-gap.pdf`의 Type3 reference는 PDFium 초기화 시 실제 HCRBatang font program을
명시 공급했다. 비어 있는 기본 reference 글리프를 정상 원본으로 쓰지 않았다. 공급은 로컬
Python 초기화의 사용자 글꼴 경로 설정이며 `visual_sweep.py`나 설치 패키지의 동작을 변경하지
않았다. 실제 글자가 있는 reference와 Native/fresh WASM 출력이100%인 것을 직접 확인했다.
상용 글꼴 파일·개인 PC 경로는 배포/CI의 새 의존성으로 포함하지 않는다.
