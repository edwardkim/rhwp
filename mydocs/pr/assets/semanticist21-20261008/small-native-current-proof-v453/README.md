# 새 소스의 CGMP·KPS Native 전쪽 검증

생산 코드 `ddf17f86476293b114b6236bfa6cf1ebc9d9d474`의 CGMP48/48쪽 최저47쪽93.64014%, KPS77/77쪽 최저42쪽90.85010%다.
미달/누락0, 실제 exit0/source guard=true. KPS68쪽98.69518%다.
현재 세 review를 독립 Print와 직접 열어 대조했다. CGMP 외곽·항목·날짜·날인칸,
KPS 본문 줄바꿈·항목 순서·끝 줄·서명칸 소속을 확인했다. 글꼴 획/굵기·미세 자간 차이는 남는다.
이는 Native125쪽의 검증이며 새 fresh 전쪽·전체 회귀·Canvas·최종 승인을 대신하지 않는다.

## 재현

ignored `output/pr-review/semanticist21-20261008/nested-frame-alignment-provenance-v453/`의
`whole-native-plan.json`에 입력/독립 Print·Native CLI·소스해시·정확한 명령을 고정했다.
`whole-native-results.json`의 cgmp/kps 종료 기록과 각 silhouette.tsv/manifest는 전쪽 원문이다.
실행은 `--silhouette-only --compat 2022 --embed-fonts full`과 실제 존재하는 검증 글꼴 경로를 사용했다.
대표 패널 명령/입력 PNG·PDF PNG·점수/해시는 `current-small-native-representatives.json`에 기록했다.

## PNG 해시

- `cgmp6035-native-p047-review.png` — SHA-256 `2ee949b56629be95b0e8e293f2b93f040bc5256df9f4176fe95e946aedef46e0`
- `cgmp6035-native-p047-overlay.png` — SHA-256 `47756826226b2e02047c9f079d30dc6775d54bc0eaf0b10a23672adab0de8e7e`
- `kps1156-native-p042-review.png` — SHA-256 `c114b98d2bd0d744ba1c1bc70fff0f0a0ca19f01bc0cf678f82fe8c4f33077ed`
- `kps1156-native-p042-overlay.png` — SHA-256 `5c232794e5c930d5eeb9c38b8f2d6e01744b70236128d64b0c7f6dd75df1f0d0`
- `kps1156-native-p068-review.png` — SHA-256 `6dd3642e70f79505e414b86e6c6cecb2d0c42f41321ca232f87d8f077e36a336`
- `kps1156-native-p068-overlay.png` — SHA-256 `5006a17108a7a9a4b835081b63b3c2a87fbe9bb209714ac81e45100e26826adc`
