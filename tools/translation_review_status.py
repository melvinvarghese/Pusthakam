"""Formula-based review status for future translation workbooks (no macros)."""
from copy import copy
from openpyxl.utils import get_column_letter
from openpyxl.worksheet.datavalidation import DataValidation
from openpyxl.workbook.properties import CalcProperties


def automatic_review_status(sheet, correction_column, status_column, guide):
    header = 'Your review (OK / Question)'
    manual = next((c.column for c in sheet[1] if c.value == header), sheet.max_column + 1)
    sheet.cell(1, manual, header)
    sheet.cell(1, manual)._style = copy(sheet.cell(1, status_column)._style)
    sheet.column_dimensions[get_column_letter(manual)].width = 27
    for row in range(2, sheet.max_row + 1):
        current = sheet.cell(row, status_column)
        cell = sheet.cell(row, manual)
        if current.data_type != 'f' and current.value not in (None, '', 'Not reviewed', 'To translate'):
            cell.value = current.value
        cell._style = copy(current._style)
        correction = f'{get_column_letter(correction_column)}{row}'
        review = f'{get_column_letter(manual)}{row}'
        current.value = f'=IF({correction}<>"","Changed",IF({review}="","Not reviewed",{review}))'
    # Manual review moves to a separate input; computed status cannot be overridden by a dropdown.
    for validation in list(sheet.data_validations.dataValidation):
        if any(cell.column == status_column for row in sheet.iter_rows(min_row=2, max_row=2) for cell in row if cell.coordinate in validation):
            sheet.data_validations.dataValidation.remove(validation)
    validation = DataValidation(type='list', formula1='"OK,Question"', allow_blank=True)
    sheet.add_data_validation(validation)
    validation.add(f'{get_column_letter(manual)}2:{get_column_letter(manual)}{sheet.max_row}')
    for table in sheet.tables.values():
        table.ref = f'A1:{get_column_letter(manual)}{sheet.max_row}'
        table.tableColumns = []
        table._initialise_columns()
        for column, cell in zip(table.tableColumns, sheet[1]):
            column.name = str(cell.value)
        table.autoFilter.ref = table.ref
    sheet.auto_filter.ref = f'A1:{get_column_letter(manual)}{sheet.max_row}'
    sheet.parent.calculation = CalcProperties(calcId=0, fullCalcOnLoad=True, forceFullCalc=True, calcMode='auto')
    guide.append(['Automatic status', 'Fill Your correction and Review status automatically becomes Changed. Do not edit Review status. Clear the correction to return to your optional OK/Question choice.'])
    guide.append(['Optional review', 'Use Your review (OK / Question) for drafts needing no correction. Earlier instructions to manually choose Changed or OK in Review status are superseded.'])


def validate_review_status(sheet, correction_column, status_column):
    assert all(sheet.cell(row, status_column).data_type == 'f' for row in range(2, sheet.max_row + 1))
    assert all(f'{get_column_letter(correction_column)}{row}<>""' in sheet.cell(row, status_column).value for row in range(2, sheet.max_row + 1))
