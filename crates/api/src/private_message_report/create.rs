use crate::{check_report_reason, Perform};
use actix_web::web::Data;
use lemmy_api_common::{
  private_message::{CreatePrivateMessageReport, PrivateMessageReportResponse},
  utils::{blocking, get_local_user_view_from_jwt},
};
use lemmy_db_schema::{
  source::{
    private_message::PrivateMessage,
    private_message_report::{PrivateMessageReport, PrivateMessageReportForm},
  },
  traits::{Crud, Reportable},
};
use lemmy_db_views::structs::PrivateMessageReportView;
use lemmy_utils::{error::LemmyError, ConnectionId};
use lemmy_websocket::LemmyContext;

#[async_trait::async_trait(?Send)]
impl Perform for CreatePrivateMessageReport {
  type Response = PrivateMessageReportResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<Self::Response, LemmyError> {
    let local_user_view =
      get_local_user_view_from_jwt(&self.auth, context.pool(), context.secret()).await?;

    // check size of report and check for whitespace
    let reason = self.reason.trim();
    check_report_reason(reason)?;

    let person_id = local_user_view.person.id;
    let private_message_id = self.private_message_id;
    let private_message = blocking(context.pool(), move |conn| {
      PrivateMessage::read(conn, private_message_id)
    })
    .await??;

    let report_form = PrivateMessageReportForm {
      creator_id: person_id,
      private_message_id,
      original_pm_text: private_message.content,
      reason: reason.to_owned(),
    };

    let report = blocking(context.pool(), move |conn| {
      PrivateMessageReport::report(conn, &report_form)
    })
    .await?
    .map_err(|e| LemmyError::from_error_message(e, "couldnt_create_report"))?;

    let private_message_report_view = blocking(context.pool(), move |conn| {
      PrivateMessageReportView::read(conn, report.id)
    })
    .await??;

    let res = PrivateMessageReportResponse {
      private_message_report_view,
    };

    // TODO: should send message about new report to admins via websocket, but there is only op
    //       for community mods

    // TODO: consider federating this

    Ok(res)
  }
}
