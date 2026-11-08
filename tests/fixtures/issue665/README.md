# #665 합성 ingest 입력

이 작업에서 직접 작성한 문항과 직접 생성한 PNG 테스트 패턴이다. 외부 저작물이나
글꼴 프로그램을 포함하지 않는다. 공개 테스트 자원으로 프로젝트와 같은 라이선스를 적용한다.

- 최상위 JSON과 img/: CLI 및 byte-only builder의 입력.
- generated/: 실제 후보 CLI로 생성·저장한 HWPX/HWP.
- references/: 기준 devel의 기존 텍스트 빌더·그림 편집 API로 별도 작성한 독립 기준 문서와 JSON.
- 독립 한컴 Print: pdf/issue665/.
- 생성 방식·SHA-256·전쪽 TSV·직접 판독: mydocs/tech/investigations/issue-665/README.md.

이 폴더는 시험지 builder의 목적별 fixtures다. 일반 samples corpus의 baseline을 변경하지 않는다.
전쪽 쪽수와 내용 보존은 issue_665_ingest_images 회귀에서 함께 보호한다.
