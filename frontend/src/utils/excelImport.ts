import * as XLSX from 'xlsx'

export type SynonymMap = Record<string, string[]>

export interface ParseResult {
  rows: Record<string, string>[]
}

async function readWorkbook(file: File): Promise<XLSX.WorkBook> {
  const buffer = await file.arrayBuffer()
  const isXlsx = /\.(xlsx|xls)$/i.test(file.name)

  if (isXlsx) {
    return XLSX.read(buffer, { type: 'array' })
  }

  // CSV: detect encoding via BOM or replacement character heuristic
  const bytes = new Uint8Array(buffer)
  if (bytes[0] === 0xef && bytes[1] === 0xbb && bytes[2] === 0xbf) {
    const text = new TextDecoder('utf-8').decode(buffer.slice(3))
    return XLSX.read(text, { type: 'string' })
  }

  const utf8 = new TextDecoder('utf-8', { fatal: false }).decode(buffer)
  if (!utf8.includes('�')) {
    return XLSX.read(utf8, { type: 'string' })
  }

  // Fallback to EUC-KR (common for Korean CSV exports)
  try {
    const euc = new TextDecoder('euc-kr').decode(buffer)
    return XLSX.read(euc, { type: 'string' })
  } catch {
    return XLSX.read(utf8, { type: 'string' })
  }
}

export async function parseExcelFile(
  file: File,
  synonymMap: SynonymMap,
  requiredFields: string[],
): Promise<ParseResult> {
  let workbook: XLSX.WorkBook
  try {
    workbook = await readWorkbook(file)
  } catch {
    throw new Error('ERR_IMPORT_PARSE_FAILED')
  }

  const sheet = workbook.Sheets[workbook.SheetNames[0]]
  if (!sheet) throw new Error('ERR_IMPORT_EMPTY')

  const raw = XLSX.utils.sheet_to_json<unknown[]>(sheet, { header: 1, defval: '' })
  if (!raw || raw.length < 2) throw new Error('ERR_IMPORT_EMPTY')

  const headerRow = (raw[0] as unknown[]).map((h) => String(h).trim())

  // Map header column index to canonical field name
  const fieldIndex: Record<string, number> = {}
  for (let i = 0; i < headerRow.length; i++) {
    const h = headerRow[i].toLowerCase()
    for (const [fieldName, synonyms] of Object.entries(synonymMap)) {
      if (fieldName in fieldIndex) continue
      if (synonyms.map((s) => s.toLowerCase()).includes(h)) {
        fieldIndex[fieldName] = i
      }
    }
  }

  // Verify required fields are present
  for (const field of requiredFields) {
    if (!(field in fieldIndex)) {
      throw new Error('ERR_IMPORT_MISSING_FIELD')
    }
  }

  const rows: Record<string, string>[] = []
  for (let i = 1; i < raw.length; i++) {
    const row = raw[i] as unknown[]
    const record: Record<string, string> = {}
    let hasValue = false

    for (const [fieldName, colIdx] of Object.entries(fieldIndex)) {
      const val = String(row[colIdx] ?? '').trim()
      record[fieldName] = val
      if (val) hasValue = true
    }

    if (hasValue) rows.push(record)
  }

  if (rows.length === 0) throw new Error('ERR_IMPORT_EMPTY')

  return { rows }
}
