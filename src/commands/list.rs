use crate::db;
use crate::ui::views::run_project_list_view;
use anyhow::Result;

pub fn run() -> Result<()> {
    let conn = db::connect()?;
    run_project_list_view(&conn)
}
