use crate::commands::auth::DbState;
use rusqlite::params;
use serde::Serialize;
use tauri::State;

#[derive(Serialize)]
pub struct MonthlyData {
    pub month: String,
    pub income: f64,
    pub expenses: f64,
    pub balance: f64,
}

#[derive(Serialize)]
pub struct ExpensePieSlice {
    pub name: String,
    pub value: f64,
    pub color: String,
}

#[derive(Serialize)]
pub struct TransactionRow {
    pub id: String,
    pub date: String,
    pub description: String,
    pub vendor: String,
    pub amount: f64,
    pub r#type: String,
    pub category: String,
    pub status: String,
}

#[derive(Serialize)]
pub struct DashboardData {
    pub monthly: Vec<MonthlyData>,
    pub pie: Vec<ExpensePieSlice>,
    pub transactions: Vec<TransactionRow>,
    pub total_income_ytd: f64,
    pub total_expenses_ytd: f64,
    pub pending_receipts: i64,
    pub overdue_bills: i64,
}

const PIE_COLORS: &[&str] = &[
    "#3b82f6", "#10b981", "#f59e0b", "#ef4444",
    "#8b5cf6", "#06b6d4", "#6b7280",
];

#[tauri::command]
pub fn get_dashboard_data(db: State<DbState>) -> DashboardData {
    let conn = db.0.lock().unwrap();

    // Monthly income = invoices paid by month (current year)
    // Monthly expenses = bills paid by month (current year)
    let year = chrono::Utc::now().format("%Y").to_string();

    let mut monthly_map: std::collections::BTreeMap<String, (f64, f64)> = std::collections::BTreeMap::new();
    for m in 1..=12u32 {
        monthly_map.insert(format!("{}-{:02}", year, m), (0.0, 0.0));
    }

    // Income from paid invoices
    {
        let mut stmt = conn.prepare(
            "SELECT strftime('%Y-%m', paid_date) as mo, SUM(gross_total) FROM invoices
             WHERE status='oplacona' AND paid_date LIKE ?1 GROUP BY mo"
        ).unwrap();
        let rows = stmt.query_map(params![format!("{}%", year)], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, f64>(1)?))
        }).unwrap();
        for row in rows.filter_map(|r| r.ok()) {
            if let Some(entry) = monthly_map.get_mut(&row.0) { entry.0 += row.1; }
        }
    }

    // Expenses from paid bills
    {
        let mut stmt = conn.prepare(
            "SELECT strftime('%Y-%m', paid_date) as mo, SUM(amount) FROM bills
             WHERE status='oplacona' AND paid_date LIKE ?1 GROUP BY mo"
        ).unwrap();
        let rows = stmt.query_map(params![format!("{}%", year)], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, f64>(1)?))
        }).unwrap();
        for row in rows.filter_map(|r| r.ok()) {
            if let Some(entry) = monthly_map.get_mut(&row.0) { entry.1 += row.1; }
        }
    }

    let month_names = ["Sty","Lut","Mar","Kwi","Maj","Cze","Lip","Sie","Wrz","Paź","Lis","Gru"];
    let monthly: Vec<MonthlyData> = monthly_map.iter().enumerate().map(|(i, (_, (inc, exp)))| MonthlyData {
        month: month_names[i].to_string(),
        income: *inc,
        expenses: *exp,
        balance: inc - exp,
    }).collect();

    // Expense pie from bills by category
    let mut cat_map: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
    {
        let mut stmt = conn.prepare(
            "SELECT category, SUM(amount) FROM bills GROUP BY category"
        ).unwrap();
        let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, f64>(1)?))).unwrap();
        for row in rows.filter_map(|r| r.ok()) { *cat_map.entry(row.0).or_insert(0.0) += row.1; }
    }
    // Also add receipt expenses
    {
        let mut stmt = conn.prepare(
            "SELECT category, SUM(amount_gross) FROM receipts WHERE status='zatwierdzony' GROUP BY category"
        ).unwrap();
        let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, f64>(1)?))).unwrap();
        for row in rows.filter_map(|r| r.ok()) { *cat_map.entry(row.0).or_insert(0.0) += row.1; }
    }
    let pie: Vec<ExpensePieSlice> = cat_map.iter().enumerate()
        .map(|(i, (name, val))| ExpensePieSlice {
            name: name.clone(),
            value: *val,
            color: PIE_COLORS[i % PIE_COLORS.len()].to_string(),
        }).collect();

    // Recent transactions (last 10 receipts + bills combined, sorted by date)
    let mut transactions: Vec<TransactionRow> = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT id, date, description, vendor, amount_gross, category FROM receipts
             WHERE status='zatwierdzony' ORDER BY date DESC LIMIT 10"
        ).unwrap();
        let rows = stmt.query_map([], |r| Ok(TransactionRow {
            id: r.get(0)?, date: r.get(1)?, description: r.get(2)?,
            vendor: r.get(3)?, amount: r.get(4)?, r#type: "expense".into(),
            category: r.get(5)?, status: "Zaksięgowany".into(),
        })).unwrap();
        transactions.extend(rows.filter_map(|r| r.ok()));
    }
    {
        let mut stmt = conn.prepare(
            "SELECT id, paid_date, description, vendor, amount, category FROM bills
             WHERE status='oplacona' AND paid_date IS NOT NULL ORDER BY paid_date DESC LIMIT 5"
        ).unwrap();
        let rows = stmt.query_map([], |r| Ok(TransactionRow {
            id: r.get(0)?, date: r.get(1).unwrap_or_default(), description: r.get(2)?,
            vendor: r.get(3)?, amount: r.get(4)?, r#type: "expense".into(),
            category: r.get(5)?, status: "Zaksięgowany".into(),
        })).unwrap();
        transactions.extend(rows.filter_map(|r| r.ok()));
    }
    transactions.sort_by(|a, b| b.date.cmp(&a.date));
    transactions.truncate(10);

    // Summary stats
    let total_income_ytd: f64 = conn.query_row(
        "SELECT COALESCE(SUM(gross_total),0) FROM invoices WHERE status='oplacona' AND paid_date LIKE ?1",
        params![format!("{}%", year)], |r| r.get(0)
    ).unwrap_or(0.0);
    let total_expenses_ytd: f64 = conn.query_row(
        "SELECT COALESCE(SUM(amount),0) FROM bills WHERE status='oplacona' AND paid_date LIKE ?1",
        params![format!("{}%", year)], |r| r.get(0)
    ).unwrap_or(0.0);
    let pending_receipts: i64 = conn.query_row(
        "SELECT COUNT(*) FROM receipts WHERE status='oczekuje' OR status='zaklasyfikowany'",
        [], |r| r.get(0)
    ).unwrap_or(0);
    let overdue_bills: i64 = conn.query_row(
        "SELECT COUNT(*) FROM bills WHERE status='przeterminowana'",
        [], |r| r.get(0)
    ).unwrap_or(0);

    DashboardData { monthly, pie, transactions, total_income_ytd, total_expenses_ytd, pending_receipts, overdue_bills }
}
