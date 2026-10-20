# KPS 최신 fresh WASM 전쪽 증거

생산 코드: `b4ff8c81b5d986baf47bad9a3eee83cbd9d8367b`.
WASM SHA-256: `94fe70f13609c5a006b8e9044463fac7ae96fe5fe03ae33bc0c0153b19f7867c`.

fresh 전77쪽 exit0, 최저42쪽 90.85010%, 미달·누락0,
원본/독립 Print 쪽수 77/77. 현재 캡처의 대표 쪽을 직접 열었다.
76쪽 RGBA 동일, 68쪽 한 픽셀의 회색 2단계 차이. 68쪽 전체 render tree 및 실루엣 98.69518% 동일.
글자 획·자간·래스터 차이는 남으며 완전 일치를 주장하지 않는다.
현재 Native 전쪽 재검증는 대기 중이다. 양쪽 완료나 PR 승인으로 확대하지 않는다.

## 입력과 독립 한컴 Print

- `samples/kps-ai.hwp` — SHA-256 `9b0fceb3d96956f27c893e15a72a1ad94f7ee005bd581381a1aadfcb1f57a7b9`; 실제 파일과 HEAD blob 일치.
- `pdf/kps-ai-2022.pdf` — SHA-256 `7c064fd290368369a3c8eaa7d7b03668c46fb4dfe0fc18ba52d00456ffe01d28`; 실제 파일과 HEAD blob 일치.

## 재현과 판독

원문 명령·글꼴·소스 해시·TSV·로그는 ignored
`output/pr-review/semanticist21-20261008/full-width-visible-alignment-v452/whole-fresh-execution-plan.json`, `whole-fresh-results.json`,
`kps-fresh/kps1156/silhouette_manifest.json`, `representative-review-provenance.json`,
`kps-fresh-direct-reading.json`, `kps-fresh-prior-pixel-comparison.json`에 보존한다.
전체 페이지 `--compat 2022 --silhouette-only --wasm-pkg pkg`를 실행했고
검증한 실제 공급 글꼴과 `--embed-fonts full`을 사용한다.

42쪽 문단 시작·제목·줄바꿈·내용 순서, 68쪽 본문·날인칸을 대조했다.

## PNG 해시

- `fresh-p042-review.png` — SHA-256 `5bc0276c7ba04a16175526edf985f59916d8f6eb0d4f5e9e1eedd1ef30a3a258`
- `fresh-p042-overlay.png` — SHA-256 `6d8c3fb1100b85d2d85bb358fe7af20626f29ffb85c7f03f91da19dc8d2f9e5b`
- `fresh-p068-review.png` — SHA-256 `8e6eaf9ffc934e2a461bd10b5e8becbc641447450b9f80ff6a74c3c8f4ecc8e1`
- `fresh-p068-overlay.png` — SHA-256 `37d0c3735d984c149e62c8eaa9fd78e1c90f2b08f0c1f38e0952ef516e6c133a`
