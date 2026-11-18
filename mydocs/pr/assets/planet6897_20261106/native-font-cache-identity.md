# 검증용 웹폰트 변환의 프로그램 신원 보존

## 관측과 선행 구현

R7 Native 측정은 20개 입력·341쪽을 끝냈지만 실행 래퍼는 종료 코드 파일 없이 143으로
종료됐다. 개별 입력의 종료 코드·TSV와 전체 완료 원장은 보존했다. 같은 엔진·입력·PDF의
`--resume` 확인에서 웹폰트가 필요한 입력은 `provenance` 불일치로 거절됐다.
글꼴 원본 WOFF2 SHA는 같은데 검증 캐시의 Native TTF SHA가 매번 달라졌다.

관련 공급 경로는 `5551091085`([PR #8565](https://github.com/edwardkim/rhwp/pull/8565))와
`96827b3acc`([PR #8583](https://github.com/edwardkim/rhwp/pull/8583))를 따른다.
원본 웹폰트의 해시를 확인하고, 검증 캐시를 매번 원본에서 변환해 기존 TTF 변조를
신뢰하지 않는 계약이다. 이 재검증은 유지해야 한다.

원인은 FontTools `TTFont`의 기본 `recalcTimestamp=True`다.
같은 WOFF/WOFF2를 매번 변환하면서 `head.modified`가 실행 시각으로 바뀌고 그 값과
체크섬이 Native 프로그램 SHA를 바꿨다. 재개 검사가 서로 다른 프로그램으로 거절하는
것은 올바르며, 그 비교나 판정 임계값을 완화할 문제가 아니다.

## 보정과 검증

Native 일반 공급과 편람 공급의 WOFF/WOFF2 변환을 각각
`TTFont(source, recalcTimestamp=False)`로 수행한다. 변환할 때마다 검증한 원본을 다시
읽고 캐시를 원자적으로 교체한다. 원본 TTF/OTF의 무변환 공급·원본 SHA 확인·cmap/이름
확인·공급 경로와 시각 판정값·임계값·provenance 검사 조건은 바꾸지 않는다.

기존 OFL 테스트 자원을 임시 WOFF/WOFF2로 만들어 변환 시계만 60초 바꾼다.
수정 전 두 경로 모두 Native SHA 불일치로 실패했다. 수정 후 같은 원본에서 동일 바이트와
SHA가 나오며 원본 수정 시각·Unicode cmap·글자 폭을 보존했다. 새 글꼴 바이너리는
저장소나 배포물에 추가하지 않는다.

- 수정 전 집중 검사: 2 FAIL, 종료 코드 1.
- 수정 후 두 공급 모듈의 Python 검사: 32 PASS, 종료 코드 0.
- 로그: `output/pr-review/planet6897-integration-20261106/`의
  `native-font-cache-determinism-red.log`, `native-font-cache-determinism-green.log`.

최초 R7의 TSV 20개·341쪽은 보존된 쪽 묶음에서 추출해 최초 완료 시 고정한 SHA-256과
바이트 단위로 모두 일치함을 확인했다. 원장은 `r7-preserved-tsv/recovery.json`이다.
이는 해당 R7 시점의 과거 증거 복구이며 실패한 `--resume`를 통과로 재표기하지 않는다.
이전의 서로 다른 변환 프로그램으로 만든 캐시도 새 프로그램의 검증 결과로 재사용하지 않는다.
이후 최종 측정은 보정된 변환과 고정한 프로그램 신원으로 다시 확인한다.
