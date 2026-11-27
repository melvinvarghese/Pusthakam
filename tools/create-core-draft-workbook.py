"""Build a correction-only workbook for the 319 core Malayalam drafts."""
import json
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(tempfile.gettempdir()) / 'pusthakam-sheet-deps'))
from translation_review_status import automatic_review_status, validate_review_status
from openpyxl import Workbook, load_workbook
from openpyxl.styles import Alignment, Font, PatternFill
from openpyxl.worksheet.table import Table, TableStyleInfo
from openpyxl.worksheet.datavalidation import DataValidation

root = Path(__file__).resolve().parents[1]
snapshot = json.loads((root / 'docs/translations/core-draft-review.json').read_text(encoding='utf-8'))
glossary = json.loads((root / 'docs/translations/approved-glossary.json').read_text(encoding='utf-8'))
wb = Workbook()
guide = wb.active
guide.title = 'Start here'
for row in [
    ['PUSTHAKAM | Review 319 Malayalam drafts', ''],
    ['Review, do not retranslate', 'English and Malayalam draft are already filled. Only correct wording you want to change.'],
    ['Yellow cells', 'Use Your correction, Review status and Notes on the 319 drafts sheet. Leave ID, English and original draft unchanged.'],
    ['If the draft is good', 'Choose OK in Review status; you do not need to copy the draft into Your correction.'],
    ['If you change it', 'Type your preferred Malayalam in Your correction and choose Changed.'],
    ['If unsure', 'Choose Question and describe the issue in Notes.'],
    ['Review in small batches', 'Filter Screen / context to review one area. A partly reviewed file is welcome. Not reviewed rows remain drafts.'],
    ['Braces', 'Keep {count}, {title}, {date} and other placeholders exactly as shown. Move them within the sentence if needed.'],
    ['Natural wording', 'Prefer everyday Malayalam and familiar words. The Approved terms sheet preserves the 20 terms already agreed.'],
    ['Screen grouping', 'Context is inferred from current source references. Shared / usage to confirm includes draft wording not directly matched to a current screen.'],
    ['Scope', 'Exactly the 319 core resource phrases, not every app phrase. Setup/auth drafts and future modules are outside this batch.'],
    ['Example', 'English: Library name | Draft: see next row. Choose OK if it reads naturally.'],
    ['Example Malayalam', '\u0d32\u0d48\u0d2c\u0d4d\u0d30\u0d31\u0d3f\u0d2f\u0d41\u0d1f\u0d46 \u0d2a\u0d47\u0d30\u0d4d'],
    ['Snapshot (UTC)', snapshot['at']],
    ['When you return', 'Send this XLSX back. Corrections will be checked against IDs, source wording and placeholders before integration.'],
]:
    guide.append(row)
guide.column_dimensions['A'].width = 29
guide.column_dimensions['B'].width = 105

ws = wb.create_sheet('319 drafts')
ws.append(['Screen / context', 'English text', 'Malayalam draft', 'Your correction', 'Review status', 'Notes', 'ID'])
for entry in snapshot['rows']:
    ws.append([entry['area'], entry['english'], entry['draft'], '', 'Not reviewed', '', entry['id']])
    for cell in ws[ws.max_row]:
        cell.data_type = 's'
ws.freeze_panes = 'D2'
for col, width in {'A':25, 'B':56, 'C':62, 'D':62, 'E':18, 'F':42, 'G':20}.items():
    ws.column_dimensions[col].width = width
ws.column_dimensions['G'].hidden = True
table = Table(displayName='CoreDraftReview', ref=f'A1:G{ws.max_row}')
table.tableStyleInfo = TableStyleInfo(name='TableStyleMedium2', showRowStripes=True)
ws.add_table(table)
status = DataValidation(type='list', formula1='"Not reviewed,OK,Changed,Question"', allow_blank=True)
ws.add_data_validation(status)
status.add(f'E2:E{ws.max_row}')
ws.sheet_view.zoomScale = 75
ws.print_title_rows = '1:1'

approved = wb.create_sheet('Approved terms')
approved.append(['English', 'Approved Malayalam'])
for english, malayalam in glossary['terms'].items():
    approved.append([english, malayalam])
approved.column_dimensions['A'].width = 32
approved.column_dimensions['B'].width = 65
approved.freeze_panes = 'B2'

for sheet in wb:
    sheet.sheet_view.showGridLines = False
    for row in sheet:
        for cell in row:
            cell.font = Font(name='Arial', size=11, color='203239')
            cell.alignment = Alignment(vertical='top', wrap_text=True)
    for cell in sheet[1]:
        cell.fill = PatternFill('solid', fgColor='143F42')
        cell.font = Font(name='Arial', size=11, bold=True, color='FFFFFF')
    sheet.row_dimensions[1].height = 34
for n in range(2, ws.max_row + 1):
    for col in ('D', 'E', 'F'):
        ws[f'{col}{n}'].fill = PatternFill('solid', fgColor='FFF2CC')
    for col in ('C', 'D'):
        ws[f'{col}{n}'].font = Font(name='Noto Sans Malayalam', size=12, color='174EA6' if col == 'D' else '203239')
    longest = max(len(str(ws.cell(n, c).value or '')) for c in (2, 3))
    ws.row_dimensions[n].height = min(170, max(48, 19 * (longest // 48 + 1)))
for n in range(2, guide.max_row + 1):
    guide.row_dimensions[n].height = 45
for n in range(2, approved.max_row + 1):
    approved.cell(n, 2).font = Font(name='Noto Sans Malayalam', size=12)
    approved.row_dimensions[n].height = 32
wb.properties.creator = ''
wb.properties.lastModifiedBy = ''
out = root / 'docs/translations/Pusthakam_Malayalam_Drafts_319.xlsx'
automatic_review_status(ws, 4, 5, guide)
wb.save(out)
check = load_workbook(out)
assert check['319 drafts'].max_row == 320
assert check['Approved terms'].max_row == 21
assert len({row[6].value for row in check['319 drafts'].iter_rows(min_row=2)}) == 319
for n, entry in enumerate(snapshot['rows'], 2):
    assert check['319 drafts'].cell(n, 2).value == entry['english']
    assert check['319 drafts'].cell(n, 3).value == entry['draft']
    assert check['319 drafts'].cell(n, 4).value is None
validate_review_status(check['319 drafts'], 4, 5)
print(json.dumps({'file': str(out), 'drafts': 319, 'approved_terms': 20, 'validation': 'passed: exact pairs, unique IDs, blank correction cells, automatic status formulas'}, ensure_ascii=True))
