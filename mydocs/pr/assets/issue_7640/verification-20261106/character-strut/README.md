# #7640 — 글리프 없는 수식 호스트의 독립 글자 상자 보존

현재 단계는 **같은 브랜치의 로컬 보정·검증**이다. 이슈 전체·PR·오래된 이슈 큐가
완료되었다는 뜻은 아니다. #7642의 남은 표시 기호와 새 devel 기능 보존은 별도 진행 항목이다.

## 위반 규칙과 공통 보정

`samples/exam_science.hwp`의 원본 문단 117은 보이는 글자가 없는 수식 호스트다.
그러나 원본 글자 크기와 저장 TH/LH는 모두 1150HU이며 기준선 978HU도 같은 글자
상자를 소유한다. 현재 수식의 1125HU 상자만 사용하면 별도 글자 상자의 25HU를
잃어 뒤 문단의 전진이 0.333333px 작아진다. 이는 진단 관측값이며 새 픽셀 보정 상수가 아니다.

공통 `equation_line_metrics`에서 온전한 저장 줄의 TH·기준선과 **그 줄의 원본 글자 모양**을
대조한다. 일치하는 글자 상자는 보이는 run이 없어도 수식·텍스트의 기준선 합집합에 포함한다.
빈 run의 존재만으로 새 strut를 만들거나 과거 수식 높이를 되살리지 않는다. 문서·쪽·문단
번호 분기와 허용오차는 추가하지 않았다.

소비 경로는 `원본 글자 모양/저장 TH → 공통 줄 합집합 → composer의 줄 높이/기준선 →
뒤 문단의 흐름과 실제 배치`다. #7640의 재저장 수학처럼 원본 글자 크기와 일치하지 않는
과거 수식 예약은 기존 현재 프레임 갱신을 유지한다.

## 수정 전후 검사

- 동일한 과학 시험지 검사 6건: 수정 전 **4 PASS / 2 FAIL** → 수정 후 최소 글꼴·일반
  환경 각각 **6 PASS**. 기존 후속 표 간격 검사를 완화하지 않았다.
- 새 `empty_equation_host_preserves_its_independent_character_strut`는 원본 글자 크기·
  TH·기준선·독립 수식 상자의 관계와 최종 공개 렌더 트리의 유일한 줄 소유를 검사한다.
  4쪽이 98% 미만이므로 새 절대 px 핀을 추가하지 않았다.
- 최소 글꼴 Native 집중 묶음 **45 PASS**. 현재 수식 줄·완료 문서·캐시·10월 미주·
  Native 실제 글꼴 프로그램 대조군을 포함한다.
- Native Skia 필수 3종: lib **4,039 PASS / 12 ignored**, PNG **2 PASS**, 직접 PDF **22 PASS**.
- 최소 글꼴 전체 nextest: **12,642 PASS / 0 FAIL / 45 skipped**, 실제 검사 **511.841초**.
  컴파일을 포함한 명령 시간은 1,215.586초다.
- Native/WASM Clippy·workspace build·기본 all-target Clippy·fmt·manifest·unit tier도 같은
  source를 고정한 순차 driver에서 모두 exit 0으로 완료했다. 각 실제 명령·결과는
  최종 검증 기록 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue_7640/verification-20261106/character-strut/validation.json`)에 연결했다.

## 전체 페이지와 devel 대조

고정 base는 `ece9c3383bb2c73e0f055459ede727a9d3101ac6`이다. 두 backend 각 99쪽,
총 198쪽을 새로 출력하고 canonical 96DPI·compat2022·보정 전2px 실루엣 TSV를 다시 산출했다.
전체 쪽수 일치, 누락·90% 미달·devel 대비 0.1%p 초과 하락은 모두 0이다.

| 입력 | 쪽수 | Native 최저 | fresh WASM 최저 |
| --- | ---: | ---: | ---: |
| 과학 시험지 | 4 | 97.11295% | 96.91722% |
| 11월 재저장본 | 21 | 99.03802% | 99.02457% |
| 11월 원본 | 21 | 98.78595% | 98.77736% |
| #7375 사각형 대조군 | 50 | 98.31507% | 98.32714% |
| #7642 수식 폭 진단 | 3 | 96.16576% | 96.26274% |

TSV의 표시 소수와 원 raw 소수를 구분한다. 하락 판정은 원 raw 소수로 계산했다.
3쪽 진단의 삼각형·인용 작은따옴표는 수치가 통과해도 실제 glyph 차이가 남으므로
#7642 해결 근거로 사용하지 않는다.

## 대표 판독

과학 시험지 4쪽의 표·후속 흐름을 Native와 fresh WASM에서 직접 대조했다. 본문과
앞뒤 표의 소속·순서를 보존하며, 두 backend가 남기는 기존 미세 glyph 차이를 이 단계에서
모두 해결했다고 주장하지 않는다. 수학 재저장본의 18·19쪽도 새 source의 전쪽 산출에 포함했다.

- [Native 수정 전 review](before-native-science-p004-review.png) · [overlay](before-native-science-p004-overlay.png)
- [Native 수정 후 review](after-native-science-p004-review.png) · [overlay](after-native-science-p004-overlay.png)
- [WASM 수정 전 review](before-wasm-science-p004-review.png) · [overlay](before-wasm-science-p004-overlay.png)
- [WASM 수정 후 review](after-wasm-science-p004-review.png) · [overlay](after-wasm-science-p004-overlay.png)
- [수학 18쪽 Native review](after-native-nov-p018-review.png) · [overlay](after-native-nov-p018-overlay.png)
- [수학 19쪽 WASM review](after-wasm-nov-p019-review.png) · [overlay](after-wasm-nov-p019-overlay.png)

과학 기준은 `pdf/issue7642-exam-print/exam-07-exam_science-hwp-print-2020.pdf`,
SHA `1c828fef29d7761d8ef46c8da50cc59936a95128920453ca5cc0d6440ac82d14`다.
11월 원본은 PR7600 input-09의 정확한 Print를 사용했으며, 이전에 잘못 선택한 별도 PDF와
섞지 않는다. 각 입력·PDF 해시는 최종 validation에 연결한다.

## source·빌드·남은 항목

실행 producer는 부모 `55756c223c76a7c824c8b0b4217c337bd3389a27`와 미커밋 소스 diff
SHA `3763725e9710209e9b4ae923e6960ebfbd01f65f797412e5684fcc24fbc5273d`를 함께 고정했다.
Native binary SHA는 `71e410aaf662a1b0365cdf88f6a5c259ac80f051fd7f287ee630df47b7ad596f`다.
Mac locked wrapper의 web/no-opt WASM은 로컬 대체 빌드이며 Docker 최적화 통과가 아니다.
root pkg·Studio public·새 브라우저의 실제 초기화 응답 hash가 일치한다.

- 현재 범위 게이트와 최종 증적을 확인했다. 검사 본문은 유지하고 헤더의 측정 근거만 갱신한다.
- 새 devel #8542 기능 보존과 최종 조판 head의 편람 Native 383쪽은 아직 남아 있다.
- #7642 이후 오래된 이슈를 같은 브랜치에서 순서대로 계속한다. #7445 부모 이슈를 먼저 닫지 않는다.
- 실행 로그·중간 JSON·진단 스크립트는 ignored output에 보존하며 원격 PR은 새 승인을 받는다.
