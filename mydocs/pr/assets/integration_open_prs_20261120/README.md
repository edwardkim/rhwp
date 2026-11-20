# 원 PR 네 건 통합 검증

고정 `upstream/devel`은 `0a437a7eefb19bfe81fc12866887e4548ea9554d`, 제품 검증 소스는
`25f9587840d09b08af2b37be178ab93379a7b9dc`다. 원본 커밋 15개에 도형 캡션과 셀 너비
구조체의 호출을 맞추는 메인터너 보정 1개를 적용했다.

[PR #9201](../../archives/pr_9201_review.md) · [PR #9202](../../archives/pr_9202_review.md) ·
[PR #9206](../../archives/pr_9206_review.md) · [PR #9207](../../archives/pr_9207_review.md)

## 공통 검증

| 항목 | 결과 |
| --- | --- |
| Ubuntu 최소 글꼴 전체 Nextest | 제품 검증 소스에서 13,844건 통과, 45건 제외, exit 0 |
| Rust 검증 | fmt·Native/WASM·workspace Clippy 및 빌드 통과, Native Skia 라이브러리 4,035건 통과·12건 무시, 직접 PDF 36건 및 그림 2건 통과 |
| 실제 Studio | 테스트 2,822건 통과, 제품 빌드·반복 저장·재열기 확인 |
| 원 PR 시각 비교 | Native·fresh WASM 26조건 540쪽, 90% 미만 및 고정 기준 대비 2%p 초과 하락 0쪽. 두 출력 경로의 #9202 9·10쪽만 제품 PNG가 달라졌다. |
| 편람·공개 보호 | 편람 HWPX Native 383쪽과 기여자 공통 보호 8조건 통과 |
| 메인터너 정본 보호 | 46조건 4,046쪽 전부 대조. 미달 2쪽으로 머지 보류. |

정본 보호는 Chrome Canvas 기반 fresh WASM과 Native Skia의 Print 출력을 각각 독립 한컴 Print와
96 DPI로 대조한 색상 경계 보정 전 2px raw 실루엣의 공식 두 자리 값을 사용했다. Mac에서
44조건 2,520쪽을 산출했다. 외부 그림을 포함한 sample10의 2조건 1,526쪽은 동일 소스의 Ubuntu
전쪽 증적과 정본을 다시 대조했다. [정본 TSV](../visual-protected-baseline/final.tsv) ·
[전쪽 비교](protected-comparison.tsv) · [생산자 해시와 재검증](protected-verification.json).

## 중첩 표 Native의 글꼴 공급 보정

중첩 표 5쪽에서 차이 6픽셀은 `※` 두 곳의 왼쪽 획에 모였다. 사용자 글꼴 디렉터리를 먼저
등록하면 Native의 256개 사용자 공급 한도에 도달하여 뒤에 지정한 독립 기호 OTF가 누락됐다.
실제 글꼴 선택 추적에서 시스템 대체 글꼴 Malgun Gothic이 선택됐고, OTF 파일 유무에 따라 출력이
바뀌지 않았다. 동일 SHA-256의 기호 OTF를 먼저 지정하니 세 `※`가 모두 그 프로그램을 사용했다.
원문 전진값 1300 HWPUNIT는 유지됐고, 5쪽은 99.99%에서 100.00%로 올라 Native 17쪽이 정본과
일치했다. 같은 공급 순서로 고정 기준을 재출력한 결과도 17쪽 TSV와 제품 PNG가 모두 같다.
정본 글꼴 환경과 프로그램 해시 집합은 동일하다.
[보정 전 review](nested-before-p005-review.png) · [overlay](nested-before-p005-overlay.png) ·
[보정 후 review](nested-after-p005-review.png) · [overlay](nested-after-p005-overlay.png) ·
[Native 전쪽 TSV](nested-native.tsv).

## 남은 WASM 두 쪽

| 문서·쪽 | 정본 | 기준·후보 | 직접 비교 |
| --- | ---: | ---: | --- |
| 가상융합산업 진흥법 시행령 19쪽 | 99.74% | 99.73% | [review](decree-p019-review.png) · [overlay](decree-p019-overlay.png) |
| 종합부동산세법 시행규칙 #7905 1쪽 | 99.98% | 99.97% | [review](issue7905-p001-review.png) · [overlay](issue7905-p001-overlay.png) |

Google 서명과 배포 SHA-256을 확인한 Chrome 157.0.8103.79에서 두 쪽을 각각 다시
산출했다. 같은 환경에서 고정 기준과 후보의 PNG 해시가 각 쪽마다 일치했고, Chrome .80에서
얻은 PNG 해시와도 같았다. 시행령 차이는 그래프 이미지 범례 주변에, 시행규칙 차이는 제목 아래
시행일·개정 문구와 일부 본문 글자에 남는다. 두 쪽에서 통합으로 인한 출력 변화는 측정되지 않았다.
정본 표시값 미달의 원인이 해소되면 두 문서 전쪽과 정본 보호 전체를 확인한다.
