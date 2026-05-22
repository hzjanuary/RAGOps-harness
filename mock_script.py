import urllib.request
import json

url = "http://localhost:8000/v1/chat/completions"
data = json.dumps({
    "model": "gpt-4o-mini",
    "messages": [{"role": "user", "content": "Giải thích RAG pipeline."}]
}).encode('utf-8')

req = urllib.request.Request(url, data=data, headers={"Content-Type": "application/json"})
try:
    with urllib.request.urlopen(req) as response:
        print(json.dumps(json.loads(response.read()), indent=2))
except Exception as e:
    print("Lỗi:", e)