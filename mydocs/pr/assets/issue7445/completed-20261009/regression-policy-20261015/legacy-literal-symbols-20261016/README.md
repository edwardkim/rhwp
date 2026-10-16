# 레거시 수식 literal 기호의 글리프·전진폭 보정

## 범위와 독립 근거

버전이 비어 있는 HYhwpEQ 수식의 literal `△․`가 일반 텍스트 추정 폭과
대체 글꼴로 넘어갔다. 원본115쪽·과거 분할197쪽의 독립 한컴2024 Print에서
두 기호의 Type3 `/Widths`는 각각1000/1000em이다. 설치한 공통 HyhwpEQ의
삼각형과 수학 곱셈점도 각각1024/1024em이며 U+2024 cmap은 없다.
독립 glyph 계약（로컬 중간 자료 `independent-glyph-contract.json`）에 원본/PDF·glyph program·글꼴
SHA와 범위를 기록했다. Type3와 설치 글꼴의 윤곽이 완전히 같다는 주장은 하지 않는다.

기존 수식 글리프 경로에서 literal을 `△`와 수학 곱셈점 `⋅`로 투영하고
전진폭·기준선·그릴 글리프를 공통 결과에 담았다. SVG·Native Skia·WASM Canvas는
같은 결과를 읽는다. 원문 스크립트와 텍스트 추출의 `△․`는 유지한다.
문서명·쪽 번호·점수 조건은 production 분기에 없다. 기존 현대 수식 경로도 유지한다.

## 전쪽 비교

정본 Visual Sweep의192DPI·print·compat2024·2px90%를 유지했다.
Native는 `export-png`의 Skia 출력, fresh WASM은 공통 Canvas PNG exporter 출력이다.
WASM은 locked wrapper `--target web --out-dir pkg --no-opt` 로컬 대체 빌드이며
최적화 Docker 통과로 보고하지 않는다. 실제 Studio 전체 소비 경로 수용은 별도 잔여다.

| 정확한 입력 | Native 최저 | fresh WASM 최저 | 쪽수 |
| --- | ---: | ---: | ---: |
| 원본 |90.74868%|90.63175%|115/115|
| 과거 d9bc 저장본 |90.24017%|90.24017%|197/197|
| 현재4a96 저장본 |90.24017%|90.24017%|197/197|

각509쪽, 합계1018쌍을 새로 비교했다.90% 미달0쪽이며, 직전 C50과 비교해
원본76쪽·각 저장본129/130쪽만 개선되고 다른 페이지는 같은 점수다.
전쪽 TSV·producer·비교와 잔여（로컬 중간 자료 `validation.json`）,
직전 후보 비교（로컬 중간 자료 `previous-candidate-comparison.json`）를 함께 확인한다.

## 직접 판독과 회귀

- [Native129쪽](native-giant4138-tab-period-review-129.png)
- [Native130쪽](native-giant4138-tab-period-review-130.png)
- [WASM 원본76쪽](wasm-giant4138-original-review-076.png)
- [WASM130쪽](wasm-giant4138-tab-period-review-130.png)

큰 삼각형과 그 뒤 점·변수 간격은 개선됐다. 역사적 Type3와 설치 글꼴의 삼각형
윤곽·굵기와 다른 수식의 첨자/간격 차이는 남는다.129쪽 첫 가속도 수식의 선두·분수
주변도 추가 확인 대상으로 남기며 전쪽 수식 fidelity 완료로 판정하지 않는다.

정식 `issue_4138_legacy_equation_literal_symbols` 검사는 실제130쪽 수식 소속,
원문·텍스트 추출 보존, 독립1em 기호 슬롯과 공통 SVG 글꼴 선택을 검사한다.
C50에서 실패하고 C51에서 통과했다. 최신 정식 Cargo 경로도1PASS,
기존 cache2검사도2PASS다. manifest/unit-tier·전체 fmt·workspace all-target Clippy도
모두 통과했다. fmt 누락과 포맷 뒤 suite 재생성 누락을 바로잡아
0건 선택 실행을 제외하고 실제 검사를 다시 확인했다. 98% 미만 쪽의 화면 좌표를 px로 고정하지 않는다.
전진폭 검사는 독립 glyph 계약의1em 비율이며 절대 화면 좌표 핀이 아니다.
기존 마지막197쪽99.11469%의 ±2CSSpx 보호는 유지한다.

## 제출 보류와 증거 한계

기존 최신 devel 대비 WASM 원본61쪽·과거 저장본7쪽의0.01%p 이상 하락은 남는다.
고정 base 비교 목록（로컬 중간 자료 `latest-base-comparison.json`）에 하락을 보존했다.
현재 저장본의 base 쪽수 차이도 미검증으로 유지한다. 법원 서식의90% 미달·공백 줄바꿈,
최종 전체 회귀·범위 gate와 실제 소비 경로 검증도 남아 PR은 보류한다.
`b88b666b`의 전체10438PASS는 C51 이전 코드의 기록이다.

Native frozen rlib를 처음 복사할 때 top-level의 이전 C50 파일을 선택한 진단 오류는
`deps/librhwp.rlib`로 바로잡아 실제 FAIL→PASS를 재검증했다. Native 실행 파일과
전쪽 PNG는 올바른 C51 빌드였으며 이 오류의 영향을 받지 않았다. 초기 full PNG 모드
후처리의 TSV 경로 오류도 보존 PNG를 정본 지표로 재산출하여 바로잡았다.
캡처 뒤 rustfmt만 적용한 source 차이는 별도 hash·동등성 기록에 구분한다.
