use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  reports::combined::{ListReports, ListReportsResponse},
  utils::check_community_mod_of_any_or_admin_action,
};
use lemmy_db_schema::traits::PaginationCursorBuilder;
use lemmy_db_views::{
  combined::report_combined_view::ReportCombinedQuery,
  structs::{LocalUserView, ReportCombinedView},
};
use lemmy_utils::error::LemmyResult;

/// Lists reports for a community if an id is supplied
/// or returns all reports for communities a user moderates
pub async fn list_reports(
  data: Query<ListReports>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<ListReportsResponse>> {
  let my_reports_only = data.my_reports_only;

  // Only check mod or admin status when not viewing my reports
  if !my_reports_only.unwrap_or_default() {
    check_community_mod_of_any_or_admin_action(&local_user_view, &mut context.pool()).await?;
  }

  let cursor_data = if let Some(cursor) = &data.page_cursor {
    Some(ReportCombinedView::from_cursor(cursor, &mut context.pool()).await?)
  } else {
    None
  };

  let reports = ReportCombinedQuery {
    community_id: data.community_id,
    post_id: data.post_id,
    type_: data.type_,
    unresolved_only: data.unresolved_only,
    cursor_data,
    page_back: data.page_back,
    show_community_rule_violations: data.show_community_rule_violations,
    my_reports_only,
  }
  .list(&mut context.pool(), &local_user_view)
  .await?;

  let next_page = reports.last().map(PaginationCursorBuilder::to_cursor);

  Ok(Json(ListReportsResponse { reports, next_page }))
}
