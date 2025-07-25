use actix_web::web::{Data, Json};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::source::post::{PostActions, PostReadForm};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_post::{
  api::{MarkPostAsRead, PostResponse},
  PostView,
};
use lemmy_utils::error::LemmyResult;

pub async fn mark_post_as_read(
  data: Json<MarkPostAsRead>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PostResponse>> {
  let person_id = local_user_view.person.id;
  let local_instance_id = local_user_view.person.instance_id;
  let post_id = data.post_id;

  // Mark the post as read / unread
  let form = PostReadForm::new(post_id, person_id);
  if data.read {
    PostActions::mark_as_read(&mut context.pool(), &form).await?;
  } else {
    PostActions::mark_as_unread(&mut context.pool(), &form).await?;
  }
  let post_view = PostView::read(
    &mut context.pool(),
    post_id,
    Some(&local_user_view.local_user),
    local_instance_id,
    false,
  )
  .await?;

  Ok(Json(PostResponse { post_view }))
}
