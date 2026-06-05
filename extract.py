import base64
import re
import os

with open('../Dokumantasyon.html', 'r', encoding='utf-8') as f:
    content = f.read()

matches = re.findall(r'<img src="data:image/png;base64,([^"]+)"', content)
if not os.path.exists('docs_images'):
    os.makedirs('docs_images')

for i, b64 in enumerate(matches):
    with open(f'docs_images/image_{i}.png', 'wb') as img_f:
        img_f.write(base64.b64decode(b64))
print(f'Extracted {len(matches)} images.')
