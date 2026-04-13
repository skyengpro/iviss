use crate::dto::audit::AuditLogEntry;
use crate::errors::AppError;
use genpdf::elements::{Paragraph, TableLayout};
use genpdf::style::{Color, Style};
use genpdf::{Alignment, Document, Element};
use std::io::Cursor;

pub async fn generate_audit_logs_pdf(items: Vec<AuditLogEntry>) -> Result<Vec<u8>, AppError> {
    // Try system DejaVu fonts first (installed via fonts-dejavu-core),
    // then fall back to the assets/fonts directory (dev volume mount).
    let font_family = genpdf::fonts::from_files(
        "/usr/share/fonts/truetype/dejavu",
        "DejaVuSans",
        None,
    )
    .or_else(|_| genpdf::fonts::from_files("./assets/fonts", "DejaVuSans", None))
    .map_err(|e| AppError::internal_error(format!("Failed to load fonts: {e}")))?;

    let mut doc = Document::new(font_family);
    doc.set_title("Audit Logs Report");

    let mut decorator = genpdf::SimplePageDecorator::new();
    decorator.set_margins(10);
    doc.set_page_decorator(decorator);

    // Title
    doc.push(
        Paragraph::new("Audit Logs Report")
            .aligned(Alignment::Center)
            .styled(Style::new().bold().with_font_size(18)),
    );
    doc.push(genpdf::elements::Break::new(1.0));

    // Table
    let mut table = TableLayout::new(vec![2, 4, 3, 3, 3]);
    table.set_cell_decorator(genpdf::elements::FrameCellDecorator::new(true, true, true));

    // Header
    let _ = table.push_row(vec![
        Box::new(Paragraph::new("ID").styled(Style::new().bold())),
        Box::new(Paragraph::new("Timestamp").styled(Style::new().bold())),
        Box::new(Paragraph::new("User").styled(Style::new().bold())),
        Box::new(Paragraph::new("Action").styled(Style::new().bold())),
        Box::new(Paragraph::new("IP Address").styled(Style::new().bold())),
    ]);

    // Rows
    for entry in items {
        let _ = table.push_row(vec![
            Box::new(Paragraph::new(entry.id.to_string().chars().take(8).collect::<String>())),
            Box::new(Paragraph::new(entry.created_at)),
            Box::new(Paragraph::new(entry.user_name.as_deref().unwrap_or(""))),
            Box::new(Paragraph::new(entry.action.as_str())),
            Box::new(Paragraph::new(entry.ip_address.as_deref().unwrap_or(""))),
        ]);
    }

    doc.push(table);

    let mut buffer = Vec::new();
    doc.render(&mut buffer)
        .map_err(|e| AppError::internal_error(format!("Failed to render PDF: {e}")))?;

    Ok(buffer)
}
