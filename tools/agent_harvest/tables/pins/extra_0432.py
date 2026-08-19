# extra pins

LINE = NAME = FIELD = CELL = ITEM = SHAPE = LEAF = None

def test_x0432_r_source():
    '''실측 잎'''
    assert LEAF == "C:\\Users\\swsz9\\rhwp-agent-cli-pack\\samples\\mix-shape-01.hwp"

def test_x0432_r_runs_0_argv_0():
    '''실측 잎'''
    assert LEAF == "tables"

def test_x0432_r_runs_0_argv_1():
    '''실측 잎'''
    assert LEAF == "--json"

def test_x0432_r_runs_0_exit():
    '''실측 잎'''
    assert LEAF == 0

def test_x0432_r_runs_0_stdout_command():
    '''실측 잎'''
    assert LEAF == "tables"

def test_x0432_r_runs_0_stdout_schemaVersion():
    '''실측 잎'''
    assert LEAF == "1.0"

def test_x0432_r_runs_0_stdout_source():
    '''실측 잎'''
    assert LEAF == "C:\\Users\\swsz9\\rhwp-agent-cli-pack\\samples\\mix-shape-01.hwp"

def test_x0432_r_runs_0_stdout_tableCount():
    '''실측 잎'''
    assert LEAF == 0

def test_x0432_r_runs_0_stdout_tool():
    '''실측 잎'''
    assert LEAF == "rhwp-agent"

def test_x0432_r_runs_0_stdout_untrustedContent():
    '''실측 잎'''
    assert LEAF == false

def test_x0432_r_runs_0_stdout_version():
    '''실측 잎'''
    assert LEAF == "0.8.4"

def test_x0432_r_runs_0_stderr_l0():
    '''stderr 줄 실측'''
    assert LINE == "표준 CFB 파서 실패: CFB 열기 실패: Malformed FAT (sector 0 pointed to twice), lenient 파서로 재시도..."

def test_x0432_r_runs_1_argv_0():
    '''실측 잎'''
    assert LEAF == "table-count"

def test_x0432_r_runs_1_argv_1():
    '''실측 잎'''
    assert LEAF == "--json"

def test_x0432_r_runs_1_exit():
    '''실측 잎'''
    assert LEAF == 0

def test_x0432_r_runs_1_stdout_command():
    '''실측 잎'''
    assert LEAF == "table-count"

def test_x0432_r_runs_1_stdout_schemaVersion():
    '''실측 잎'''
    assert LEAF == "1.0"

def test_x0432_r_runs_1_stdout_source():
    '''실측 잎'''
    assert LEAF == "C:\\Users\\swsz9\\rhwp-agent-cli-pack\\samples\\mix-shape-01.hwp"

def test_x0432_r_runs_1_stdout_tableCount():
    '''실측 잎'''
    assert LEAF == 0

def test_x0432_r_runs_1_stdout_tool():
    '''실측 잎'''
    assert LEAF == "rhwp-agent"

def test_x0432_r_runs_1_stdout_untrustedContent():
    '''실측 잎'''
    assert LEAF == false

def test_x0432_r_runs_1_stdout_version():
    '''실측 잎'''
    assert LEAF == "0.8.4"

def test_x0432_r_runs_1_stderr_l0():
    '''stderr 줄 실측'''
    assert LINE == "표준 CFB 파서 실패: CFB 열기 실패: Malformed FAT (sector 0 pointed to twice), lenient 파서로 재시도..."

def test_x0432_r_runs_2_argv_0():
    '''실측 잎'''
    assert LEAF == "table-inspect"

def test_x0432_r_runs_2_argv_1():
    '''실측 잎'''
    assert LEAF == "--json"

def test_x0432_r_runs_2_exit():
    '''실측 잎'''
    assert LEAF == 0

def test_x0432_r_runs_2_stdout_command():
    '''실측 잎'''
    assert LEAF == "table-inspect"

def test_x0432_r_runs_2_stdout_emittedCount():
    '''실측 잎'''
    assert LEAF == 0

def test_x0432_r_runs_2_stdout_schemaVersion():
    '''실측 잎'''
    assert LEAF == "1.0"

def test_x0432_r_runs_2_stdout_source():
    '''실측 잎'''
    assert LEAF == "C:\\Users\\swsz9\\rhwp-agent-cli-pack\\samples\\mix-shape-01.hwp"

def test_x0432_r_runs_2_stdout_tableCount():
    '''실측 잎'''
    assert LEAF == 0

def test_x0432_r_runs_2_stdout_tool():
    '''실측 잎'''
    assert LEAF == "rhwp-agent"

def test_x0432_r_runs_2_stdout_untrustedContent():
    '''실측 잎'''
    assert LEAF == true

def test_x0432_r_runs_2_stdout_untrustedFields_0():
    '''실측 잎'''
    assert LEAF == "tables[].cells[].text"

def test_x0432_r_runs_2_stdout_untrustedFields_1():
    '''실측 잎'''
    assert LEAF == "tables[].caption"

def test_x0432_r_runs_2_stdout_version():
    '''실측 잎'''
    assert LEAF == "0.8.4"

def test_x0432_r_runs_2_stderr_l0():
    '''stderr 줄 실측'''
    assert LINE == "표준 CFB 파서 실패: CFB 열기 실패: Malformed FAT (sector 0 pointed to twice), lenient 파서로 재시도..."

def test_x0432_r_runs_3_argv_0():
    '''실측 잎'''
    assert LEAF == "merged-tables"

def test_x0432_r_runs_3_argv_1():
    '''실측 잎'''
    assert LEAF == "--json"

def test_x0432_r_runs_3_exit():
    '''실측 잎'''
    assert LEAF == 0

def test_x0432_r_runs_3_stdout_command():
    '''실측 잎'''
    assert LEAF == "merged-tables"

def test_x0432_r_runs_3_stdout_mergedCount():
    '''실측 잎'''
    assert LEAF == 0

def test_x0432_r_runs_3_stdout_schemaVersion():
    '''실측 잎'''
    assert LEAF == "1.0"

def test_x0432_r_runs_3_stdout_source():
    '''실측 잎'''
    assert LEAF == "C:\\Users\\swsz9\\rhwp-agent-cli-pack\\samples\\mix-shape-01.hwp"

def test_x0432_r_runs_3_stdout_tableCount():
    '''실측 잎'''
    assert LEAF == 0

def test_x0432_r_runs_3_stdout_tool():
    '''실측 잎'''
    assert LEAF == "rhwp-agent"

def test_x0432_r_runs_3_stdout_untrustedContent():
    '''실측 잎'''
    assert LEAF == false

def test_x0432_r_runs_3_stdout_version():
    '''실측 잎'''
    assert LEAF == "0.8.4"

def test_x0432_r_runs_3_stderr_l0():
    '''stderr 줄 실측'''
    assert LINE == "표준 CFB 파서 실패: CFB 열기 실패: Malformed FAT (sector 0 pointed to twice), lenient 파서로 재시도..."

