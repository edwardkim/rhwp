---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase231-wmf-paint-contracts/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase231-wmf-paint-contracts/goal-checkpoint.md` · 원문 SHA-256: `b5c163215aa995e28a07e02a0abc254e5f1ba8a5b0997948c95acac031b1ffd9`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 231차 기록

WMF 보류 함수 12개를 현재 원 Native로 실행해 12 PASS를 확인했다. 같은 HEAD CLI에서 실제 converter SVG 15입력을 얻고 opacity=0 진단 사본을 렌더했다. 진단 사본의 보이는 픽셀은 모두 0이지만 각 원 문자열 predicate는 여전히 참이다. 원 Rust 반환값 주입 PASS나 자연 발생 제품 소실은 아니다.

완성 마스크의 강한 bit 계약은 정상 3,025표본에서 오류 0개, 숨긴 표본에서 오류 1,250개로 구분했다. 이를 페이지 ±2 CSS px로 완화하지 않는다. 별도 curve 문자열 분기를 실제 원본 소속·paint 계약으로 보정할 대상으로 판정했다.

원본 curve의 팔레트 전역 검색 7곳은 실제 DIB 2곳·escape 주석 4곳·펜 1곳이었다. 펜은 Width.y만 0→255로 바뀌며 공식 MS-WMF에서 무시되는 필드이므로 펜 표시 회귀로 부르지 않는다. 입력 변형은 실제 팔레트 필드에만 한정해야 한다.

최초 escape 부재 가정으로 멈춘 분석기는 실제 WMFC 청크 10개와 EMF+ GetDC/GDI의 정식 fallback 거부 경계를 추적해 보완했다. 이미 끝난 원 검사 12 PASS와 wrapper 생성을 다시 실행하지 않았다. 빌드 없이 공개 소비 경로를 연결했다.

증거 보류는 417→405개, 보정·정리 함수 지점은 324→336개다. 직접 검토 15,847개는 중복 증가시키지 않는다. 소스 수정·개명·외부 게시는 0이며 전체 감사는 진행 중이다.
