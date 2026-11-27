"""Append 200 drafts to a new copy, preserving all original review cells."""
import hashlib
import json
import sys
import tempfile
from copy import copy
from pathlib import Path

sys.path.insert(0, str(Path(tempfile.gettempdir()) / 'pusthakam-sheet-deps'))
from openpyxl import load_workbook
from openpyxl.styles import Alignment, Font, PatternFill
from openpyxl.worksheet.table import Table, TableStyleInfo

root = Path(__file__).resolve().parents[1]
folder = root / 'docs/translations'
original = folder / 'Pusthakam_Malayalam_Drafts_319.xlsx'
output = folder / 'Pusthakam_Malayalam_Drafts_519.xlsx'
if output.exists():
    raise SystemExit('Expanded workbook already exists; do not overwrite possible owner edits.')
before_digest = hashlib.sha256(original.read_bytes()).hexdigest()
wb = load_workbook(original, rich_text=True)
if wb.sheetnames != ['Start here', '319 drafts', 'Approved terms']:
    raise SystemExit('Unexpected original workbook structure; inspect before changing.')
ws = wb['319 drafts']
headers = ['Screen / context', 'English text', 'Malayalam draft', 'Your correction', 'Review status', 'Notes', 'ID']
assert [cell.value for cell in ws[1]] == headers
assert ws.max_row == 320
preserved = [tuple(ws.cell(row, col).value for col in range(1, 8)) for row in range(2, 321)]
old_ids = {row[6] for row in preserved}
snapshot = json.loads((folder / 'additional-draft-review.json').read_text(encoding='utf-8'))
assert len(snapshot['rows']) == 200
assert len(old_ids) == 319 and not old_ids.intersection(row['id'] for row in snapshot['rows'])
ws.title = '519 drafts'
ws.cell(1, 8, 'Batch')
for row in range(2, 321):
    ws.cell(row, 8, 'Original 319')
for entry in snapshot['rows']:
    ws.append([entry['area'], entry['english'], entry['draft'], '', 'Not reviewed', '', entry['id'], 'New 200'])
    row = ws.max_row
    for col in range(1, 9):
        cell = ws.cell(row, col)
        cell.data_type = 's'
        cell.font = Font(name='Nirmala UI' if col in (3, 4) else 'Arial', size=12 if col in (3, 4) else 11, color='174EA6' if col == 4 else '203239')
        cell.alignment = Alignment(vertical='top', wrap_text=True)
        if col in (4, 5, 6):
            cell.fill = PatternFill('solid', fgColor='FFF2CC')
    longest = max(len(entry['english']), len(entry['draft']))
    ws.row_dimensions[row].height = min(220, max(48, 19 * (longest // 48 + 1)))
ws['H1']._style = copy(ws['A1']._style)
ws.column_dimensions['H'].width = 19
ws.column_dimensions['G'].hidden = True
for row in range(2, 321):
    ws.cell(row, 8)._style = copy(ws.cell(row, 1)._style)
for name in list(ws.tables):
    del ws.tables[name]
table = Table(displayName='CoreDraftReview', ref='A1:H520')
table.tableStyleInfo = TableStyleInfo(name='TableStyleMedium2', showRowStripes=True)
ws.add_table(table)
ws.auto_filter.ref = table.ref
for validation in ws.data_validations.dataValidation:
    if validation.type == 'list' and str(validation.sqref) == 'E2:E320':
        validation.sqref = 'E2:E520'
guide = wb['Start here']
for row in guide:
    for cell in row:
        if isinstance(cell.value, str):
            cell.value = cell.value.replace('Review 319 Malayalam drafts', 'Review 519 Malayalam drafts').replace('319 drafts sheet', '519 drafts sheet')
            if cell.value.startswith('Exactly the 319 core resource phrases'):
                cell.value = 'Original 319 core drafts plus 200 new phrases. The new 200 are review drafts, not yet installed in the app. This is not complete app coverage.'
guide.append(['Batch filter', 'Use the Batch column in 519 drafts to show New 200 or Original 319. Original corrections, statuses and notes are preserved.'])
guide.append(['New batch snapshot (UTC)', snapshot['at']])
for row in (guide.max_row - 1, guide.max_row):
    guide.row_dimensions[row].height = 48
    for cell in guide[row]:
        cell.font = Font(name='Arial', size=11, color='203239')
        cell.alignment = Alignment(vertical='top', wrap_text=True)
wb.properties.creator = ''
wb.properties.lastModifiedBy = ''
wb.save(output)
check = load_workbook(output, rich_text=True)
sheet = check['519 drafts']
assert sheet.max_row == 520
assert [tuple(sheet.cell(row, col).value for col in range(1, 8)) for row in range(2, 321)] == preserved
assert len({sheet.cell(row, 7).value for row in range(2, 521)}) == 519
for row, entry in enumerate(snapshot['rows'], 321):
    assert sheet.cell(row, 2).value == entry['english'] and sheet.cell(row, 3).value == entry['draft']
    assert sheet.cell(row, 4).value is None and sheet.cell(row, 5).value == 'Not reviewed'
assert not any(cell.data_type == 'f' for page in check for row in page for cell in row)
assert hashlib.sha256(original.read_bytes()).hexdigest() == before_digest
print(json.dumps({'file': str(output), 'total': 519, 'new': 200, 'original_review_rows_preserved': 319, 'original_unchanged': True, 'validation': 'passed'}, ensure_ascii=True))
