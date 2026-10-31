# #8403 PDF 패턴 이미지와 문자 외곽선 처리

상태: 2026-10-31 승인 범위 구현·최종 로컬 검증 완료. PR 제출 후보 확정.
이슈: [#8403](https://github.com/edwardkim/rhwp/issues/8403).
최신 기준: `99a176b7cb34fde54454c03433baa1382b2de9fa`.
Native 생산 코드: `21a4902f142bc424c7a941e6328fb61a5f34b9e5`.
fresh WASM 생산 코드: `b13334f327193b7a6ed76c9f501f6fdee71c5e11`.
두 생산 코드 사이의 변경은 기존 `cfg(test)` 모듈 안으로 테스트 reader를 옮긴 것뿐이며
제품 코드는 같다. 작업지시자는 PR 생성 → self-review → 머지를 승인했다.

제출 준비 중 devel에 병합된 [#8397](https://github.com/edwardkim/rhwp/pull/8397)을 반영했다.
`TextStyle` 복사 목록의 충돌은 HFT 글꼴 필드와 `text_paint`를 모두 유지해 해소했다.
기존 기록의 이전 기준 `3c01421e89`와 생산 코드 `6d95c45f08`의 결과는 이력이며,
아래 결과는 새 기준에서 재실행한 최종 후보의 결과다.

## 수정 결과

검정 이미지의 원인은 이미지 디코더가 아니라 coloured tiling pattern의 이미지 수집 누락과
이전 검정 채움 상태의 잔류였다. 패턴 Matrix·BBox·반복 간격·Form·clip을 공통 추출 경로에서
처리하고 기존 alpha·SMask·디코드 캐시와 요청 예산을 재사용했다.

작업지시자는 문자 차이 예외를 철회하고 외곽선도 함께 수정하도록 결정했다. PDF `Tr=1/2`의
실제 채움 여부·획 두께·색·투명도·선 연결을 편집 가능한 글자모양과 paint에 보존한다.
Native Skia·SVG·Canvas2D·CanvasKit·HTML에 전달하며, 글꼴 weight를 올려 획을 대신하지
않는다. 내장 OpenType의 family 이름도 해석해 PostScript 이름의 대체 글꼴 문제를 보정했다.
줄 높이·기준선·줄바꿈 계산에는 획 두께를 더하지 않았다.

실제 검증에서 두 소비 경계도 보정했다. PDF Worker의 전달용 HWPX를 원본 형식으로 오인해
HWP 저장 때 그림 자르기 기준 크기가 지워지는 문제를 고쳤다. 또한 2×2 이미지의 1픽셀
선택을 무시해 반쪽 타일에 전체 색상을 압축하던 렌더러 조건을 제거했다. 공통 source 창을
그대로 소비하며 문서 ID·크기별 예외를 추가하지 않았다. 실제 색상을 검사하는 회귀도 보강했다.
계보·원인·수정 전 실패는 [작업 기록](../working/task_m100_8403_visual_gate.md)에 있다.

## 최종 시각·실사용 검증

최적화 WASM SHA-256:
`b9a7ebc5875dee4cd2d068a2e3bf3283b8beede8fd8f26c24eca0cb0d1f78d78`.
`pkg/`와 Studio 적용본, 실제 HTTP로 받은 JS/WASM의 해시를 확인했다.

96dpi print profile의 색상 경계 보정 전 2px raw 실루엣이다. 모든 대상 쪽을 비교했으며,
내용·쪽수·저장 전후 페이지별 텍스트 보존과 직접 판독을 함께 확인했다.

| 입력 | 쪽수 | Native 최저 | fresh WASM 최저 |
| --- | ---: | ---: | ---: |
| 원본 PDF 및 HWP/HWPX 재열기 | 각각 8 | 94.70% | 94.77% |
| 패턴 대조군 및 HWP/HWPX 재열기 | 각각 5 | 99.98% | 99.98% |
| 문자 획 및 14곳 편집·HWP/HWPX 재열기 | 각각 7 | 100.00% | 100.00% |
| 기존 PDF 대조군 직접 출력 | 12 | 99.20% | 98.85% |
| 기존 대조군 HWP/HWPX 재열기 | 각각 12 | 99.20% | 98.79% |

실제 CanvasKit은 패턴 5쪽 최저 99.98%, 문자 7쪽 전쪽 100.00%다. 기본 239개와 기존 그림 간격 대조 2개를 더해 총 241개 페이지 비교를
완료했다. 패턴 끝의 반쪽 타일은 Native·Canvas2D·CanvasKit과 저장 후에도 올바른 원본 색상
열만 표시한다. 잘못된 패턴·문자 paint 11종의 명시적 오류 및 취소 뒤 정상 문서 복구도 통과했다.

처음 이미지 수정만 적용한 원본 1쪽은 88.11%였다. 문자 획과 family 보정 후 위 결과로
개선됐으며 90% 미달 예외를 사용하지 않았다. 원본 내장 글꼴과 설치 family의 버전·농도,
hairline의 안티앨리어싱 차이까지 완전히 같다는 뜻은 아니다. 실루엣 100%도 색상 동일의
보증이 아니므로 직접 판독과 색상 선택 검사 결과를 별도로 남겼다.

[전쪽 TSV·공개 PNG·재현 스크립트](../tech/investigations/issue-8403/validation/README.md),
[공개 PDF와 생성기](../../pdf/issue8403/README.md)를 보존했다. 생성기 재실행도 19쪽의
독립 렌더링 픽셀이 보관본과 같았다. 원본 비공개 문서와 로컬 글꼴 바이트는 게시하지 않는다.

### 최신 글꼴 정책과 Studio 실제 화면

새 devel의 동의 기반 글꼴 정책에 맞춰 검증에서도 지원되는 host font provider에 실제 글꼴을
공급하고 Studio bridge로 Canvas를 출력했다. 원시 WASM Canvas 호출은 이 글꼴 세션을
우회하므로 최종 근거로 쓰지 않았다. 해당 우회에서 관측한 1쪽 87.15%와 브라우저 종료는
검증 경로를 보정한 뒤 위 전쪽 검사로 해소했다. 큰 글꼴은 한 번에 전달하지 않고 파일별로
공급한다. 원본 글꼴 두 개의 빈 VTT 소스 테이블은 기존 font catalog의 OTS 정규화를 사용했고
원본/정규화 SHA와 처리 내역을 기록했다. 글리프 윤곽·메트릭을 바꾸지 않았다.

실제 Studio 화면에서 공개 대조군의 Canvas2D와 CanvasKit을 각각 확인했다. 비공개 원본은
CanvasKit 요청 시 Arial·Droid Sans Fallback 리소스 미준비로 Canvas2D에 fallback되었다.
따라서 원본의 실제 화면 근거는 Canvas2D이고, CanvasKit 검증은 공개 패턴·문자 12쪽이다.

### 최신 devel 기능 보존

같은 독립 기준·글꼴 조건으로 base와 candidate의 241개 페이지 비교를 완료했다.
후보는 모두 90% 이상이며 기존 값보다 낮아진 최대 차이는 0.02%p였다.
추가로 #8397의 `hwp3-sample10.hwp` 763쪽을 Native/fresh WASM 양쪽에서 대조했다.
Native 최저 96.61%, WASM 최저 99.17%로 전쪽 통과했고, 각 backend에서 base/candidate
763개 PNG가 모두 바이트 단위로 동일했다. Native 제목 글꼴의 기존 모양 차이는 그대로
기록했으며 이번 수정의 개선으로 주장하지 않는다.

공개 HWP/HWPX 6개는 실제 Native 저장본의 바이트를 첨부했다. 이 파일을 fresh WASM으로
다시 열어 38쪽을 비교했으며 최저 99.98%였다. 원본 PDF에서 새로 변환한 별도 바이트로
첨부 파일 검증을 대신하지 않았다.

## 자동 검사

최소 글꼴 격리 환경에서 전체 회귀와 Native·Studio 검사를 완료했다. 이전 실행에서
발견한 SVG reader 4건은 기대값·허용 오차를 바꾸지 않고 소유한 crop viewport를 읽도록
보정했으며, 최신 기준의 전체 실행에서는 모두 통과했다.

| 검사 | 최종 근거 |
| --- | --- |
| Rust 전체 nextest | 12,233 통과, skip 45, 실패 0 |
| Native 라이브러리 | 4,040 통과, ignore 12 |
| Native 이미지 자리표시자 / 직접 PDF 출력 | 2 / 22 통과 |
| 이번 문자 획 / 이미지 패턴 회귀 | 기본 각 3 통과. 실제 crop 색상 검사를 포함한 Native 패턴 3 통과 |
| Studio 전체 / TypeScript | 최종 제품 코드에서 2,700 통과, skip 2, 실패 0 / 통과 |
| 최종 WASM lint·최적화 빌드·브라우저 | 통과 |
| 신규 HWP/HWPX 6개 보안 입력 | 명시적 입력 목록으로 20개 검사 통과 |
| 신규 샘플 포함 6종 레이아웃 래칫 | 6개 실행 파일의 81개 검사 통과, baseline 완화 없음 |
| fmt·Native·workspace lint·manifest | 제품 코드와 테스트 reader 보정 후 모두 통과 |

실행 명령·종료 상태·로그 해시는 [최종 검사 기록](../tech/investigations/issue-8403/validation/rebase/gates.json)과
같은 디렉터리의 `visual-run.json`·`finish-local.json`에 보존했다. 문서 링크는 통과했으며 메타데이터
검사의 기존 16건은 기준 브랜치의 동일 파일과 대조했다. 신규 오류는 0건이다.

## 지원 경계

- 이미지로 구성된 coloured tiling pattern을 지원한다. shading·uncoloured·이미지 외 paint,
  잘못된 간격·특이 행렬·순환은 명시적인 오류다.
- 문자 획은 단색 채움/획, 같은 저장 폭으로 표현되는 직교 CTM, q/Q·ExtGState·Form과
  hairline을 검증했다. 다른 폭을 요구하는 비균일 변환·점선·글리프 clip과 결합한 획 및
  패턴 문자 채움은 `PDF_TEXT_UNSUPPORTED`다.
- 정밀 문자 paint는 한도가 있는 RHWP 확장으로 HWP/HWPX 왕복 보존한다. 다른 편집기가
  이를 해석한다는 보장은 없으며 저장 진단은 `MetadataReduced`다. CLI `import-pdf` 성공
  JSON은 저장 진단 전체를 노출하지 않으므로 사용자 가이드에도 한계를 명시했다.
- 추가 기존 PDF의 전체 CLI HWPX 변환은 standard 누적 작업 한도에 도달했다. 정책을 바꾸지
  않고 HWPX·WASM은 1–6쪽·7–12쪽을 선택해 전쪽을 검증했다. 전체 HWPX 변환 성공이나
  같은 한도 초과의 base 재현을 주장하지 않는다. 원본 8쪽은 전체 문서로 검증했다.
