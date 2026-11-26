"""Create the owner's translation review workbook from the source inventory."""
import json
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(tempfile.gettempdir()) / 'pusthakam-sheet-deps'))
from openpyxl import Workbook, load_workbook
from openpyxl.styles import Alignment, Font, PatternFill
from openpyxl.worksheet.table import Table, TableStyleInfo
from openpyxl.worksheet.datavalidation import DataValidation

root = Path(__file__).resolve().parents[1]
inventory = json.loads((root / 'docs/translations/english-review.json').read_text(encoding='utf-8'))
wb = Workbook()
guide = wb.active
guide.title = 'Start here'
instructions = [
    ['PUSTHAKAM | Malayalam translation review', ''],
    ['What to edit', 'In Translations, fill the yellow Malayalam translation column. Add any questions in Your notes.'],
    ['Where to begin', 'Filter Screen / context to work on one module at a time. You can return a partly completed file.'],
    ['Existing drafts', 'Draft translation contains the setup wording already drafted. Copy or improve it in Malayalam translation if you approve it.'],
    ['What to keep', 'Leave ID and English text unchanged. Rows may be sorted: IDs identify each phrase.'],
    ['Dynamic text', 'Keep anything inside {braces} exactly as shown, including the braces. These are values inserted by the app.'],
    ['Wording', 'Use natural Malayalam script and consistent library terminology. Translate full phrases, not each English word separately.'],
    ['Data and names', 'Keep Pusthakam, ISBN, MARC, URLs, file extensions and protocol names as appropriate. Book titles and library records are not included for translation.'],
    ['Repeated words', 'Identical text is listed once with its contexts. If a word needs different translations in different contexts, explain in Your notes.'],
    ['Fragments', 'Some current screens assemble sentences from fragments. Explain any needed rewording in Your notes; these will be adjusted during integration.'],
    ['Scope', 'Source-derived frontend and public OPAC inventory, including shared text. Some runtime/backend errors and generated reports may need a later pass. Technical-looking candidates can be marked Skip.'],
    ['Review status', 'Optional: To translate, Ready, Question or Skip. Integration will be checked against the current source before use.'],
    ['Return', 'Send this XLSX back when you are ready. Existing app data and the English default will be preserved.'],
    ['Example English', 'Library name'],
    ['Example Malayalam', '\u0d32\u0d48\u0d2c\u0d4d\u0d30\u0d31\u0d3f\u0d2f\u0d41\u0d1f\u0d46 \u0d2a\u0d47\u0d30\u0d4d'],
    ['Source snapshot (UTC)', inventory['generatedAt']],
]
for r in instructions:
    guide.append(r)
guide.column_dimensions['A'].width = 29
guide.column_dimensions['B'].width = 105
guide.freeze_panes = 'B2'

ws = wb.create_sheet('Translations')
ws.append(['ID', 'English text', 'Malayalam translation', 'Draft translation - review only', 'Screen / context', 'Review status', 'Your notes', 'Source references'])
for entry in inventory['rows']:
    ws.append([entry['id'], entry['english'], '', entry['draft'], entry['contexts'], 'To translate', '', entry['sources']])
    # Always store text literally; never interpret a UI message as an Excel formula.
    for c in ws[ws.max_row]:
        c.data_type = 's'
table = Table(displayName='TranslationReview', ref=f'A1:H{ws.max_row}')
table.tableStyleInfo = TableStyleInfo(name='TableStyleMedium2', showRowStripes=True)
ws.add_table(table)
ws.freeze_panes = 'C2'
for col, width in {'A':20, 'B':66, 'C':66, 'D':60, 'E':45, 'F':18, 'G':44, 'H':65}.items():
    ws.column_dimensions[col].width = width
validation = DataValidation(type='list', formula1='"To translate,Ready,Question,Skip"', allow_blank=True)
ws.add_data_validation(validation)
validation.add(f'F2:F{ws.max_row}')
ws.auto_filter.ref = table.ref
ws.sheet_view.zoomScale = 80
ws.print_title_rows = '1:1'
for sheet in wb:
    sheet.sheet_view.showGridLines = False
    for row in sheet:
        for cell in row:
            cell.font = Font(name='Arial', size=11, color='203239')
            cell.alignment = Alignment(vertical='top', wrap_text=True)
    for cell in sheet[1]:
        cell.fill = PatternFill('solid', fgColor='143F42')
        cell.font = Font(name='Arial', size=11, bold=True, color='FFFFFF')
    sheet.row_dimensions[1].height = 32
for row in range(2, ws.max_row + 1):
    for col in ('C', 'F', 'G'):
        ws[f'{col}{row}'].fill = PatternFill('solid', fgColor='FFF2CC')
    ws[f'C{row}'].font = Font(name='Nirmala UI', size=12, color='174EA6')
    ws[f'D{row}'].font = Font(name='Nirmala UI', size=12, color='52636B')
    length = max(len(str(ws.cell(row, c).value or '')) for c in (2,4,5))
    ws.row_dimensions[row].height = min(160, max(44, 16 * (length // 65 + 1)))
for row in range(2, guide.max_row + 1):
    guide.row_dimensions[row].height = 46
wb.properties.creator = ''
wb.properties.lastModifiedBy = ''
out = root / 'docs/translations/Pusthakam_Malayalam_Translation.xlsx'
wb.save(out)
check = load_workbook(out)
assert check['Translations'].max_row == len(inventory['rows']) + 1
assert len({r[0].value for r in check['Translations'].iter_rows(min_row=2)}) == len(inventory['rows'])
assert all(not c.data_type == 'f' for sheet in check for row in sheet for c in row)
assert all(check['Translations'].cell(r, 3).value is None for r in range(2, check['Translations'].max_row + 1))
print(json.dumps({'path': str(out), 'phrases': len(inventory['rows']), 'drafts': sum(bool(r['draft']) for r in inventory['rows']), 'validation': 'passed: reopen, unique IDs, blank owner input, no formulas'}, ensure_ascii=True))
