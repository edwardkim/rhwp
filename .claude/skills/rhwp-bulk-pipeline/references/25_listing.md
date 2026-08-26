# 25 — 목록 만들기

batch 의 입력이 목록이다. 목록이 틀리면 모든 축이 틀린다.

## POSIX

```bash
find 폴더/ \( -name '*.hwp' -o -name '*.hwpx' \) -type f > 목록.txt
# 정렬이 필요하면
find 폴더/ \( -name '*.hwp' -o -name '*.hwpx' \) -type f | sort > 목록.txt
```

`find -name '*.hwp' -o -name '*.hwpx'` 는 괄호가 없으면 예상과 다른
트리를 걷는다. 반드시 `\( ... \)`.

## 실측 목록 (레시피 9)

```
samples/2022년 국립국어원 업무계획.hwp
samples/156636617_240617 2024년 5월 월간 수출입 현황(확정치).hwp
samples/field-01.hwp
samples/hwp3-sample.hwp
samples/없는파일.hwp
```

마지막 줄은 실패 시연. 커밋된 복사본: `examples/lists/recipe9.txt`.

## 넣지 말 것

- 디렉터리 경로
- `.pdf` / `.docx` / `.txt`
- 중복 경로 (두 번 처리된다. 버그가 아니라 입력)
- 주석
- 빈 줄

## 인코딩

UTF-8, 가능하면 BOM 없음. 경로에 한글·공백이 있어도 한 줄이면 된다.

## 권위

- `mydocs/manual/cli_commands.md` §batch
- `mydocs/manual/cli_json_pipeline_guide.md`
- `mydocs/manual/recipes/09_bulk_extract_convert.md`
- `mydocs/manual/recipes/05_mail_merge_batch_fill.md`
- `rhwp capabilities` 의 batch 항목
- 픽스처: `fixtures/` · 이 장: `25_listing.md`
- 이슈 #5311. gym 아님. 새 CLI 아님.
