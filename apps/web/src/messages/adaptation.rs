use crate::adaptation::ReviewStatus;
use cantos_api::{AdaptationCost, AdaptationCurrency, AdaptationFindingCode, AdaptationStatus};

#[derive(Clone, Copy)]
pub struct AdaptationCopy {
    pub title: &'static str,
    pub help: &'static str,
    pub provider_check: &'static str,
    pub provider_unknown: &'static str,
    pub provider_loading: &'static str,
    pub provider_absent: &'static str,
    pub provider_configured: &'static str,
    pub provider_failed: &'static str,
    pub provider: &'static str,
    pub model: &'static str,
    pub endpoint: &'static str,
    pub prompt: &'static str,
    pub config: &'static str,
    pub source_id: &'static str,
    pub source_open: &'static str,
    pub script_id: &'static str,
    pub base: &'static str,
    pub base_help: &'static str,
    pub rights: &'static str,
    pub start: &'static str,
    pub required: &'static str,
    pub pending: &'static str,
    pub retry: &'static str,
    pub operation: &'static str,
    pub run_id: &'static str,
    pub open: &'static str,
    pub refresh: &'static str,
    pub cancel: &'static str,
    pub new_run: &'static str,
    pub new_help: &'static str,
    pub provenance: &'static str,
    pub checksum: &'static str,
    pub extraction: &'static str,
    pub input_digest: &'static str,
    pub records: &'static str,
    pub cost: &'static str,
    pub usage: &'static str,
    pub unknown: &'static str,
    pub original: &'static str,
    pub source_missing: &'static str,
    pub original_binary: &'static str,
    pub download: &'static str,
    pub proposal: &'static str,
    pub proposal_help: &'static str,
    pub edit: &'static str,
    pub findings: &'static str,
    pub coverage: &'static str,
    pub coverage_help: &'static str,
    pub represented: &'static str,
    pub omitted: &'static str,
    pub block: &'static str,
    pub review: &'static str,
    pub accept: &'static str,
    pub accept_help: &'static str,
    pub accept_required: &'static str,
    pub accepted: &'static str,
    pub reopen: &'static str,
    pub problem: &'static str,
    pub no_findings: &'static str,
}

pub fn adaptation_copy(english: bool) -> AdaptationCopy {
    if english {
        AdaptationCopy {
            title: "AI radio-drama proposal", help: "Open an immutable imported source, choose a script and its exact base, then request a separate proposal. Narration, dialogue, delivery and sound cues remain suggestions until you review and accept them.",
            provider_check: "Check configured local AI runtime", provider_unknown: "Check the runtime before requesting adaptation.", provider_loading: "Reading runtime configuration…", provider_absent: "No local AI runtime is configured. An operator must provide an approved local endpoint and installed model before a live adaptation can run.", provider_configured: "Configured local runtime · a configuration check does not prove generation succeeds.", provider_failed: "Runtime configuration could not be read. Sign in and check again.", provider: "Provider", model: "Model", endpoint: "Local destination", prompt: "Prompt / contract version", config: "Generation settings",
            source_id: "Adaptation source ID", source_open: "Open source for comparison", script_id: "Target script ID (UUID)", base: "Pinned base revision", base_help: "Use 0 only for a new script. For an existing script, read its current revision below and copy that revision number. Acceptance fails if this base has changed.", rights: "I have permission to adapt this source and authorize processing by the displayed local runtime. This claim does not clear production or publication rights.", start: "Request AI proposal", required: "Sign in, check the configured runtime, open a parsed source, enter a script UUID and base, and confirm adaptation permission.", pending: "The submitted outcome is unresolved. Retry the exact operation as its original actor; keep the operation ID to recover it.", retry: "Retry exact submitted operation", operation: "Operation ID",
            run_id: "Adaptation run ID (UUID)", open: "Open saved run", refresh: "Refresh run status", cancel: "Cancel this run", new_run: "Start over / discard local proposal edits", new_help: "Refreshing preserves local edits. To select another run or request again, explicitly start over. A new run is a new generation; retries reuse the original operation.", provenance: "Pinned run provenance", checksum: "Source checksum", extraction: "Extraction version", input_digest: "Input content / export digest", records: "Generation / rights claim records", cost: "Provider-reported cost", usage: "Input / output tokens", unknown: "Unknown · not reported", original: "Preserved source and extraction", source_missing: "Open the pinned source before accepting. Compare every extraction page and its warnings.", original_binary: "The original is a binary document. Download it and compare its extraction below.", download: "Download preserved original", proposal: "Separate AI proposal", proposal_help: "This JSON is editable. The backend validates the complete Script IR and its pinned provenance on acceptance. Source coverage is a provider claim to review, not proof that every fact is preserved.", edit: "Proposal Script IR (editable JSON)", findings: "Review findings", coverage: "Source coverage claimed by the provider", coverage_help: "Block numbers follow the extraction above. Review represented text as well as omissions; all coverage records are shown.", represented: "Represented", omitted: "Omitted", block: "Source block", review: "I compared the source with the current proposal and reviewed unresolved facts, speakers, omissions and performance suggestions.", accept: "Accept current proposal as a new revision", accept_help: "This explicit action saves a new immutable revision. It does not overwrite the source or earlier revisions, approve production, or grant publication rights.", accept_required: "A successful run, matching source comparison and explicit review of the current text are required. A changed base must be handled as a new adaptation; it is never silently rebased.", accepted: "Accepted immutable revision · read only", reopen: "Reopen this exact accepted revision", problem: "Run failed or needs recovery", no_findings: "No additional provider findings. You must still compare the complete source and proposal.",
        }
    } else {
        AdaptationCopy {
            title: "Đề xuất kịch phát thanh bằng AI", help: "Mở nguồn đã nhập bất biến, chọn kịch bản và bản sửa đổi gốc chính xác, rồi yêu cầu đề xuất riêng. Lời dẫn, lời thoại, cách diễn và âm thanh vẫn là gợi ý cho đến khi bạn xem và chấp nhận.",
            provider_check: "Kiểm tra cấu hình AI local", provider_unknown: "Kiểm tra runtime trước khi yêu cầu chuyển thể.", provider_loading: "Đang đọc cấu hình runtime…", provider_absent: "Chưa cấu hình runtime AI local. Operator cần cung cấp endpoint local đã được cho phép và model đã cài trước khi chạy chuyển thể thật.", provider_configured: "Runtime local đã cấu hình · kiểm tra cấu hình chưa chứng minh việc sinh nội dung thành công.", provider_failed: "Chưa đọc được cấu hình runtime. Đăng nhập rồi kiểm tra lại.", provider: "Nhà cung cấp", model: "Model", endpoint: "Đích xử lý local", prompt: "Phiên bản prompt / hợp đồng", config: "Thiết lập sinh nội dung",
            source_id: "ID nguồn chuyển thể", source_open: "Mở nguồn để đối chiếu", script_id: "ID kịch bản đích (UUID)", base: "Bản sửa đổi gốc đã ghim", base_help: "Chỉ dùng 0 cho kịch bản mới. Với kịch bản hiện có, đọc bản hiện tại bên dưới và điền số bản sửa đổi đó. Chấp nhận sẽ thất bại nếu bản gốc đã đổi.", rights: "Tôi có quyền chuyển thể nguồn này và cho phép xử lý bằng runtime local đang hiển thị. Xác nhận này chưa cấp quyền sản xuất hoặc xuất bản.", start: "Yêu cầu đề xuất AI", required: "Đăng nhập, kiểm tra runtime đã cấu hình, mở nguồn phân tích thành công, điền UUID kịch bản và bản gốc, rồi xác nhận quyền chuyển thể.", pending: "Chưa xác định được kết quả đã gửi. Đăng nhập đúng actor ban đầu rồi thử lại chính operation đó; giữ operation ID để khôi phục.", retry: "Thử lại đúng operation đã gửi", operation: "Operation ID",
            run_id: "ID lần chuyển thể (UUID)", open: "Mở lần chuyển thể đã lưu", refresh: "Cập nhật trạng thái", cancel: "Hủy lần chuyển thể này", new_run: "Bắt đầu lại / bỏ sửa đề xuất local", new_help: "Cập nhật trạng thái giữ phần đang sửa. Để chọn run khác hoặc tạo lại, hãy chủ động bắt đầu lại. Run mới là một lần sinh mới; thử lại dùng operation ban đầu.", provenance: "Nguồn gốc đã ghim của run", checksum: "Checksum nguồn", extraction: "Phiên bản phân tích", input_digest: "Digest nội dung / export đầu vào", records: "Hồ sơ sinh nội dung / xác nhận quyền", cost: "Chi phí do provider báo", usage: "Token đầu vào / đầu ra", unknown: "Chưa biết · chưa được báo", original: "Nguồn gốc được giữ và phần phân tích", source_missing: "Mở đúng nguồn đã ghim trước khi chấp nhận. Đối chiếu mọi trang phân tích và cảnh báo.", original_binary: "Bản gốc là tài liệu nhị phân. Tải xuống và đối chiếu phần phân tích bên dưới.", download: "Tải bản gốc được giữ", proposal: "Đề xuất AI riêng", proposal_help: "Bạn có thể sửa JSON này. Khi chấp nhận, backend kiểm tra Script IR đầy đủ và nguồn gốc đã ghim. Độ bao phủ nguồn do provider đề xuất cần được xem lại; đó chưa phải bằng chứng giữ đủ mọi sự kiện.", edit: "Script IR đề xuất (JSON có thể sửa)", findings: "Các điểm cần xem lại", coverage: "Độ bao phủ nguồn do provider đề xuất", coverage_help: "Số block khớp phần phân tích bên trên. Kiểm tra cả đoạn được chuyển thể và đoạn bỏ qua; toàn bộ hồ sơ bao phủ được hiển thị.", represented: "Đã thể hiện", omitted: "Bỏ qua", block: "Block nguồn", review: "Tôi đã đối chiếu nguồn với đề xuất hiện tại và xem lại sự kiện chưa rõ, người nói, phần bỏ qua và gợi ý diễn xuất.", accept: "Chấp nhận đề xuất hiện tại thành bản sửa đổi mới", accept_help: "Thao tác rõ ràng này lưu bản sửa đổi bất biến mới. Nguồn và các bản trước được giữ nguyên; thao tác chưa duyệt sản xuất hay cấp quyền xuất bản.", accept_required: "Cần run thành công, nguồn đối chiếu khớp và xác nhận đã xem văn bản hiện tại. Nếu bản gốc đã đổi, cần lần chuyển thể mới; không tự đổi gốc.", accepted: "Bản sửa đổi bất biến đã chấp nhận · chỉ đọc", reopen: "Mở lại chính bản sửa đổi đã chấp nhận", problem: "Run thất bại hoặc cần khôi phục", no_findings: "Provider không ghi thêm điểm cần xem. Bạn vẫn phải đối chiếu toàn bộ nguồn và đề xuất.",
        }
    }
}

pub fn adaptation_status(status: &ReviewStatus, english: bool) -> &'static str {
    match (status, english) {
        (ReviewStatus::Idle, true) => {
            "Source and accepted revisions are preserved. Open a source or saved run."
        }
        (ReviewStatus::Idle, false) => {
            "Nguồn và bản đã chấp nhận được giữ nguyên. Mở nguồn hoặc run đã lưu."
        }
        (ReviewStatus::LoadingSource, true) => "Opening the preserved source…",
        (ReviewStatus::LoadingSource, false) => "Đang mở nguồn được giữ…",
        (ReviewStatus::SourceReady, true) => "Source opened for comparison.",
        (ReviewStatus::SourceReady, false) => "Đã mở nguồn để đối chiếu.",
        (ReviewStatus::Starting, true) => "Submitting the pinned adaptation request…",
        (ReviewStatus::Starting, false) => "Đang gửi yêu cầu chuyển thể đã ghim…",
        (ReviewStatus::LoadingRun, true) => "Reading the saved run…",
        (ReviewStatus::LoadingRun, false) => "Đang đọc run đã lưu…",
        (ReviewStatus::RunOpened, true) => {
            "Saved run opened. Review its status, source and proposal."
        }
        (ReviewStatus::RunOpened, false) => "Đã mở run đã lưu. Xem trạng thái, nguồn và đề xuất.",
        (ReviewStatus::DraftChanged, true) => {
            "Local proposal text is preserved; review it before acceptance."
        }
        (ReviewStatus::DraftChanged, false) => {
            "Văn bản đề xuất local được giữ; xem lại trước khi chấp nhận."
        }
        (ReviewStatus::Accepting, true) => {
            "Validating and accepting the submitted proposal snapshot…"
        }
        (ReviewStatus::Accepting, false) => "Đang kiểm tra và chấp nhận snapshot đề xuất đã gửi…",
        (ReviewStatus::Accepted, true) => {
            "A new immutable revision was accepted. Later local edits are kept separately."
        }
        (ReviewStatus::Accepted, false) => {
            "Đã chấp nhận bản sửa đổi bất biến mới. Phần sửa local sau đó được giữ riêng."
        }
        (ReviewStatus::Cancelling, true) => {
            "Requesting cancellation; a provider attempt may already have started."
        }
        (ReviewStatus::Cancelling, false) => "Đang yêu cầu hủy; provider có thể đã bắt đầu xử lý.",
        (ReviewStatus::LoadingAccepted, true) => "Reopening the exact accepted revision…",
        (ReviewStatus::LoadingAccepted, false) => "Đang mở lại đúng bản sửa đổi đã chấp nhận…",
        (ReviewStatus::AcceptedOpened, true) => {
            "The exact immutable accepted export reopened successfully."
        }
        (ReviewStatus::AcceptedOpened, false) => "Đã mở lại đúng export bất biến được chấp nhận.",
        (ReviewStatus::Error(code), _) => super::status(crate::editor::error_status(code), english),
    }
}

pub fn adaptation_run_status(status: AdaptationStatus, english: bool) -> &'static str {
    match (status, english) {
        (AdaptationStatus::Queued, true) => "Queued · refresh to observe durable progress",
        (AdaptationStatus::Queued, false) => "Đã xếp hàng · cập nhật để xem tiến độ đã lưu",
        (AdaptationStatus::Running, true) => "Running · refresh to observe durable progress",
        (AdaptationStatus::Running, false) => "Đang xử lý · cập nhật để xem tiến độ đã lưu",
        (AdaptationStatus::Succeeded, true) => "Proposal ready for human review",
        (AdaptationStatus::Succeeded, false) => "Đề xuất sẵn sàng để người biên tập xem",
        (AdaptationStatus::InvalidOutput, true) => "Invalid provider output · no revision accepted",
        (AdaptationStatus::InvalidOutput, false) => {
            "Đầu ra provider không hợp lệ · chưa chấp nhận revision"
        }
        (AdaptationStatus::Failed, true) => "Failed · no revision accepted",
        (AdaptationStatus::Failed, false) => "Thất bại · chưa chấp nhận revision",
        (AdaptationStatus::Ambiguous, true) => {
            "Provider outcome unknown · do not silently generate again"
        }
        (AdaptationStatus::Ambiguous, false) => "Chưa rõ kết quả provider · không tự sinh lại",
        (AdaptationStatus::Cancelled, true) => "Cancelled · source and saved revisions preserved",
        (AdaptationStatus::Cancelled, false) => "Đã hủy · nguồn và revision đã lưu được giữ",
        (AdaptationStatus::Accepted, true) => "Accepted as an immutable revision",
        (AdaptationStatus::Accepted, false) => "Đã chấp nhận thành revision bất biến",
    }
}

pub fn adaptation_finding(code: AdaptationFindingCode, english: bool) -> &'static str {
    match (code, english) {
        (AdaptationFindingCode::SourceWarning, true) => "Source extraction warning",
        (AdaptationFindingCode::SourceWarning, false) => "Cảnh báo từ phần phân tích nguồn",
        (AdaptationFindingCode::UncoveredSource, true) => "Source block has no coverage record",
        (AdaptationFindingCode::UncoveredSource, false) => "Block nguồn chưa có hồ sơ bao phủ",
        (AdaptationFindingCode::OmittedSource, true) => "Source material omitted",
        (AdaptationFindingCode::OmittedSource, false) => "Nội dung nguồn bị bỏ qua",
        (AdaptationFindingCode::TextChanged, true) => "Text changed during adaptation",
        (AdaptationFindingCode::TextChanged, false) => "Văn bản đổi khi chuyển thể",
        (AdaptationFindingCode::UnresolvedSpeaker, true) => "Speaker unresolved",
        (AdaptationFindingCode::UnresolvedSpeaker, false) => "Chưa xác định người nói",
        (AdaptationFindingCode::NewSpeaker, true) => "New speaker proposed",
        (AdaptationFindingCode::NewSpeaker, false) => "Đề xuất người nói mới",
        (AdaptationFindingCode::PossibleNarrationConfusion, true) => {
            "Possible confusion between narration and dialogue"
        }
        (AdaptationFindingCode::PossibleNarrationConfusion, false) => {
            "Có thể nhầm lời dẫn với lời thoại"
        }
        (AdaptationFindingCode::CueLikeMarkup, true) => "Possible sound cue inside spoken text",
        (AdaptationFindingCode::CueLikeMarkup, false) => "Có thể có chỉ dẫn âm thanh trong lời nói",
        (AdaptationFindingCode::UnsupportedPerformanceControl, true) => {
            "Performance control requires explicit fallback or correction"
        }
        (AdaptationFindingCode::UnsupportedPerformanceControl, false) => {
            "Điều khiển diễn xuất cần chọn cách thay thế hoặc sửa rõ ràng"
        }
        (AdaptationFindingCode::ProviderReviewNote, true) => "Provider review suggestion",
        (AdaptationFindingCode::ProviderReviewNote, false) => "Gợi ý cần xem từ provider",
        (AdaptationFindingCode::SourceCoverageRequiresReview, true) => {
            "Source coverage requires human comparison"
        }
        (AdaptationFindingCode::SourceCoverageRequiresReview, false) => {
            "Độ bao phủ nguồn cần được người biên tập đối chiếu"
        }
    }
}

pub fn adaptation_problem(code: &str, english: bool) -> &'static str {
    match (code, english) {
        ("provider_outcome_unknown", true) => "The provider outcome is unknown. The saved attempt will not be automatically generated again. Review the run and its operator recovery record.",
        ("provider_outcome_unknown", false) => "Chưa biết kết quả provider. Lần xử lý đã lưu sẽ không tự sinh lại. Xem run và hồ sơ khôi phục của operator.",
        ("provider_unavailable" | "provider_rejected", true) => "The local provider could not fulfill this request. Inspect the saved failure before starting a new generation.",
        ("provider_unavailable" | "provider_rejected", false) => "Provider local chưa xử lý được yêu cầu này. Xem lỗi đã lưu trước khi bắt đầu lần sinh mới.",
        ("provider_configuration_changed", true) => "The run's pinned provider configuration changed. An operator must review it; a different configuration cannot silently replace this attempt.",
        ("provider_configuration_changed", false) => "Cấu hình provider đã ghim của run đã đổi. Operator cần xem lại; cấu hình khác không tự thay thế lần xử lý này.",
        ("output_too_large" | "provider_output_too_large" | "output_findings_too_large", true) => "Provider output exceeded the bounded size. No revision was accepted; the source and prior revisions remain available.",
        ("output_too_large" | "provider_output_too_large" | "output_findings_too_large", false) => "Đầu ra provider vượt giới hạn dung lượng. Chưa chấp nhận revision; nguồn và các bản trước vẫn còn.",
        ("invalid_output" | "invalid_script_output" | "provider_malformed_response", true) => "Provider output failed the structured or semantic contract. No revision was accepted. Inspect the findings and saved run.",
        ("invalid_output" | "invalid_script_output" | "provider_malformed_response", false) => "Đầu ra provider chưa đạt hợp đồng cấu trúc hoặc ngữ nghĩa. Chưa chấp nhận revision. Xem các lỗi và run đã lưu.",
        (_, true) => "This run needs operator review. Its saved source and accepted revisions remain intact; use the displayed findings for recovery.",
        (_, false) => "Run này cần operator xem lại. Nguồn và revision đã chấp nhận vẫn được giữ; dùng các lỗi hiển thị để khôi phục.",
    }
}

pub fn adaptation_finding_explanation(code: AdaptationFindingCode, english: bool) -> &'static str {
    match (code, english) {
        (AdaptationFindingCode::SourceWarning, true) => "Review the original extraction warning before accepting this proposal.",
        (AdaptationFindingCode::SourceWarning, false) => "Xem cảnh báo phân tích nguồn ban đầu trước khi chấp nhận đề xuất này.",
        (AdaptationFindingCode::UncoveredSource, true) => "No proposed line or cue cites this source block. Check whether its content was lost.",
        (AdaptationFindingCode::UncoveredSource, false) => "Chưa có lời thoại hoặc âm thanh đề xuất dẫn tới block nguồn này. Kiểm tra xem nội dung có bị mất không.",
        (AdaptationFindingCode::OmittedSource, true) => "The provider omitted this source block. Review the stated reason against the source.",
        (AdaptationFindingCode::OmittedSource, false) => "Provider đã bỏ block nguồn này. Đối chiếu lý do đã nêu với bản gốc.",
        (AdaptationFindingCode::TextChanged, true) => "Adapted spoken text differs from its source block. Compare meaning, facts and omissions.",
        (AdaptationFindingCode::TextChanged, false) => "Lời nói được chuyển thể khác block nguồn. Đối chiếu ý nghĩa, sự kiện và phần bỏ qua.",
        (AdaptationFindingCode::UnresolvedSpeaker, true) => "The speaker is unknown. A placeholder does not resolve attribution; confirm or correct it.",
        (AdaptationFindingCode::UnresolvedSpeaker, false) => "Người nói chưa rõ. Tên tạm chưa xác định được người nói; cần xác nhận hoặc sửa.",
        (AdaptationFindingCode::NewSpeaker, true) => "This speaker name was not recorded in the source extraction. Confirm its attribution.",
        (AdaptationFindingCode::NewSpeaker, false) => "Tên người nói này chưa được ghi trong phần phân tích nguồn. Cần xác nhận cách gán lời.",
        (AdaptationFindingCode::PossibleNarrationConfusion, true) => "The source text type and proposed speaker role differ. Check narration and dialogue separately.",
        (AdaptationFindingCode::PossibleNarrationConfusion, false) => "Loại văn bản nguồn và vai người nói đề xuất khác nhau. Kiểm tra riêng lời dẫn và lời thoại.",
        (AdaptationFindingCode::CueLikeMarkup, true) => "Spoken text contains markup-like characters. Decide whether they should be spoken or moved to a separate cue.",
        (AdaptationFindingCode::CueLikeMarkup, false) => "Lời nói chứa ký tự giống chỉ dẫn. Xác định chúng cần được đọc hay chuyển thành âm thanh riêng.",
        (AdaptationFindingCode::UnsupportedPerformanceControl, true) => "Prosody or pacing is a review note, not an implemented playback control. Choose an explicit fallback or correction.",
        (AdaptationFindingCode::UnsupportedPerformanceControl, false) => "Ngữ điệu hoặc nhịp diễn là ghi chú cần xem, chưa phải điều khiển phát đã triển khai. Chọn cách thay thế hoặc sửa rõ ràng.",
        (AdaptationFindingCode::ProviderReviewNote, true) => "Treat this provider note as an untrusted suggestion and compare it with the source.",
        (AdaptationFindingCode::ProviderReviewNote, false) => "Xem ghi chú provider như gợi ý chưa được xác nhận và đối chiếu với nguồn.",
        (AdaptationFindingCode::SourceCoverageRequiresReview, true) => "Citations show the model's claimed coverage. A person must compare facts, chronology and attribution against the complete source.",
        (AdaptationFindingCode::SourceCoverageRequiresReview, false) => "Trích dẫn thể hiện độ bao phủ do model đề xuất. Người biên tập cần đối chiếu sự kiện, thứ tự và cách gán lời với toàn bộ nguồn.",
    }
}

pub fn adaptation_has_provider_detail(code: AdaptationFindingCode) -> bool {
    match code {
        AdaptationFindingCode::NewSpeaker
        | AdaptationFindingCode::OmittedSource
        | AdaptationFindingCode::UnsupportedPerformanceControl
        | AdaptationFindingCode::ProviderReviewNote => true,
        AdaptationFindingCode::SourceWarning
        | AdaptationFindingCode::UncoveredSource
        | AdaptationFindingCode::TextChanged
        | AdaptationFindingCode::UnresolvedSpeaker
        | AdaptationFindingCode::PossibleNarrationConfusion
        | AdaptationFindingCode::CueLikeMarkup
        | AdaptationFindingCode::SourceCoverageRequiresReview => false,
    }
}

pub fn adaptation_provider_detail(english: bool) -> &'static str {
    if english {
        "Provider-proposed content · requires review"
    } else {
        "Nội dung do provider đề xuất · cần xem lại"
    }
}

pub fn adaptation_local_model_label(english: bool) -> &'static str {
    if english {
        "Local model fingerprint"
    } else {
        "Dấu vân tay model local"
    }
}

pub fn adaptation_local_model_evidence(verified: bool, english: bool) -> &'static str {
    match (verified, english) {
        (true, true) => "The backend verified local model identity. This configuration evidence does not prove generation succeeded.",
        (true, false) => "Backend đã xác minh danh tính model local. Bằng chứng cấu hình này chưa chứng minh sinh nội dung thành công.",
        (false, true) => "No verified local model weights. A fixture is not evidence of a real model run.",
        (false, false) => "Chưa xác minh weights model local. Fixture chưa phải bằng chứng một lần chạy model thật.",
    }
}

pub fn adaptation_cost(cost: Option<&AdaptationCost>, english: bool) -> String {
    match cost {
        None => adaptation_copy(english).unknown.into(),
        Some(cost) => match cost.currency {
            AdaptationCurrency::USD => format!(
                "USD {}.{:02}",
                cost.amount_minor / 100,
                cost.amount_minor % 100
            ),
            AdaptationCurrency::VND => format!("VND {}", cost.amount_minor),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_run_and_review_finding_has_both_locales_and_unknown_cost_is_not_zero() {
        for english in [false, true] {
            for status in [
                AdaptationStatus::Queued,
                AdaptationStatus::Running,
                AdaptationStatus::Succeeded,
                AdaptationStatus::InvalidOutput,
                AdaptationStatus::Failed,
                AdaptationStatus::Ambiguous,
                AdaptationStatus::Cancelled,
                AdaptationStatus::Accepted,
            ] {
                assert!(!adaptation_run_status(status, english).is_empty());
            }
            for finding in [
                AdaptationFindingCode::SourceWarning,
                AdaptationFindingCode::UncoveredSource,
                AdaptationFindingCode::OmittedSource,
                AdaptationFindingCode::TextChanged,
                AdaptationFindingCode::UnresolvedSpeaker,
                AdaptationFindingCode::NewSpeaker,
                AdaptationFindingCode::PossibleNarrationConfusion,
                AdaptationFindingCode::CueLikeMarkup,
                AdaptationFindingCode::UnsupportedPerformanceControl,
                AdaptationFindingCode::ProviderReviewNote,
                AdaptationFindingCode::SourceCoverageRequiresReview,
            ] {
                assert!(!adaptation_finding(finding, english).is_empty());
                assert!(!adaptation_finding_explanation(finding, english).is_empty());
            }
            assert!(!adaptation_copy(english).unknown.contains('0'));
            assert!(!adaptation_copy(english).provider_absent.is_empty());
        }
    }

    #[test]
    fn deterministic_findings_use_localized_explanations_and_only_content_notes_are_quoted() {
        for code in [
            AdaptationFindingCode::UncoveredSource,
            AdaptationFindingCode::TextChanged,
            AdaptationFindingCode::UnresolvedSpeaker,
            AdaptationFindingCode::PossibleNarrationConfusion,
            AdaptationFindingCode::CueLikeMarkup,
            AdaptationFindingCode::SourceCoverageRequiresReview,
        ] {
            assert!(!adaptation_has_provider_detail(code));
            assert_ne!(
                adaptation_finding_explanation(code, false),
                adaptation_finding_explanation(code, true)
            );
        }
        for code in [
            AdaptationFindingCode::NewSpeaker,
            AdaptationFindingCode::OmittedSource,
            AdaptationFindingCode::UnsupportedPerformanceControl,
            AdaptationFindingCode::ProviderReviewNote,
        ] {
            assert!(adaptation_has_provider_detail(code));
        }
        assert!(!adaptation_has_provider_detail(
            AdaptationFindingCode::SourceWarning
        ));
    }

    #[test]
    fn cost_uses_recorded_currency_minor_units_and_missing_cost_stays_unknown() {
        use cantos_api::AdaptationCostBasis;
        let usd = AdaptationCost {
            currency: AdaptationCurrency::USD,
            amount_minor: 123,
            basis: AdaptationCostBasis::ProviderReported,
        };
        let vnd = AdaptationCost {
            currency: AdaptationCurrency::VND,
            amount_minor: 123,
            basis: AdaptationCostBasis::ProviderReported,
        };
        assert_eq!(adaptation_cost(Some(&usd), true), "USD 1.23");
        assert_eq!(adaptation_cost(Some(&vnd), false), "VND 123");
        for english in [false, true] {
            assert_eq!(
                adaptation_cost(None, english),
                adaptation_copy(english).unknown
            );
        }
    }

    #[test]
    fn local_model_identity_copy_distinguishes_configuration_from_generation_and_fixture() {
        assert!(adaptation_local_model_evidence(true, true)
            .contains("does not prove generation succeeded"));
        assert!(adaptation_local_model_evidence(true, false)
            .contains("chưa chứng minh sinh nội dung thành công"));
        assert!(adaptation_local_model_evidence(false, true)
            .contains("No verified local model weights"));
        assert!(
            adaptation_local_model_evidence(false, false).contains("Fixture chưa phải bằng chứng")
        );
        assert_ne!(
            adaptation_local_model_label(true),
            adaptation_local_model_label(false)
        );
    }
}
