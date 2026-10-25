// =========================================================================
// MONGODB ATLAS SCHEDULED TRIGGER FUNCTION - TỰ ĐỘNG GỬI EMAIL NHẮC VIỆC
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

exports = async function() {
    // 1. Cấu hình dịch vụ gửi Email Brevo
    const BREVO_API_KEY = "DIEN_BREVO_API_KEY_VAO_DAY"; 
    const SENDER_EMAIL  = "tuankug@gmail.com"; // Email của bạn (đã đăng ký trên Brevo)
    const SENDER_NAME   = "Hệ Thống Quản Lý Công Việc";

    // 2. Kết nối tới CSDL MongoDB
    let mongodb = context.services.get("qlcv") || context.services.get("mongodb-atlas") || context.services.get("Cluster0");

    if (!mongodb) {
        throw new Error("Không tìm thấy kết nối MongoDB! Vui lòng vào App Services -> Linked Data Sources ở cột trái để xem tên chính xác của Data Source.");
    }

    const db = mongodb.db("qlcv_db");
    const tasksCol = db.collection("tasks");

    const now = new Date();
    const fallbackWindow = new Date(now.getTime() + 2 * 24 * 60 * 60 * 1000);

    // 3. Tìm các công việc đã đến ngày nhắc và chưa gửi cho lịch nhắc hiện tại
    const candidates = await tasksCol.find({
        status: { $ne: "DONE" },
        $or: [
            { remind_at: { $lte: now } },
            {
                remind_at: { $exists: false },
                due_date: { $lte: fallbackWindow }
            }
        ]
    }).toArray();

    const tasks = candidates.filter((task) => {
        const remindedAt = task.reminded_at ? new Date(task.reminded_at) : null;
        if (!remindedAt || Number.isNaN(remindedAt.getTime())) {
            if (!task.remind_at && task.reminded_48h === true) {
                return false;
            }
            return true;
        }
        if (task.remind_at) {
            const remindAt = new Date(task.remind_at);
            // Lịch nhắc mới (sau lần gửi cũ) thì gửi lại
            return remindAt.getTime() > remindedAt.getTime();
        }
        return false;
    });

    console.log(`[Scheduled Trigger] Đã tìm thấy ${tasks.length} công việc đến ngày nhắc cần gửi email.`);

    if (tasks.length === 0) {
        return { success: true, message: "Không có công việc nào cần nhắc nhở." };
    }

    let sentCount = 0;

    // 4. Duyệt qua từng công việc và gửi email cho người phụ trách
    for (const task of tasks) {
        const assignees = task.assigned_to || [];
        if (assignees.length === 0) continue;

        const dueDate = new Date(task.due_date);

        // Định dạng ngày giờ hiển thị theo múi giờ Việt Nam (UTC+7)
        const dueFormatted = dueDate.toLocaleString("vi-VN", {
            timeZone: "Asia/Ho_Chi_Minh",
            year: "numeric", month: "2-digit", day: "2-digit",
            hour: "2-digit", minute: "2-digit"
        });

        const isOverdue = dueDate < now;
        const remindDays = task.remind_before_days || 2;

        const subject = isOverdue 
            ? `[QLCV] Cập nhật tiến độ: Công việc '${task.title}' đã quá hạn hoàn thành`
            : `[QLCV] Nhắc việc: Công việc '${task.title}' sắp đến hạn hoàn thành (${dueFormatted})`;

        const statusBanner = isOverdue
            ? `<div style="background-color: #FEF2F2; border-left: 4px solid #EF4444; color: #991B1B; padding: 12px 16px; border-radius: 4px; font-weight: 500; margin-bottom: 20px;">
                 Thông báo: Công việc này hiện đã quá hạn hoàn thành. Vui lòng sắp xếp hoàn tất và báo cáo tiến độ sớm.
               </div>`
            : `<div style="background-color: #FEF3C7; border-left: 4px solid #F59E0B; color: #92400E; padding: 12px 16px; border-radius: 4px; font-weight: 500; margin-bottom: 20px;">
                 Nhắc việc: Công việc này sẽ hết hạn trong khoảng ${remindDays} ngày tới (${dueFormatted}). Vui lòng chú ý thời hạn.
               </div>`;

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
                .header { background: #2563EB; color: #ffffff; padding: 20px 24px; text-align: left; }
                .header h2 { margin: 0; font-size: 18px; font-weight: 600; letter-spacing: 0.3px; }
                .content { padding: 24px; }
                .task-title { font-size: 17px; font-weight: bold; color: #0F172A; margin: 0 0 12px 0; }
                .desc-box { background: #F8FAFC; border: 1px solid #E2E8F0; border-radius: 6px; padding: 12px 16px; margin: 0 0 16px 0; font-size: 14px; color: #1E293B; }
                .info-table { width: 100%; border-collapse: collapse; margin-bottom: 20px; }
                .info-table td { padding: 8px 0; font-size: 14px; vertical-align: top; }
                .info-label { width: 140px; color: #64748B; font-weight: 600; }
                .info-val { color: #1E293B; }
                .app-box { background-color: #F8FAFC; border: 1px solid #E2E8F0; padding: 12px 16px; border-radius: 6px; font-size: 13px; color: #475569; }
                .footer { background: #F8FAFC; padding: 16px 24px; text-align: center; font-size: 12px; color: #94A3B8; border-top: 1px solid #E5E7EB; }
            </style>
        </head>
        <body>
            <div class="wrapper">
                <div class="container" style="max-width:760px;width:100%;margin:0 auto;">
                    <div class="header">
                        <h2>THÔNG BÁO TIẾN ĐỘ CÔNG VIỆC</h2>
                    </div>
                    <div class="content">
                        ${statusBanner}
                        <div class="task-title">${escapeHtml(task.title || "")}</div>
                        <div class="desc-box">${formatDescriptionForEmail(task.description)}</div>
                        <table class="info-table">
                            <tr>
                                <td class="info-label">Mức độ ưu tiên:</td>
                                <td class="info-val"><b>${escapeHtml(task.priority || "Trung bình")}</b></td>
                            </tr>
                            <tr>
                                <td class="info-label">Hạn hoàn thành:</td>
                                <td class="info-val"><b style="color: #DC2626;">${dueFormatted}</b></td>
                            </tr>
                            <tr>
                                <td class="info-label">Người giao việc:</td>
                                <td class="info-val">${escapeHtml(task.created_by || "Quản trị viên")}</td>
                            </tr>
                        </table>
                        <div class="app-box">
                            💡 Vui lòng mở ứng dụng <b>QLCV</b> trên máy tính để cập nhật trạng thái tiến độ công việc.
                        </div>
                    </div>
                    <div class="footer">
                        Email nhắc việc tự động từ Hệ thống Quản Lý Công Việc Nội bộ (QLCV).<br>
                        Thư này gửi tự động định kỳ 24/7 theo thời hạn được giao trên phần mềm.
                    </div>
                </div>
            </div>
        </body>
        </html>
        `;

        const textBody = 
            `THÔNG BÁO TIẾN ĐỘ CÔNG VIỆC\n\n` +
            `Tiêu đề: ${task.title}\n` +
            `Hạn hoàn thành: ${dueFormatted}\n` +
            `Trạng thái: ${isOverdue ? "Đã quá hạn hoàn thành" : `Sắp đến hạn (nhắc trước ${remindDays} ngày)`}\n` +
            `Mô tả: ${htmlToPlainText(task.description) || "Không có mô tả chi tiết."}\n` +
            `Người giao việc: ${task.created_by || "Quản trị viên"}\n\n` +
            `Vui lòng mở ứng dụng QLCV trên máy tính để cập nhật trạng thái công việc.\n` +
            `Email tự động từ Hệ thống Quản Lý Công Việc (QLCV).`;

        // Gửi email cho từng người phụ trách
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
                            "X-Mailin-tag": "deadline-reminder",
                            "X-Auto-Response-Suppress": "OOF, AutoReply"
                        }
                    })
                });

                console.log(`Đã gửi email nhắc việc tới ${recipient}: status ${response.statusCode}`);
                sentCount++;
            } catch (err) {
                console.error(`Lỗi gửi email tới ${recipient}:`, err.message);
            }
        }

        // Đánh dấu công việc đã được gửi email nhắc nhở
        await tasksCol.updateOne(
            { _id: task._id },
            {
                $set: { reminded_at: new Date() },
                $unset: { reminded_48h: "" }
            }
        );
    }

    return { success: true, sentEmails: sentCount };
};
