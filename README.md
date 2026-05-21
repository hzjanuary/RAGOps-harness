# Spec Intake: RAGOps Harness (CLI + Tauri)

## Loại công việc (Input Type) & Làn rủi ro (Lane)

**Loại:** New spec — tạo ra một dự án mới hoàn toàn.

**Làn (Lane):** High-Risk.

**Lý do:** Công cụ này can thiệp trực tiếp vào request API external systems, phân tích dữ liệu bảo mật, kiểm thử prompt injection/audit/security và hoạt động trên nhiều nền tảng CLI + Desktop App.

---

## Tóm tắt dự án (Project Summary)

Xây dựng một **RAG-Harness** — nền tảng điều phối LLMOps nhẹ gọn chạy dưới dạng CLI và ứng dụng Desktop bằng Tauri.

Ứng dụng hoạt động như một middleware proxy giữa hệ thống RAG của người dùng và các LLM provider như OpenAI, Claude, v.v.

Nền tảng cung cấp 3 giá trị cốt lõi:

1. **Giám sát chi phí/độ trễ (Observability)**
2. **Đánh giá chất lượng RAG (Eval Pipeline)**
3. **Kiểm thử bảo mật tự động (Red Teaming)**

---

## Kiến trúc dự kiến (Architecture Questions)

Dựa theo `docs/ARCHITECTURE.md`:

### Runtime stack

- **Backend/Core:** Rust  
  Dùng để xử lý proxy tốc độ cao, độ trễ thấp.

- **GUI:** Tauri + React/Next.js + Tailwind  
  Dùng để xây dựng Dashboard.

- **Storage:** SQLite cục bộ  
  Lưu lịch sử log, test results và config. Không dùng cloud database để đảm bảo tính Native & Offline của CLI.

### Product surfaces

- Giao diện dòng lệnh **CLI** cho developer.
- Giao diện **GUI Tauri Desktop** cho báo cáo trực quan.

### Boundary inputs

- Tham số từ terminal:
  - `harness scan`
  - `harness eval`
- API request bị intercept.
- File config:
  - `.harness.yaml`

---

## Kế hoạch chia nhỏ (Candidate Epics & Product Docs)

Dự án sẽ được chia thành 4 Epic chính để hoàn thành trong 2 ngày Hackathon.

| File Product Docs | Epic | Trạng thái / Hackathon Plan |
|---|---|---|
| `docs/product/finops-proxy.md` | **E01-Proxy-And-FinOps**: Xây dựng Core CLI bằng Rust. Chạy local proxy server ở port `8000` để chặn LLM request, tính toán token usage, latency và lưu vào SQLite. | Làm trong nửa ngày đầu — Ngày 1 |
| `docs/product/red-teaming.md` | **E02-Red-Teaming**: Viết logic CLI gửi tự động hàng loạt prompt độc hại, ví dụ Prompt Injection, vào ứng dụng RAG mục tiêu và chấm điểm phòng thủ. | Làm trong nửa ngày sau — Ngày 1 |
| `docs/product/eval-pipeline.md` | **E03-RAG-Eval**: Cung cấp lệnh CLI chấm điểm bộ Golden Dataset theo các trục Faithfulness và Answer Relevance. | Làm trong nửa ngày đầu — Ngày 2 |
| `docs/product/tauri-dashboard.md` | **E04-GUI-Dashboard**: Bọc Tauri lên trên Core Rust. Đọc dữ liệu từ SQLite và hiển thị lên biểu đồ trực quan, tạo Wow Factor để đi thi. | Làm trong nửa ngày cuối — Ngày 2 |

---

## Hình dạng kiểm chứng (Validation Shape)

### Unit Proof

Test các hàm tính toán giá tiền token.

Ví dụ:

- GPT-4o có giá bao nhiêu trên mỗi `1K token`.
- Hàm tính chi phí có xử lý đúng input/output token không.

### Integration Proof

Đảm bảo Local Proxy chuyển tiếp request thành công tới API thật mà không làm hỏng payload.

### E2E Proof

Chạy lệnh:

```bash
harness start
