import * as XLSX from 'xlsx'

export function downloadExcelTemplate(
  filename: string,
  headers: string[],
  sampleRows: string[][],
): void {
  const ws = XLSX.utils.aoa_to_sheet([headers, ...sampleRows])
  ws['!cols'] = headers.map(() => ({ wch: 22 }))

  const wb = XLSX.utils.book_new()
  XLSX.utils.book_append_sheet(wb, ws, 'Sheet1')
  XLSX.writeFile(wb, `${filename}.xlsx`)
}
