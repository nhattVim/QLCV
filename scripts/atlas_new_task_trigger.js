// =========================================================================
// MONGODB ATLAS DATABASE TRIGGER - GỬI EMAIL KHI TẠO CÔNG VIỆC MỚI
// =========================================================================
// Tối ưu chuẩn Spam Filter (Đạt điểm uy tín cao, vào Hộp thư chính / Inbox)
// =========================================================================

function htmlToPlainText(html) {
    return String(html || "")
        .replace(/<br\s*\/?>/gi, "\n")
        .replace(/<\/(p|div|h[1-6]|tr|li|blockquote)>/gi, "\n")
        .replace(/<li[^>]*>/gi, "• ")
        .replace(/<[^>]+>/g, "")
        .replace(/&nbsp;/gi, " ")
        .replace(/&amp;/gi, "&")
        .replace(/&lt;/gi, "<")
        .replace(/&gt;/gi, ">")
        .replace(/&quot;/gi, '"')
        .replace(/\n{3,}/g, "\n\n")
        .trim();
}

function escapeHtml(text) {
    return String(text)
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;")
        .replace(/"/g, "&quot;");
}

function addInlineStyle(html, tag, style) {
    const re = new RegExp("<" + tag + "(\\s[^>]*)?>", "gi");
    return html.replace(re, function (match, attrs) {
        attrs = attrs || "";
        if (/style\s*=/i.test(attrs)) {
            return match.replace(/style\s*=\s*(['"])(.*?)\1/i, function (_, q, existing) {
                return "style=" + q + existing + ";" + style + q;
            });
        }
        return "<" + tag + attrs + ' style="' + style + '">';
    });
}

function formatDescriptionForEmail(html) {
    const raw = String(html || "").trim();
    const plain = htmlToPlainText(raw);
    if (!plain) {
        return '<p style="margin:0;color:#64748B;">Không có mô tả chi tiết.</p>';
    }
    if (!/<[a-z][\s\S]*>/i.test(raw)) {
        return '<p style="margin:0;white-space:pre-wrap;color:#1E293B;">' + escapeHtml(raw) + "</p>";
    }

    let s = raw
        .replace(/<script[\s\S]*?<\/script>/gi, "")
        .replace(/<iframe[\s\S]*?<\/iframe>/gi, "")
        .replace(/<object[\s\S]*?<\/object>/gi, "")
        .replace(/<embed[\s\S]*?>/gi, "")
        .replace(/\son\w+\s*=\s*("[^"]*"|'[^']*'|[^\s>]+)/gi, "")
        .replace(/javascript:/gi, "");

    s = addInlineStyle(s, "p", "margin:0 0 10px 0;line-height:1.55;color:#1E293B;");
    s = addInlineStyle(s, "h1", "margin:12px 0 8px;font-size:20px;color:#0F172A;");
    s = addInlineStyle(s, "h2", "margin:12px 0 8px;font-size:17px;color:#0F172A;");
    s = addInlineStyle(s, "h3", "margin:10px 0 6px;font-size:15px;color:#0F172A;");
    s = addInlineStyle(s, "ul", "margin:0 0 10px 20px;padding:0;");
    s = addInlineStyle(s, "ol", "margin:0 0 10px 20px;padding:0;");
    s = addInlineStyle(s, "li", "margin:0 0 4px 0;");
    s = addInlineStyle(s, "blockquote", "margin:8px 0;padding:8px 12px;border-left:3px solid #2563EB;background:#F8FAFC;color:#475569;");
    s = addInlineStyle(s, "a", "color:#2563EB;text-decoration:underline;");
    s = addInlineStyle(s, "img", "max-width:100%;height:auto;border:0;display:block;margin:8px 0;");
    s = addInlineStyle(s, "table", "border-collapse:collapse;width:100%;margin:8px 0;");
    s = addInlineStyle(s, "th", "border:1px solid #E5E7EB;padding:6px 8px;background:#F8FAFC;text-align:left;");
    s = addInlineStyle(s, "td", "border:1px solid #E5E7EB;padding:6px 8px;");
    s = addInlineStyle(s, "code", "background:#F1F5F9;padding:1px 4px;border-radius:3px;font-size:13px;");
    return s;
}

exports = async function (changeEvent) {
    // 1. Lấy thông tin công việc vừa được tạo
    const doc = changeEvent.fullDocument;
    if (!doc) {
        console.log("Không có dữ liệu fullDocument.");
        return;
    }

    // 2. Tra cứu thông tin Đội nhóm (Team) và Không gian làm việc (Workspace)
    let teamName = "";
    let workspaceName = "";
    try {
        let mongodb = context.services.get("qlcv") || context.services.get("mongodb-atlas") || context.services.get("Cluster0");
        if (mongodb) {
            const db = mongodb.db("qlcv_db");
            if (doc.team_id) {
                const teamsCol = db.collection("teams");
                let teamQuery = [{ _id: doc.team_id }];
                try {
                    teamQuery.push({ _id: new BSON.ObjectId(String(doc.team_id)) });
                } catch (e) { }
                const teamDoc = await teamsCol.findOne({ $or: teamQuery });
                if (teamDoc && teamDoc.name) {
                    teamName = teamDoc.name;
                }
            }

            if (doc.workspace_id) {
                const wsCol = db.collection("workspaces");
                let wsQuery = [{ _id: doc.workspace_id }];
                try {
                    wsQuery.push({ _id: new BSON.ObjectId(String(doc.workspace_id)) });
                } catch (e) { }
                const wsDoc = await wsCol.findOne({ $or: wsQuery });
                if (wsDoc && wsDoc.name) {
                    workspaceName = wsDoc.name;
                }
            }
        }
    } catch (lookupErr) {
        console.log("Lỗi tra cứu thông tin team/workspace:", lookupErr.message);
    }

    // 3. Cấu hình gửi Email qua Brevo
    const BREVO_API_KEY = "DIEN_BREVO_API_KEY_VAO_DAY";
    const SENDER_EMAIL = "tuankug@gmail.com"; // Email của bạn (đã đăng ký trên Brevo)
    const SENDER_NAME = "Hệ Thống Quản Lý Công Việc";

    const assignees = doc.assigned_to || [];
    if (assignees.length === 0) {
        console.log(`Công việc '${doc.title}' không có người phụ trách, bỏ qua.`);
        return;
    }

    // 4. Tính toán thời gian hạn chót
    const now = new Date();
    const dueDate = new Date(doc.due_date);
    const diffHours = (dueDate.getTime() - now.getTime()) / (1000 * 60 * 60);
    const isUrgent = diffHours <= 48; // Trong vòng 48 giờ tới

    // Định dạng ngày giờ hiển thị theo múi giờ Việt Nam (UTC+7)
    const dueFormatted = dueDate.toLocaleString("vi-VN", {
        timeZone: "Asia/Ho_Chi_Minh",
        year: "numeric", month: "2-digit", day: "2-digit",
        hour: "2-digit", minute: "2-digit"
    });

    // Tiêu đề chuẩn mực
    const subject = isUrgent
        ? `Giao việc ưu tiên: ${doc.title} (Hạn: ${dueFormatted})`
        : `Thông báo công việc mới: ${doc.title}`;

    const priorityColors = {
        "Urgent": "#DC2626",
        "High": "#EA580C",
        "Medium": "#2563EB",
        "Low": "#16A34A"
    };
    const priorityColor = priorityColors[doc.priority] || "#2563EB";

    const noticeBanner = isUrgent
        ? `<div style="background-color: #FEF3C7; border-left: 4px solid #F59E0B; color: #92400E; padding: 12px 16px; border-radius: 4px; font-weight: 500; margin-bottom: 20px;">
             Lưu ý: Công việc này có thời hạn hoàn thành trong vòng 48 giờ (${dueFormatted}). Vui lòng ưu tiên thực hiện sớm.
           </div>`
        : `<div style="background-color: #F0FDF4; border-left: 4px solid #16A34A; color: #166534; padding: 12px 16px; border-radius: 4px; font-weight: 500; margin-bottom: 20px;">
             Bạn vừa được phân công một công việc mới trên hệ thống QLCV.
           </div>`;

    // 5. Nội dung HTML chuẩn Responsive
    const htmlBody = `
    <!DOCTYPE html>
    <html lang="vi">
    <head>
        <meta charset="utf-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>${subject}</title>
        <style>
            body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; line-height: 1.6; color: #1F2937; margin: 0; padding: 0; background-color: #F9FAFB; }
            .wrapper { width: 100%; padding: 25px 0; background-color: #F9FAFB; }
            .container { max-width: 760px; width: 100%; margin: 0 auto; background: #ffffff; border: 1px solid #E5E7EB; border-radius: 8px; overflow: hidden; }
            .header { background: #1E293B; color: #ffffff; padding: 20px 24px; text-align: left; }
            .header h2 { margin: 0; font-size: 18px; font-weight: 600; letter-spacing: 0.3px; }
            .content { padding: 24px; }
            .task-title { font-size: 17px; font-weight: bold; color: #0F172A; margin: 0 0 12px 0; }
            .desc-box { background: #F8FAFC; border: 1px solid #E2E8F0; border-radius: 6px; padding: 12px 16px; margin: 0 0 16px 0; font-size: 14px; color: #1E293B; }
            .info-table { width: 100%; border-collapse: collapse; margin-bottom: 20px; }
            .info-table td { padding: 8px 0; font-size: 14px; vertical-align: top; }
            .info-label { width: 160px; color: #64748B; font-weight: 600; }
            .info-val { color: #1E293B; }
            .app-box { background-color: #F8FAFC; border: 1px solid #E2E8F0; padding: 12px 16px; border-radius: 6px; font-size: 13px; color: #475569; }
            .footer { background: #F8FAFC; padding: 16px 24px; text-align: center; font-size: 12px; color: #94A3B8; border-top: 1px solid #E5E7EB; }
        </style>
    </head>
    <body>
        <div class="wrapper">
            <div class="container" style="max-width:760px;width:100%;margin:0 auto;">
                <div class="header">
                    <h2>THÔNG BÁO PHÂN CÔNG CÔNG VIỆC</h2>
                </div>
                <div class="content">
                    ${noticeBanner}
                    <div class="task-title">${escapeHtml(doc.title || "")}</div>
                    <div class="desc-box">${formatDescriptionForEmail(doc.description)}</div>
                    <table class="info-table">
                        ${teamName ? `
                        <tr>
                            <td class="info-label">Đội nhóm (Team):</td>
                            <td class="info-val"><b>${escapeHtml(teamName)}</b></td>
                        </tr>` : ""}
                        ${workspaceName ? `
                        <tr>
                            <td class="info-label">Không gian làm việc:</td>
                            <td class="info-val"><b>${escapeHtml(workspaceName)}</b></td>
                        </tr>` : ""}
                        <tr>
                            <td class="info-label">Mức độ ưu tiên:</td>
                            <td class="info-val"><b style="color: ${priorityColor};">${escapeHtml(doc.priority || "Trung bình")}</b></td>
                        </tr>
                        <tr>
                            <td class="info-label">Hạn hoàn thành:</td>
                            <td class="info-val"><b style="color: #0F172A;">${dueFormatted}</b></td>
                        </tr>
                        <tr>
                            <td class="info-label">Người giao việc:</td>
                            <td class="info-val">${escapeHtml(doc.created_by || "Quản trị viên")}</td>
                        </tr>
                    </table>
                    <div class="app-box">
                        💡 Vui lòng mở ứng dụng <b>QLCV</b> trên máy tính để tiếp nhận và cập nhật tiến độ công việc.
                    </div>
                </div>
                <div class="footer">
                    Email thông báo tự động từ Hệ thống Quản Lý Công Việc Nội bộ (QLCV).<br>
                    Bạn nhận được thư này vì được phân công phụ trách công việc trên phần mềm.
                </div>
            </div>
        </div>
    </body>
    </html>
    `;

    // 6. Nội dung thuần Text (Plain Text) song song - Cực kỳ quan trọng để vượt qua Spam Filter
    const textBody =
        `THÔNG BÁO PHÂN CÔNG CÔNG VIỆC\n\n` +
        `Tiêu đề: ${doc.title}\n` +
        (teamName ? `Đội nhóm: ${teamName}\n` : "") +
        (workspaceName ? `Không gian làm việc: ${workspaceName}\n` : "") +
        `Mô tả: ${htmlToPlainText(doc.description) || "Không có mô tả chi tiết."}\n` +
        `Mức độ ưu tiên: ${doc.priority || "Trung bình"}\n` +
        `Hạn hoàn thành: ${dueFormatted}\n` +
        `Người giao việc: ${doc.created_by || "Quản trị viên"}\n\n` +
        `Vui lòng mở ứng dụng QLCV trên máy tính để cập nhật trạng thái công việc.\n` +
        `Email tự động từ Hệ thống Quản Lý Công Việc (QLCV).`;

    let sentCount = 0;
    for (const recipient of assignees) {
        try {
            const response = await context.http.post({
                url: "https://api.brevo.com/v3/smtp/email",
                headers: {
                    "api-key": [BREVO_API_KEY],
                    "Content-Type": ["application/json"],
                    "Accept": ["application/json"]
                },
                body: JSON.stringify({
                    sender: { name: SENDER_NAME, email: SENDER_EMAIL },
                    to: [{ email: recipient }],
                    replyTo: { email: SENDER_EMAIL, name: SENDER_NAME },
                    subject: subject,
                    htmlContent: htmlBody,
                    textContent: textBody,
                    headers: {
                        "X-Mailin-tag": "task-assignment",
                        "X-Auto-Response-Suppress": "OOF, AutoReply"
                    }
                })
            });
            console.log(`Đã gửi email việc mới tới ${recipient}: status ${response.statusCode}`);
            sentCount++;
        } catch (err) {
            console.error(`Lỗi gửi email tới ${recipient}:`, err.message);
        }
    }

    return { success: true, sentEmails: sentCount };
};
