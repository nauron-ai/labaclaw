#[derive(Debug, Clone, Copy)]
pub(super) enum WaAttachmentKind {
    Image,
    Document,
    Video,
    Audio,
}

impl WaAttachmentKind {
    pub(super) fn from_marker(value: &str) -> Option<Self> {
        match value.to_ascii_uppercase().as_str() {
            "IMAGE" => Some(Self::Image),
            "DOCUMENT" => Some(Self::Document),
            "VIDEO" => Some(Self::Video),
            "AUDIO" => Some(Self::Audio),
            _ => None,
        }
    }

    pub(super) fn media_type(self) -> wa_rs_core::download::MediaType {
        match self {
            Self::Image => wa_rs_core::download::MediaType::Image,
            Self::Document => wa_rs_core::download::MediaType::Document,
            Self::Video => wa_rs_core::download::MediaType::Video,
            Self::Audio => wa_rs_core::download::MediaType::Audio,
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct WaAttachment {
    pub(super) kind: WaAttachmentKind,
    pub(super) target: String,
}

pub(super) fn parse_wa_attachment_markers(message: &str) -> (String, Vec<WaAttachment>) {
    let mut cleaned = String::with_capacity(message.len());
    let mut attachments = Vec::new();
    let mut cursor = 0;

    while cursor < message.len() {
        let Some(open_rel) = message[cursor..].find('[') else {
            cleaned.push_str(&message[cursor..]);
            break;
        };

        let open = cursor + open_rel;
        cleaned.push_str(&message[cursor..open]);

        let Some(close_rel) = message[open..].find(']') else {
            cleaned.push_str(&message[open..]);
            break;
        };

        let close = open + close_rel;
        let marker = &message[open + 1..close];
        let parsed = marker.split_once(':').and_then(|(kind, target)| {
            let kind = WaAttachmentKind::from_marker(kind)?;
            let target = target.trim();
            if target.is_empty() {
                return None;
            }
            Some(WaAttachment {
                kind,
                target: target.to_string(),
            })
        });

        if let Some(attachment) = parsed {
            attachments.push(attachment);
        } else {
            cleaned.push_str(&message[open..=close]);
        }

        cursor = close + 1;
    }

    (cleaned.trim().to_string(), attachments)
}

pub(super) fn mime_from_path(path: &std::path::Path) -> &'static str {
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "mp4" => "video/mp4",
        "mov" => "video/quicktime",
        "mp3" => "audio/mpeg",
        "ogg" | "opus" => "audio/ogg",
        "pdf" => "application/pdf",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xls" => "application/vnd.ms-excel",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        _ => "application/octet-stream",
    }
}
