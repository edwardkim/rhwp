# 동일 face 부분 글꼴 공급 진단

- source: `e6c4d315b9b01b54eeaa7ca414b951b36f3d1553`. 새 코드 head의 출력 증거가 아닌 원인 분리 진단이다.
- 입력: `pdf/issue7642-structure-argument-boundary-diagnostic/native-command-escape-scope-hancom-2020.hwp`.
- 기준: 같은 경로의 `native-command-escape-scope-hancom-2020.pdf`, 독립 한컴 2020 Print 전1쪽.
- 기존 Native binary에 `--embed-fonts=full --font-path /usr/local/share/fonts/hancom-office-2020/Install --compat 2022 --profile print`를 사용했다. 실제 cmap가 완전한 설치 face의 공급만 달리했다. 문서·글리프 위치·기준 PDF는 바꾸지 않았다.
- 전체 일치율은 98.69218%다. 정체 literal의 세리프·획이 복구되었다. 기존 부분 파일은 ASCII cmap가 없어 대체 글꼴로 표시됐다.
- `native-p001-complete-face-review.png`와 `native-p001-complete-face-overlay.png`는 위 진단의 대표 자료다. 새 source의 Native/fresh WASM 재검증과 확대 판독은 별도로 수행한다.
- 전체 실행·출처·중간 파일은 ignored `output/pr-review/issue7642/failed-inputs-20261008/stage15-current-validation/stage549-escape-complete-face/`에 보존한다.
