# 이슈 #9101: OS 기본 글꼴 환경의 전쪽 조사와 제출

Issue: https://github.com/edwardkim/rhwp/issues/9101

## 범위

1단계에서 공개 공통 4개 문서의 검사·지침과 Native/fresh WASM 증적을 커밋했다.
이 단계는 일반 Windows 제공 글꼴 19개를 추가한 18개 원본의 환경별 조사 결과를 완성하고 PR을 제출한다.
측정하는 엔진·입력·PDF·공식 계산기는 4e0b20d source에서 고정했다. 이후 Python wrapper와
문서만 커밋했으며 Rust·WASM 엔진 source는 바꾸지 않았다. 장시간 측정의 첫 실행계획·binary·package
신원을 유지하고, 실행 중 새 PDF를 출력하거나 시스템 글꼴을 설치하지 않는다.

## 계획

- Native 18개 완료 결과와 WASM 나머지 전쪽 결과를 대응한다. 누락·중복·전체 쪽수 차이를 확인한다.
- 표본별 OS 글꼴 의존성과 정본 대비 차이를 표로 남기고 전체 TSV 원본을 보존한다.
- 실패 환경을 통과·렌더러 보정 완료로 표시하지 않는다. 공개 공통 gate 범위와 조사 범위를 구분한다.
- 변경한 문서의 링크와 metadata, 공통 자동 gate의 재개 및 267개 Python 계약 검사를 확인한다.
- 최신 base·본인 열린 PR 확인과 병합 시뮬레이션 후 승인된 PR·CI·병합·후속 처리를 진행한다.

## 검증 상태

공통 4개 문서 8개 출력의 첫 실행과 재개가 모두 passed이며 각 raw 100.00%다.
Python Visual Sweep 계약 267건 통과. Rust·Studio 소비 로직 변경이 없어 전체 Rust/Studio 재실행은
이 Python 정책 변경의 필수 범위에 해당하지 않는다. 기존 실제 Native/fresh WASM 산출물은 재사용한다.
18개 환경 조사의 마지막 sample10 WASM 763쪽이 진행 중이다. 완료 전에 조사 완료로 표시하지 않는다.

## 중간 검증 결과

1단계 커밋: `1dd9869dbb96ff997551bb604a139c353baf4969`.
전체 Markdown 상대 링크 668개와 변경 문서 5개 검사를 모두 통과했다.
새 공통 검사 CLI의 최초 실행과 재개가 각각 8/8 통과했으며 267개 계약 검사도 통과했다.
18개 조사 중 Native는 전부 완료, WASM은 17개 완료이며 sample10 763쪽을 진행한다.

## 최종 공통 실행

대표 이미지의 중복 쪽 라벨을 정리한 최종 driver를 다시 실제 실행해 Native/fresh WASM 8/8 passed를
확인했다. 네 문서의 전쪽 raw 표시값은 양쪽 모두 100.00%다.
16개 실제 측정 PNG의 SHA가 이전 직접 판독본과 동일함을 대조했다. 새 review·overlay와
runner/catalog·binary/package의 해시를 정식 증적으로 갱신했다.
`test_visual_sweep_contributor.py` 9건도 최종 driver에서 통과했다.

## 새 base의 영향 분석

원격 devel은 `d954632aab5b80aadbea39cece4f5a783634856b`로 전진했다.
`c5022cbe8d`는 기존 검토 후속 기록, `5ab44e8315`는 PR #9099의 CI frontend Canvas 분리,
`d954632aab`는 PR #9100의 후속 기록이다. Rust·기준 입력/PDF·폰트 registry·Visual Sweep
계산기·Native/WASM exporter에는 차이가 없다. 18개 측정 종료 후 커밋·리베이스하고
CI 정책의 소비 경로를 집중 검사한다. 엔진 출력 재측정이나 Cargo 재빌드는 이 변경의 영향 범위가 아니다.

## 전체 조사 완료와 OS별 공식 목록

Native와 fresh WASM 모두 18개·각 1,771쪽, 총 3,542쪽의 전쪽 측정을 완료했다.
모든 원본의 독립 PDF 쪽수가 일치했다. 전쪽 TSV 36개를 바이트 그대로 보존하고 SHA와
원본/PDF/compat/페이지별 미달·정본 하락 목록을 `os-stock-evidence.json`에 기록했다.
다른 글꼴 조건의 실패를 통합 환경의 무회귀나 렌더러 보정 완료로 표시하지 않았다.

작업지시자 피드백에 따라 Apple·Microsoft·Ubuntu의 공식 자료를 웹에서 조회했다.
macOS 기본/다운로드/문서 지원, Windows Desktop/한국어 추가 기능, Ubuntu 의존/권장 패키지를
구분해 기여자 가이드에 출처 링크와 함께 반영했다. Palatino와 Palatino Linotype 등 유사 이름을
동일 프로그램으로 간주하지 않는다. 원본 18개의 font-plan 선언 목록도 추출했으며 실제 본문 사용
목록이라고 표현하지 않는다.

공식 목록 대조로 실험의 19개 파일이 Desktop 17개와 한국어 추가 TTC 2개임을 명확히 구분했다. Windows 기본 글꼴 전체나 macOS를 직접 실행한 결과로 일반화하지 않는다.

## 제출·code CI 확정

충돌 없는 리베이스 뒤 code head `5e0d8b6bc07e81065f572d55bdad4a71c9fe61df`로 PR #9102를 제출했다.
2026-11-16 01:38 KST 확인에서 CI·CodeQL·Adapter·Proptest·정책 검사 다섯 실행이 모두 success다.
CI lint의 Python 267건 실제 실행과 Native Skia·기본 archive A–E·frontend 성공도 확인했다.
이전 중간 상태의 진행 중 관측은 전체 조사 완료·최종 결과로 해소됐다. 정확한 check별 URL은
`mydocs/pr/assets/issue_9101/code-ci.json`에 보존하고 review·오늘할일 trailing 기록을 진행한다.
