//! Typed resource catalog; vi-VN and en are exhaustive, with no view literals.
use crate::editor::Status;

#[derive(Clone, Copy)]
pub struct Copy {
    pub theme: &'static str,
    pub system: &'static str,
    pub light: &'static str,
    pub dark: &'static str,
    pub unknown: &'static str,
    pub title: &'static str,
    pub mode: &'static str,
    pub token: &'static str,
    pub login: &'static str,
    pub script: &'static str,
    pub read: &'static str,
    pub draft: &'static str,
    pub save: &'static str,
    pub retry: &'static str,
    pub base: &'static str,
    pub preview: &'static str,
    pub rebase: &'static str,
    pub help: &'static str,
    pub busy: &'static str,
    pub pending: &'static str,
    pub actor: &'static str,
    pub validation: &'static str,
    pub sign_in_first: &'static str,
}

pub fn copy(english: bool) -> Copy {
    if english {
        Copy {
        theme:"Appearance", system:"System", light:"Light", dark:"Dark", unknown:"Unknown",
        title:"Cantos Studio", mode:"Live · local Axum / PostgreSQL · no mock data",
        token:"Development session token", login:"Sign in", script:"Script ID (UUID)",
        read:"Read current revision", draft:"Complete Script IR draft (JSON)",
        save:"Save a new immutable revision", retry:"Retry the submitted snapshot",
        base:"Base revision", preview:"Stored export · read only", rebase:"Keep my text and use this head as base",
        help:"Paste a complete 0.1.0 export. Reading keeps any existing draft. Compare the stored export before choosing a new base. A save does not approve production or publication.",
        busy:"Request in progress; editing remains available.", pending:"Reconcile the submitted snapshot with Retry before reading or making a new save. Sign in as the original submitting actor.",
        actor:"Signed-in actor", validation:"Validation findings", sign_in_first:"Sign in before saving.",
    }
    } else {
        Copy {
        theme:"Giao diện", system:"Theo hệ thống", light:"Sáng", dark:"Tối", unknown:"Chưa xác định",
        title:"Cantos Studio", mode:"Live · Axum / PostgreSQL local · không dùng dữ liệu mock",
        token:"Token phiên phát triển", login:"Đăng nhập", script:"ID kịch bản (UUID)",
        read:"Đọc bản sửa đổi hiện tại", draft:"Bản nháp Script IR đầy đủ (JSON)",
        save:"Lưu bản sửa đổi bất biến mới", retry:"Thử lại snapshot đã gửi",
        base:"Bản sửa đổi gốc", preview:"Export đã lưu · chỉ đọc", rebase:"Giữ văn bản và dùng head này làm gốc",
        help:"Dán export 0.1.0 đầy đủ. Khi đọc, bản nháp hiện có được giữ nguyên. So sánh export đã lưu trước khi chọn gốc mới. Lưu kịch bản chưa phải duyệt sản xuất hay xuất bản.",
        busy:"Đang xử lý; bạn vẫn có thể sửa văn bản.", pending:"Dùng Thử lại để xác định kết quả snapshot đã gửi trước khi đọc hoặc lưu lần mới. Đăng nhập đúng actor đã gửi snapshot.",
        actor:"Actor đã đăng nhập", validation:"Các lỗi cần sửa", sign_in_first:"Đăng nhập trước khi lưu.",
    }
    }
}

pub fn status(status: Status, english: bool) -> &'static str {
    match (status,english) {
        (Status::Dirty,false) => "Có văn bản chưa lưu.",
        (Status::Dirty,true) => "Your text has unsaved changes.",
        (Status::Idle,false) => "Đăng nhập, rồi đọc hoặc lưu một kịch bản.",
        (Status::Idle,true) => "Sign in, then read or save a script.",
        (Status::Loading,false) => "Đang đọc bản đã lưu…", (Status::Loading,true) => "Reading stored revision…",
        (Status::Saving,false) => "Đang lưu snapshot…", (Status::Saving,true) => "Saving snapshot…",
        (Status::Loaded,false) => "Đã mở bản đã lưu.", (Status::Loaded,true) => "Stored revision opened.",
        (Status::Saved,false) => "Đã lưu bản sửa đổi mới.", (Status::Saved,true) => "New revision saved.",
        (Status::DraftPreserved,false) => "Đã giữ văn bản đang sửa. So sánh với export đã lưu bên dưới.",
        (Status::DraftPreserved,true) => "Your draft is preserved. Compare it with the stored export below.",
        (Status::SignedIn,false) => "Đăng nhập thành công.", (Status::SignedIn,true) => "Signed in.",
        (Status::Unauthenticated,false) => "Phiên chưa hợp lệ hoặc đã hết hạn. Đăng nhập lại; văn bản được giữ nguyên.",
        (Status::Unauthenticated,true) => "Session missing or expired. Sign in again; your text is preserved.",
        (Status::NotFound,false) => "Không tìm thấy kịch bản trong phạm vi quyền của bạn.",
        (Status::NotFound,true) => "Script not found within your access scope.",
        (Status::Forbidden,false) => "Bạn không có quyền thực hiện thao tác này.", (Status::Forbidden,true) => "You do not have permission for this action.",
        (Status::InvalidRequest,false) => "Kiểm tra ID và định dạng yêu cầu; văn bản chưa bị thay đổi.",
        (Status::InvalidRequest,true) => "Check the ID and request format; your text has not changed.",
        (Status::InvalidScript,false) => "Kịch bản chưa hợp lệ. Sửa các lỗi dưới đây rồi lưu lại.",
        (Status::InvalidScript,true) => "Script invalid. Correct the findings below and save again.",
        (Status::EvidenceUnavailable,false) => "Tham chiếu nguồn hoặc quyền chưa có trong tài khoản sở hữu. Cần đăng ký hồ sơ trước khi lưu.",
        (Status::EvidenceUnavailable,true) => "Source or rights references are unavailable in the owner account. Register evidence before saving.",
        (Status::StaleRevision,false) => "Có bản mới hơn. Văn bản của bạn được giữ nguyên; đọc head, so sánh rồi chọn gốc mới.",
        (Status::StaleRevision,true) => "A newer revision exists. Your text is preserved; read the head, compare, then choose a new base.",
        (Status::OperationReused,false) => "Operation ID đã dùng với snapshot khác. Văn bản được giữ; cần bắt đầu lần lưu mới.",
        (Status::OperationReused,true) => "Operation ID was used with another snapshot. Text preserved; start a new save.",
        (Status::Unavailable,false) => "Chưa xác định được kết quả. Văn bản và snapshot được giữ; thử lại khi kết nối phục hồi.",
        (Status::Unavailable,true) => "Outcome unavailable. Text and snapshot preserved; retry after connection recovers.",
        (Status::CorruptRevision,false) => "Không kiểm chứng được bản đã lưu. Văn bản được giữ; cần kiểm tra backend.",
        (Status::CorruptRevision,true) => "Stored revision integrity check failed. Text preserved; inspect the backend.",
    }
}

#[cfg(test)]
mod tests {
    use super::{copy, status};
    use crate::editor::Status;
    #[test]
    fn every_async_and_error_state_has_both_locales() {
        for state in [
            Status::Idle,
            Status::Dirty,
            Status::Loading,
            Status::Saving,
            Status::Loaded,
            Status::Saved,
            Status::DraftPreserved,
            Status::SignedIn,
            Status::Unauthenticated,
            Status::NotFound,
            Status::Forbidden,
            Status::InvalidRequest,
            Status::InvalidScript,
            Status::EvidenceUnavailable,
            Status::StaleRevision,
            Status::OperationReused,
            Status::Unavailable,
            Status::CorruptRevision,
        ] {
            assert!(!status(state, false).is_empty());
            assert!(!status(state, true).is_empty());
        }
        assert!(copy(false).mode.contains("Live"));
        assert!(copy(true).mode.contains("no mock"));
    }
}
