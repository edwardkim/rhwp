# 교육과정 문서의 주 프로그램 전진·저장 어절 꼬리 보정

- [원본 HWP](../../issue7445/education-curriculum-map/1342000_edu_curriculum_map.hwp) SHA-256: `623b00d56beffc45d27c5bf23911bdc49d3a541ded8aecbb323d0716a2bc9f4e`.
- [독립 Print PDF](../../issue7445/education-curriculum-map/existing-hwp-hancom2020-print.pdf) SHA-256: `bc16ee92640644307fd1ce92ab8ffe663088210caa7740bef620daaa0187e197`.
  원 PR 코드 `8053bc6ea84036b8bdc50ae57dd7c9f7414e2a54`와 입력·PDF 바이트가 동일하다.
  파일명의 2020 표기와 PDF Creator `Hwp2022`는 구분하며 같은 Print/2022 호환 조건을 유지했다.
- [검증 원장](validation.json)에 생산 source 바이트·Native binary·fresh WASM·글꼴 공급·PDFium 신원을 연결한다.
  Mac WASM은 locked wrapper의 `--target web --out-dir pkg --no-opt` 로컬 대체 빌드다.

## 원인과 공통 보정

현재 devel의 `60972534680a0c5e51e54ec189b92e252e81a867`은 원 HFT 굵게 전진과 저장 셀 미스 정책을 보완했다.
같은 이름의 실제 주 TrueType 프로그램 신원이 스타일 해소에 전달되지 않으면 원 HFT의 5% 추적이 남았다.
또한 일반 셀 캐시 미스 정책으로 마커 기하·실제 어절 흡수 인증까지 끄면 앞 쪽의 꼬리가 다음 쪽에 유령 행으로 남았다.
주 프로그램·원 HFT 출처·치환 경계를 공통 해소에서 구분하고, 기존 공통 어절 인증을 저장 세대와 무관하게 소비한다.
문서명·쪽 번호에 따른 구현 분기는 추가하지 않았다.

## 전쪽 결과와 직접 판독

| 출력 | 전체 쪽수 | 최저 raw | 90% 미만 |
| --- | --- | --- | --- |
| [Native 원문 TSV](native-silhouette.tsv) | 415 | 95.79585% | 0 |
| [fresh WASM 원문 TSV](wasm-silhouette.tsv) | 415 | 95.79585% | 0 |

양쪽 쪽수는 독립 PDF와 같다. 415쪽의 표·셀·문단 소속과 내용 순서를 대조했고 추가·유실은 0자다.
Native64/WASM32의 같은 각주·머리말 `usize::MAX` 합성 마커는 같은 부호 값으로 대조했다.
이 집계는 공백 배치나 독립 Print 판독을 대신하지 않는다.

| 페이지 | Native review / overlay | fresh WASM review / overlay |
| --- | --- | --- |
| 4 | [review](native-p004-review.png) · [overlay](native-p004-overlay.png) | [review](wasm-p004-review.png) · [overlay](wasm-p004-overlay.png) |
| 271 | [review](native-p271-review.png) · [overlay](native-p271-overlay.png) | [review](wasm-p271-review.png) · [overlay](wasm-p271-overlay.png) |

두 페이지의 양쪽 raw는 각각98% 이상이며 같은 배율에서 직접 판독했다.
4쪽의 두 문단 시작은 독립 PDF 글자 원점139.2845/171.9068 CSSpx를, 271쪽 다음 행은 독립 벡터 선146.6413 CSSpx를 기준으로 ±2px만 허용한다.
대상은 원문 문단·셀 내용과 소속으로 찾는다. 글리프·안티앨리어싱 차이는 남아 픽셀 동일을 주장하지 않는다.

## 회귀와 반례

- `tests/cases/issue_8152_completed_curriculum_primary_program.rs`: 같은 Cargo 컴파일 검사 binary에 Native CLI만 바꾸어 실제 수정 전1 PASS/1 FAIL → 수정 후2 PASS를 확인했다.
  271쪽의 앞 행 부재·다음 행 내용 보존·독립 선 위치를 함께 검사한다. 4쪽은 선언한 주 프로그램의 독립 원점 대조군이다.
- `tests/cases/issue_7445_original_font_program_ownership.rs`: 실제 프로그램 공급 신원과 이름 존재만 확인한 경우, 다른 언어 슬롯의 원 HFT 출처를 구분하며 22건이 통과했다.
  초기 요청 이름 alias 문자열의 완전 일치 검사는 잘못된 진단으로 폐기했다. 이를 출력 회귀의 FAIL→PASS로 계산하지 않는다.
- px 회귀는 주 프로그램 선택을 명시하는 조판 계약을 사용하며 private TTF 설치나 실제 글리프 래스터에 의존하지 않는다.
  실제 글꼴 공급·출력 검증은 위 Native/fresh WASM 전쪽 증적과 구분한다. 글꼴 바이너리를 추가하지 않았다.

이 자료는 교육과정 문서의 현재 범위 증적이다. 다른 문서의 미달·전체 nextest·lint/Skia·Studio 실제 소비 검증을 대신하지 않는다.
