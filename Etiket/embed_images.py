import base64
import re
import os

html_file = 'Dokumantasyon.html'
with open(html_file, 'r', encoding='utf-8') as f:
    html = f.read()

def repl(m):
    path = m.group(1)
    with open(path, 'rb') as img_f:
        img = img_f.read()
    b64 = base64.b64encode(img).decode('utf-8')
    ext = path.split('.')[-1]
    return f'src="data:image/{ext};base64,{b64}"'

html = re.sub(r'src="(EKRAN GÖRÜNTÜLERİ/[^"]+)"', repl, html)

with open(html_file, 'w', encoding='utf-8') as f:
    f.write(html)
