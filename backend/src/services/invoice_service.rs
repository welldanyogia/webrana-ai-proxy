//! Invoice Service for generating and managing invoices
//!
//! Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6
//! Generates HTML invoices that can be printed to PDF by the browser

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Invoice entity
#[derive(Debug, Serialize, Clone)]
pub struct Invoice {
    pub id: Uuid,
    pub user_id: Uuid,
    pub subscription_id: Option<Uuid>,
    pub invoice_number: String,
    pub subtotal_idr: i64,
    pub ppn_idr: i64,
    pub total_idr: i64,
    pub payment_method: Option<String>,
    pub midtrans_transaction_id: Option<String>,
    pub status: String,
    pub paid_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Invoice line item
#[derive(Debug, Serialize, Clone)]
pub struct InvoiceLineItem {
    pub description: String,
    pub quantity: i32,
    pub unit_price: i64,
    pub total: i64,
}

/// Invoice with user details for rendering
#[derive(Debug, Serialize)]
pub struct InvoiceWithDetails {
    pub invoice: Invoice,
    pub user_email: String,
    pub user_name: Option<String>,
    pub plan_tier: String,
    pub line_items: Vec<InvoiceLineItem>,
}

/// Invoice service error
#[derive(Debug, thiserror::Error)]
pub enum InvoiceError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Invoice not found")]
    NotFound,
}

/// Invoice Service
pub struct InvoiceService {
    pool: PgPool,
}


impl InvoiceService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get invoice by ID with user details
    pub async fn get_invoice(&self, invoice_id: Uuid) -> Result<InvoiceWithDetails, InvoiceError> {
        let row = sqlx::query(
            r#"
            SELECT 
                i.id, i.user_id, i.subscription_id, i.invoice_number,
                i.subtotal_idr, i.ppn_idr, i.total_idr, i.payment_method,
                i.midtrans_transaction_id, i.status, i.paid_at, i.created_at,
                u.email as user_email, u.name as user_name,
                COALESCE(s.plan_tier::text, 'free') as plan_tier
            FROM invoices i
            JOIN users u ON u.id = i.user_id
            LEFT JOIN subscriptions s ON s.id = i.subscription_id
            WHERE i.id = $1
            "#,
        )
        .bind(invoice_id)
        .fetch_optional(&self.pool)
        .await?;

        let row = row.ok_or(InvoiceError::NotFound)?;

        let invoice = Invoice {
            id: row.get("id"),
            user_id: row.get("user_id"),
            subscription_id: row.get("subscription_id"),
            invoice_number: row.get("invoice_number"),
            subtotal_idr: row.get("subtotal_idr"),
            ppn_idr: row.get("ppn_idr"),
            total_idr: row.get("total_idr"),
            payment_method: row.get("payment_method"),
            midtrans_transaction_id: row.get("midtrans_transaction_id"),
            status: row.get("status"),
            paid_at: row.get("paid_at"),
            created_at: row.get("created_at"),
        };

        let plan_tier: String = row.get("plan_tier");
        let line_items = vec![
            InvoiceLineItem {
                description: format!("Webrana {} Plan - 1 Month", plan_tier.to_uppercase()),
                quantity: 1,
                unit_price: invoice.subtotal_idr,
                total: invoice.subtotal_idr,
            },
            InvoiceLineItem {
                description: "PPN (11%)".to_string(),
                quantity: 1,
                unit_price: invoice.ppn_idr,
                total: invoice.ppn_idr,
            },
        ];

        Ok(InvoiceWithDetails {
            invoice,
            user_email: row.get("user_email"),
            user_name: row.get("user_name"),
            plan_tier,
            line_items,
        })
    }

    /// Get invoices for a user
    pub async fn get_user_invoices(&self, user_id: Uuid) -> Result<Vec<Invoice>, InvoiceError> {
        let rows = sqlx::query(
            r#"
            SELECT id, user_id, subscription_id, invoice_number,
                   subtotal_idr, ppn_idr, total_idr, payment_method,
                   midtrans_transaction_id, status, paid_at, created_at
            FROM invoices
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| Invoice {
                id: r.get("id"),
                user_id: r.get("user_id"),
                subscription_id: r.get("subscription_id"),
                invoice_number: r.get("invoice_number"),
                subtotal_idr: r.get("subtotal_idr"),
                ppn_idr: r.get("ppn_idr"),
                total_idr: r.get("total_idr"),
                payment_method: r.get("payment_method"),
                midtrans_transaction_id: r.get("midtrans_transaction_id"),
                status: r.get("status"),
                paid_at: r.get("paid_at"),
                created_at: r.get("created_at"),
            })
            .collect())
    }

    /// Generate HTML invoice for printing/PDF
    /// Requirements: 4.1, 4.2, 4.3, 4.4
    pub fn generate_html_invoice(invoice: &InvoiceWithDetails) -> String {
        let paid_date = invoice
            .invoice
            .paid_at
            .map(|d| d.format("%d %B %Y").to_string())
            .unwrap_or_else(|| "-".to_string());

        let customer_name = invoice
            .user_name
            .clone()
            .unwrap_or_else(|| invoice.user_email.clone());

        format!(
            r#"<!DOCTYPE html>
<html lang="id">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Invoice {invoice_number}</title>
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{ font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; padding: 40px; max-width: 800px; margin: 0 auto; color: #333; }}
        .header {{ display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 40px; border-bottom: 2px solid #3B82F6; padding-bottom: 20px; }}
        .logo {{ font-size: 28px; font-weight: bold; color: #3B82F6; }}
        .invoice-info {{ text-align: right; }}
        .invoice-number {{ font-size: 24px; font-weight: bold; color: #1F2937; }}
        .invoice-date {{ color: #6B7280; margin-top: 5px; }}
        .parties {{ display: flex; justify-content: space-between; margin-bottom: 40px; }}
        .party {{ width: 45%; }}
        .party-title {{ font-weight: bold; color: #6B7280; margin-bottom: 10px; text-transform: uppercase; font-size: 12px; }}
        .party-name {{ font-size: 18px; font-weight: bold; margin-bottom: 5px; }}
        .party-detail {{ color: #6B7280; font-size: 14px; line-height: 1.6; }}
        table {{ width: 100%; border-collapse: collapse; margin-bottom: 30px; }}
        th {{ background: #F3F4F6; padding: 12px; text-align: left; font-weight: 600; color: #374151; border-bottom: 2px solid #E5E7EB; }}
        td {{ padding: 12px; border-bottom: 1px solid #E5E7EB; }}
        .text-right {{ text-align: right; }}
        .totals {{ margin-left: auto; width: 300px; }}
        .totals-row {{ display: flex; justify-content: space-between; padding: 8px 0; }}
        .totals-row.total {{ font-size: 20px; font-weight: bold; border-top: 2px solid #1F2937; padding-top: 12px; margin-top: 8px; }}
        .status {{ display: inline-block; padding: 4px 12px; border-radius: 20px; font-size: 12px; font-weight: bold; }}
        .status-paid {{ background: #D1FAE5; color: #065F46; }}
        .status-pending {{ background: #FEF3C7; color: #92400E; }}
        .footer {{ margin-top: 60px; padding-top: 20px; border-top: 1px solid #E5E7EB; color: #6B7280; font-size: 12px; text-align: center; }}
        .payment-info {{ background: #F9FAFB; padding: 20px; border-radius: 8px; margin-bottom: 30px; }}
        .payment-info-title {{ font-weight: bold; margin-bottom: 10px; }}
        @media print {{ body {{ padding: 20px; }} .no-print {{ display: none; }} }}
    </style>
</head>
<body>
    <div class="header">
        <div class="logo">🌐 Webrana</div>
        <div class="invoice-info">
            <div class="invoice-number">{invoice_number}</div>
            <div class="invoice-date">Tanggal: {paid_date}</div>
            <div style="margin-top: 10px;">
                <span class="status {status_class}">{status}</span>
            </div>
        </div>
    </div>

    <div class="parties">
        <div class="party">
            <div class="party-title">Dari</div>
            <div class="party-name">PT Webrana Indonesia</div>
            <div class="party-detail">
                Jl. Teknologi No. 123<br>
                Jakarta Selatan, 12345<br>
                Indonesia<br>
                NPWP: 00.000.000.0-000.000
            </div>
        </div>
        <div class="party">
            <div class="party-title">Kepada</div>
            <div class="party-name">{customer_name}</div>
            <div class="party-detail">
                {customer_email}
            </div>
        </div>
    </div>

    <table>
        <thead>
            <tr>
                <th>Deskripsi</th>
                <th class="text-right">Qty</th>
                <th class="text-right">Harga</th>
                <th class="text-right">Total</th>
            </tr>
        </thead>
        <tbody>
            <tr>
                <td>Webrana {plan_tier} Plan - 1 Bulan</td>
                <td class="text-right">1</td>
                <td class="text-right">{subtotal_formatted}</td>
                <td class="text-right">{subtotal_formatted}</td>
            </tr>
        </tbody>
    </table>

    <div class="totals">
        <div class="totals-row">
            <span>Subtotal</span>
            <span>{subtotal_formatted}</span>
        </div>
        <div class="totals-row">
            <span>PPN (11%)</span>
            <span>{ppn_formatted}</span>
        </div>
        <div class="totals-row total">
            <span>Total</span>
            <span>{total_formatted}</span>
        </div>
    </div>

    <div class="payment-info">
        <div class="payment-info-title">Informasi Pembayaran</div>
        <div>Metode: {payment_method}</div>
        <div>Transaction ID: {transaction_id}</div>
    </div>

    <div class="footer">
        <p>Terima kasih telah menggunakan Webrana!</p>
        <p style="margin-top: 5px;">Invoice ini dibuat secara otomatis dan sah tanpa tanda tangan.</p>
        <p style="margin-top: 10px;">support@webrana.id | webrana.id</p>
    </div>
</body>
</html>"#,
            invoice_number = invoice.invoice.invoice_number,
            paid_date = paid_date,
            status = if invoice.invoice.status == "paid" { "LUNAS" } else { "PENDING" },
            status_class = if invoice.invoice.status == "paid" { "status-paid" } else { "status-pending" },
            customer_name = customer_name,
            customer_email = invoice.user_email,
            plan_tier = invoice.plan_tier.to_uppercase(),
            subtotal_formatted = format_rupiah(invoice.invoice.subtotal_idr),
            ppn_formatted = format_rupiah(invoice.invoice.ppn_idr),
            total_formatted = format_rupiah(invoice.invoice.total_idr),
            payment_method = invoice.invoice.payment_method.clone().unwrap_or_else(|| "-".to_string()),
            transaction_id = invoice.invoice.midtrans_transaction_id.clone().unwrap_or_else(|| "-".to_string()),
        )
    }
}

/// Format number as Indonesian Rupiah
fn format_rupiah(amount: i64) -> String {
    let formatted = amount
        .to_string()
        .chars()
        .rev()
        .collect::<Vec<_>>()
        .chunks(3)
        .map(|c| c.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join(".")
        .chars()
        .rev()
        .collect::<String>();
    format!("Rp {}", formatted)
}
