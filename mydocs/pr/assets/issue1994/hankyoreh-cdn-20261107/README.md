# 한겨레결체 원본 CDN 공급 검증

사용자가 지정한 `@noonnu/hankc@0.1.0`의 한겨레결체·Hankc 공급을 등록했다. 공식 원본 TTF와 CDN WOFF/WOFF2의 전체 12,443 cmap 문자 윤곽·전진폭을 대조해 차이 0을 확인했다. 글꼴 바이너리는 저장소에 포함하지 않는다.

최소 글꼴 macOS sandbox에서 일반 Native `export-png`가 실제 CDN을 내려받아 TSThgrgl 프로그램을 선택했다. 같은 sandbox의 Chrome에서 실제 Studio `loadWebFonts`가 한겨레결체를 loaded로 등록했고 fresh WASM으로 제어 문서를 출력했다. 실제 HTTP 글꼴·JS·WASM 응답 해시도 frozen 입력과 같다. 두 PNG는 직접 판독했다.

이 한 쪽은 공급 가용성 대조군이며 독립 한컴 PDF 대비 일치율 검사가 아니다. CanvasKit은 온라인 WOFF 계획 및 오프라인 미가용 계약만 확인했으며 실제 replay 통과로 표시하지 않는다. 원본 TTF를 Chrome FontFace가 거부하는 경우와 공개 WOFF2의 성공을 구분한다.

규칙 replay·projection 42 PASS, Studio runtime 4 PASS, generated projection check 통과. 공급 별칭·고정 URL·오프라인 미가용을 관계형 검사로 보호하며 픽셀 좌표 핀을 추가하지 않는다. 상세 출처·원본 및 공급 해시·실제 응답·producer는 validation.json (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue1994/hankyoreh-cdn-20261107/validation.json`)에 있다.
