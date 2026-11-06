# 최종 로컬 검증

기준 `5716a226b562b7300193dd0b573ded9cd71ac890`, 검증 제품 src tree `c1ea19fbc94dd6aac77d4368755b6da5a9544e42`다. 보정 code `99a1085a9c` 이후 제품 소스는 바뀌지 않았다. `ec814abd46`은 기존 baseline 순서 복원이고 최종 증적 commit은 문서·PNG·JSON·TSV뿐이다. 로컬 검증 이후 최종 head `71052fccc6fc5090fa94ca9dd013be0ff5984ff7`의 GitHub 필수 CI와 self-review도 성공했다. [원격 완료 기록](remote-completion.md)에서 실제 URL·결과와 로컬 집계를 구분한다.

| 검사 | 실제 결과 |
| --- | --- |
| fmt·Native/WASM32 Clippy·workspace build·all-target Clippy | 전부 exit 0 |
| manifest prepare/check·manifest 계약 | 전부 exit 0, 비교 base 고정 |
| 집중 nextest | 27 passed |
| 전체 nextest | 12,752 passed, 기존 45 skipped, 실패 0 |
| Native Skia lib | 3,858 + 15 + 165 + 2 = 4,040 passed, 기존 ignored 12 |
| Native placeholder / direct PDF | 2 / 22 passed |
| 새 sample 보안 | 명시한 8개 입력, 전체 실행의 security_corpus_regression 6개 PASS |
| Native/fresh optimized WASM 원 사례+경계+정상 | 각각 12쪽 전부 90% 이상 |
| 최신 devel 기능 보존 | Native base/후보 55쌍 PNG 동일, fresh WASM 55쪽 전부 90% 이상 |
| 최신 devel 실제 WASM 공개 API | 12그룹 PASS, 실제 저장·재열기 |
| CLI 저장·재열기 | 16건 쪽수 보존, Native/WASM 각각 저장본 20쪽 전부 90% 이상 |
| 별도 실제 WASM 그림 저장 | 4건, 독립 원본 그림 bytes/원본 크기/crop/alpha 보존 |
| 실제 SVG browser 소비 | 4쪽 전부 99.94% 이상 |
| 최종 Native 편람 | 383쪽, 최저 99.96465%, 기존 baseline 하락 0쪽, passed |

최소 글꼴 bwrap에서 전체/집중 회귀와 Native 3종을 실행했다. 실제 노출 공개 글꼴 5개는 기존 minimal-font-environment.json에 기록했다. 시각 검증에는 입력의 원 글꼴을 별도로 공급했으며 글꼴 프로그램·embedded-font SVG를 커밋하지 않았다.

raw 실행 기록 latest-required-validation.json의 마지막 143은 제품 실패가 아니다. 전체 실행에 같은 환경변수로 8개 sample을 전달했고 보안 6개가 실제 PASS한 뒤, 동일 보안 필터만 다시 빌드하는 중복 실행을 중지한 기록이다. 종료 코드를 0으로 바꾸지 않았다. required-validation-completion.json은 이 실행 근거를 별도 집계하며 security-subset-proof.json에 원 PASS 행·명령·입력 목록이 있다. 나머지 필수 실행 13건은 모두 실제 exit 0이다.

자동 글꼴 공급의 #8572 7쪽 83.33333%와 CLI output 인수를 잘못 전달했던 첫 보조 드라이버 결과도 성공으로 집계하지 않았다. 원 PDF HCRDotum을 공급한 전 7쪽 재실행과 위치 인수 보정 후 16건 저장/전쪽 재검사를 완료했다. 이 과정에 제품 소스나 검사 기대값·threshold 변경은 없다.

이전 음성 대조의 Storage helper·두 실제 소비 경로·public handoff 파일 4개는 현재 제품과 bytes가 같다. 해당 실제 음성 테스트 exit 101은 과거 기준의 실행으로 보존했고, 최신 기준의 양성 경계 5개와 전체 집중·회귀는 다시 실행했다. source 범위를 구분하며 과거 음성을 최신 기준에서 실행했다고 표시하지 않는다.

#8599의 뒤 본문 회피 차이는 direct 판독에서 남는다. 원 사례 2쪽 100%나 추가 경계 95%로 이 차이를 면제하지 않으며, 작업지시자가 이 기존 차이에 한정한 수용 예외와 #8579 OPEN 유지·원격 후속 범위를 승인했다. 새 회귀는 예외 대상이 아니다.
