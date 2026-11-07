---
kind: snapshot
status: historical
canonical: mydocs/pr/assets/issue7375/final-20261108/validation.md
last_verified: 2026-11-08
---

# 최종 구현·검증 결과

## 실행 신원과 최종 커밋의 대응

조판·Rust 제품 코드의 검증 source는 `5011277356ad04aac8b07df7e56cc695ba297943`다.
고정 devel은 `5077b216cbc51d2046da07350de562f6f521a7c1`, 리베이스 직전은
`e944fff2f919f540c19e21caf4902b92f5a6b6da`다. 최종 증적·설명 커밋을 실행 producer와 구분한다.
검증 후 제품 Rust·Studio 소스·Rust 회귀·입력·도구 7,647개 파일의 바이트를 대조했다.
달라진 실행 파일은 암호 열기 E2E 한 개이며 해당 전체 E2E를 두 번 재실행하고 Studio 전체 검사도 다시 통과했다.
공개 TypeScript 선언은 같은 fresh WASM에서 생성한 root package와 일치하도록 동기화했다.

| producer | SHA-256 |
| --- | --- |
| Native Skia release-test | `18c5e2e3a1b120fb0641e1ce010e62b58495cb0258b2f657dac5c4b04fd23675` |
| fresh WASM | `2092cfe6d1b546ba24399f5d7db503ad12c49d096e47cbdce1bcd30b9605b78b` |
| 초기화 JS | `e1c2a4402be1060a666dda07d51e280ac08b3682d3a229495a630394249614b4` |

Mac에서 jobs 8·test threads 8·총 capture 8, Cargo 순차 및 캡처 배타 lock을 사용했다.
WASM은 사용자 지시의 locked wrapper `--target web --out-dir pkg --no-opt` 로컬 대체 빌드다.
Docker 최적화 통과를 주장하지 않는다. root pkg·Studio public·실제 브라우저 초기화 응답의 JS/WASM 해시가 같다.

## 완료한 검사

| 검사 | 실제 결과 |
| --- | --- |
| 전체 release-test nextest | **12,962 PASS / 0 FAIL / 45 skip**, 810.366초 |
| 준비·fmt·Native/WASM Clippy·workspace/all-target build/lint·Skia·manifest·unit tiers | 13개 gate 모두 PASS |
| Native Skia unit | root 3,857 PASS / 12 ignore, 기타 library 182 PASS |
| Native Skia PNG / PDF | 2 PASS / 35 PASS; 선택하지 않은 검사는 전체 실행으로 세지 않음 |
| 수동 frozen 집중 / 자동 집중 | 36 PASS / 163 PASS |
| Studio 전체 | 2,769 PASS / 2 skip; 최종 E2E 보강 뒤 동일 결과 재확인 |
| font contract / 도구 contract | 42 PASS / 81 PASS (Node 41·Python 38·Chrome 2) |
| Studio build | 최신 생성 선언을 소비해 PASS |
| 검증 입력 커밋 감사 | 실제 HWP/HWPX/PDF 99개와 Git 입력 바이트 일치 |
| Native/fresh WASM 본 검증 | 각각 **20조건·562쪽**, 98% 미만·누락·쪽수 차이 0 |
| devel / 리베이스 직전과의 전쪽 대조 | 페이지별 0.1%p 초과 하락 0 |
| 편람 Native `--handbook-gate` | 383쪽 PASS, 최저 99.96465% (130쪽), 기준 하락 0 |
| 실제 소비 backend | Native Skia 12 + Studio Canvas2D 12 + 지원 CanvasKit 12 PASS |
| 새 devel 보존 | 32조건·88쪽, base/후보×Native/WASM 128조건; 90% 미만·0.1%p 초과 하락 0 |
| 기존 #8738 보존 | 독립 Print 7조건×4경로 및 실제 편집·취소·저장 재열기 UI 8건 PASS |
| 암호·내용 손실 저장·감지 초기화 | 관련 실제 E2E PASS; HWP/HWP3/HWPX 암호 전체 entry는 연속 2회 PASS |

전체와 선택 실행·의도적 skip을 구분한다. 편람은 Native 383쪽 검증이며 fresh WASM 383쪽을 새로 완료했다는 뜻이 아니다.
원값 100%인 편람은 370쪽, 소수 둘째 자리에서 100.00%로 표시되는 쪽은 382쪽이다.
현재 전체 범위의 최저는 Native **98.31507%**, fresh WASM **98.32714%** (도형 문서 35쪽)다.
RGB 전체 동일이나 모든 글리프의 완전 동일을 주장하지 않는다.

[입력·기준 Print PDF·전쪽 TSV](README.md), [페이지별 base/후보/리베이스 전 대조](pagewise-preservation.tsv),
[원본 파일의 SHA-256·바이트 수·producer](assets.tsv)를 함께 제공한다.
TSV는 헤더·전쪽 행·원값을 변경하지 않고 복사했다. 기준 PDF는 기존 저장 버전별 Print 원본을 재사용했다.
각 입력의 제품 판정·compat·DPI·해시는 README의 실행 원장과 기존 개별 보정 기록에 연결된다.

## 직접 판독과 잔여 차이

아래는 실제로 review와 overlay를 모두 열어 대조한 대표 페이지다. 전체 TSV 판정과 직접 판독을 구분한다.

| 경로·조건·쪽 | 직접 판독 |
| --- | --- |
| native · clip5587 p3 | 왼쪽 연락처·교육부 목록과 오른쪽 예배 표의 소속·순서·외곽선을 확인했다. 약간의 글리프 가장자리 차이는 남는다. |
| native · delegation7700 p26 | 마지막 통계관리 행·병합 칸·오른쪽 결재 원과 하단 괘선의 포함·동일 종료를 확인했다. |
| native · giant4138 p90 | 8.2.1.1~8.2.2의 문장 순서·온도 숫자·C 기호·줄 끝 소속을 확인했다. raw100은 RGB 완전 동일을 뜻하지 않는다. |
| native · science p4 | 17~20번 문제·분수·화학식 첨자·그래프·보기 번호 및 하단 확인 사항의 소속과 간격을 확인했다. |
| native · lpg71352 p2 | 첫 저장 블록의 가운데 정렬·중첩 집단 표·다음 종료 행과 페이지 번호를 확인했다. |
| native · electrical7728 p6 | 현행/개정안 양쪽의 중첩 표·5번 항목·별표3/4 제목과 하단 물리 괘선을 확인했다. 미세한 선 농도/1px 가장자리 차이는 남는다. |
| native · rectangles p27 | 번호1의 둥근 외곽과 주석 상자·그림·다음 설명의 소유·상대 위치를 확인했다. |
| wasm · rectangles p27 | 같은 원본의 Canvas Print에서도 번호1 곡률·외곽·그림과 설명 순서가 유지된다. |
| native · prime-width p2 | 1000/1200/1800HU 구간의 접두어·prime·분수·함수 합성 기호와 원점을 확인했다. 래스터 가장자리 차이는 남는다. |
| wasm · nov-resaved p18 | 재저장본 양단의 적분·극한·함수식과 그래프·다음 문제의 소속을 확인했다. 적분/괄호의 미세한 가장자리와 기준선 차이를 raw100/RGB 동일로 주장하지 않는다. |
| wasm · prime-width p2 | 프라임·합성 함수·1000/1200/1800HU의 크기와 앞뒤 기호 간격을 대조했다. 작은 래스터 가장자리 차이는 남는다. |
| wasm · delegation7700 p26 | 마지막 통계관리 행·병합 칸·결재 원과 하단 괘선의 내용 포함을 확인했다. |
| wasm · electrical7728 p6 | 현행/개정안의 중첩 칸과 별표3·4 시작, 좌우 부모 괘선·하단 쪽번호를 확인했다. 제목 획 농도 차이는 남는다. |
| native · nov-resaved p18 | 적분·극한·분수·도함수와 그래프·다음 문항을 대조했다. 일부 수식 기호 가장자리·농도 차이는 남는다. |
| wasm · nov-original p18 | 원본의 적분·분수·극한과 그래프/다음 문항 소속을 대조했다. 일부 글리프 가장자리와 미세 기준선 차이는 남는다. |
| wasm · pii5846 p4 | 조문 대비표의 제31조·신설4/5항·밑줄과 하단 괘선/다음 조각 소속을 확인했다. |
| wasm · lpg71352 p2 | 규제개요 병합 칸·중첩 통계 표와 다음 내용의 시작·하단 괘선을 확인했다. |
| native · clip5587 p1 | 놀라운일/전파하리 반복 대시와 악보·기도 제목·오른쪽 등록 표를 대조했다. 글리프 농도·가장자리 차이는 남는다. |
| wasm · clip5587 p3 | 왼쪽 연락처·교육부 목록과 오른쪽 예배 표의 내용 순서·포함·외곽선을 확인했다. 일부 글리프 농도 차이는 남는다. |
| native · nov-original p18 | 원본의 적분·분수·극한과 그래프·다음 문항을 대조했다. 잉크 가장자리 차이는 남는다. |
| native · pii5846 p4 | 조문 대비표의 31조·신설 조항과 표 괘선·내용 소속을 확인했다. |
| wasm · clip5587 p1 | 가사 반복 대시·악보·등록 표의 소속·순서를 확인했다. 작은 잉크 농도 차이는 남는다. |
| native · rectangles p35 | 최저 페이지의 비트맵·점선 테두리와 본문·그림 소속을 확인했다. 작은 비트맵 보간/테두리 래스터 차이가 남는다. |
| wasm · rectangles p35 | 최저 페이지의 비트맵·점선 테두리와 본문·그림 소속을 확인했다. 작은 비트맵 보간/테두리 래스터 차이가 남는다. |

편람 130·272쪽 review/overlay에서는 본문·원문자·표·그림과 둘레선을 대조했다.
130쪽에는 작은 래스터 경계 차이가 남고 272쪽은 2px raw 100%다.
실제 일반 화면의 2쪽은 옥수수 원본 윤곽과 승인된 가사 Haansoft Batang 선택을 확인했다.
일반 화면 캡처와 독립 Print 비교 PNG를 동일 조건의 수치로 혼동하지 않는다.

기존 #7765의 25~29쪽은 본 20조건 완료와 별도의 보존 대조다.
후보 Native 최저 96.98466%, WASM 최저 98.05371%로 원래 요구된 90% 보존을 충족하며 28·29쪽은 양쪽 98% 이상이다.
기준 Native 최저 29.81354%와 구분하고 새 하락은 없다. 이 기록으로 #7765 전쪽 완료/종료를 주장하지 않는다.
CBTA #7727은 사용자가 지정한 제외 범위다. 기존 부분 보정·검사·미달 기록은 보존하고 추가 완료·이슈 종료를 주장하지 않는다.
#7445 전체도 미완료다. #1994·#4138·#8060은 이미 닫힌 이슈의 관련 보완으로 구분한다.

## 검증 과정의 실패와 정정

- #8418의 HFT 굵기 전진: 고정 devel 99.99715% → 초기 후보 99.89494%로 허용폭을 0.00221%p 넘었다.
  설치 윤곽 공급 시 기존 HFT 꾸밈 출처가 소실된 원인을 공통 측정 소비 지점에서 보정했다.
  같은 최종 공개 fixture 검사에서 수정 전 2 PASS·1 FAIL → 수정 후 3 PASS이며 원본 Native 2쪽은 99.99715%로 복원됐다.
  최종 새 devel 보존 대조의 WASM도 99.99715%이며 기준 99.82079%보다 개선됐다.
- 암호 E2E: 이전 문서의 폰트/view 초기화 종료 전에 다음 문서를 올려 실제 입력 길이가 0인 실행이 있었다.
  기존 UI 시간 제한 안에서 초기화 종료를 기다리고 실제 입력 길이를 검증했다. 암호 오답·정답·취소 기대값은 유지했다.
  최종 원 entry를 두 번 실행해 통과했고 actual JS/WASM 응답도 같은 producer임을 확인했다.
- 웹폰트 변환 cache의 메타데이터 시간으로 이전 manifest의 font hash가 달라진 경우, 이를 무시하지 않았다.
  해당 고정 base 3조건 13쪽과 source-frame 5쪽만 독립 cache로 다시 출력했다.
  실제 PNG는 이전 고정 base와 바이트·수치가 같으며 새 manifest hash를 검증했다.
- 외부 macOS font sandbox 안에서 Chrome 자체 sandbox가 중첩되어 실패한 진단은 최종 결과로 세지 않았다.
  외부 파일·fontserver 차단은 유지하고 Chrome 내부 sandbox만 해제한 실행에서 글꼴 차단 음성 대조와 브라우저 실행을 확인했다.
- source-frame symlink main 경로로 CLI entry가 실행되지 않은 결과·잘못된 cwd의 Studio 재실행은 무효 기록으로 구분했다.
  올바른 경로·cwd로 완료한 결과만 위 표에 기록했다. 전체 controller의 중간 실패는 보정한 개별 실행 결과로 대조했다.

## 제출물과 원격 단계

최종 제출은 TSV·대표 raw/review/overlay·설명만 포함한다. 진단 스크립트·로그·중간 JSON·설치 글꼴은 ignored output에만 보관했다.
기존 중간 JSON 170개도 바이트를 검증해 ignored output으로 옮기고 문서 링크를 정리했다.
원격 PR 번호와 CI 결과는 실제 채번·완료 뒤 기록한다. 이 보고서의 로컬 통과를 GitHub CI 통과로 대신하지 않는다.
