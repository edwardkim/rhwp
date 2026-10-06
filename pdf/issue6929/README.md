# #6929 검토 기준 PDF

- 입력: [저장소 HWP](../../samples/issue6929/148776468_search_ad_terms_press_release.hwp).
- 입력 SHA-256: `a05d313f34852e07305d8bf16223303d9cc00c3d9bdc0b5258f96dfcb347a43d`.
- 원본은 [#6929](https://github.com/edwardkim/rhwp/issues/6929)의 `148776468_0403 검색광고 광고주 약관 보도자료.hwp`이며, #7145가 추가한 Git 파일을 그대로 사용했다. 로컬 수집 원본과 바이트가 같다.
- `rhwp info --json`: `lastSavedWith.product=hancom-office-2010`, version `8.5.8.1327`. 규정에 따라 engine `2020`을 명시했다.
- 출력: [148776468_search_ad_terms_press_release-hwp-2020.pdf](148776468_search_ad_terms_press_release-hwp-2020.pdf), 17쪽, 510,756 bytes.
- 출력 SHA-256: `61c28f0265bde456dea299a6793e00cd731fabe78cb1e258bb9ca5397d233495`.
- 2026-09-15 HWP MCP job: `a93a5516-65d4-457e-bbf8-bb4c85e30e61`.
- `start(queued) → status(running/converting) → status(succeeded) → download(success)`를 확인했다. 완료 시각 `2026-09-15T09:18:50.828Z`; client/server byte 수와 SHA-256이 일치했다.
- Hancom `12.0.0.4605`, engine/profile `2020`, backend `hwp-managed-direct-dll-host`, `pdf_print_method=0`, `frame_print_to_pdf_ex_one_up`, input preprocessing `none`.
- font scope는 `session0/verified`, mapped/registered 2, failed 0, parent/child visibility `verified`였다. 이 출력은 신고자의 과거 변환 job을 재다운로드한 것이 아니라 같은 입력의 새 변환이다.

인증 정보와 서버 주소는 보존하지 않는다. 판정은 [#7145 검토](../../mydocs/pr/archives/pr_7145_review.md)에 기록한다.
