const xlsx = require('xlsx');
const fs = require('fs');

const files = [
    'C:/projects/Etiket/Gerekli/2026.05.20 VNT.XLSX',
    'C:/projects/Etiket/Gerekli/2026.05.16 VNT.XLSX'
];

files.forEach(file => {
    if (!fs.existsSync(file)) return;
    console.log(`\n--- Reading ${file} ---`);
    const wb = xlsx.readFile(file);
    const sheetName = wb.SheetNames[0];
    const data = xlsx.utils.sheet_to_json(wb.Sheets[sheetName], { header: 1 });
    
    data.forEach((row, idx) => {
        const rowStr = row.join(' ').toLowerCase();
        if (rowStr.includes('halit') || rowStr.includes('henne') || rowStr.includes('nubuk') || rowStr.includes('banko') || rowStr.includes('bord')) {
            console.log(`Row ${idx}: ${row.join(' | ')}`);
        }
    });
});
