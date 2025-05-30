use crate::PostReportView;
use diesel::{
  BoolExpressionMethods,
  ExpressionMethods,
  JoinOnDsl,
  NullableExpressionMethods,
  QueryDsl,
  SelectableHelper,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  aliases::{self, creator_community_actions},
  newtypes::{PersonId, PostReportId},
  utils::{get_conn, DbPool},
};
use lemmy_db_schema_file::schema::{
  community,
  community_actions,
  local_user,
  person,
  person_actions,
  post,
  post_actions,
  post_report,
};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

impl PostReportView {
  #[diesel::dsl::auto_type(no_type_alias)]
  fn joins(my_person_id: PersonId) -> _ {
    let recipient_id = aliases::person1.field(person::id);
    let resolver_id = aliases::person2.field(person::id);

    let community_join = community::table.on(post::community_id.eq(community::id));

    let report_creator_join = person::table.on(post_report::creator_id.eq(person::id));

    let post_creator_join = aliases::person1.on(post::creator_id.eq(recipient_id));

    let creator_community_actions_join = creator_community_actions.on(
      creator_community_actions
        .field(community_actions::community_id)
        .eq(post::community_id)
        .and(
          creator_community_actions
            .field(community_actions::person_id)
            .eq(post::creator_id),
        ),
    );

    let community_actions_join = community_actions::table.on(
      community_actions::community_id
        .eq(post::community_id)
        .and(community_actions::person_id.eq(my_person_id)),
    );

    let local_user_join = local_user::table.on(
      post::creator_id
        .eq(local_user::person_id)
        .and(local_user::admin.eq(true)),
    );

    let post_actions_join = post_actions::table.on(
      post_actions::post_id
        .eq(post::id)
        .and(post_actions::person_id.eq(my_person_id)),
    );

    let person_actions_join = person_actions::table.on(
      person_actions::target_id
        .eq(post::creator_id)
        .and(person_actions::person_id.eq(my_person_id)),
    );

    let resolver_join = aliases::person2.on(post_report::resolver_id.eq(resolver_id.nullable()));

    post_report::table
      .inner_join(post::table)
      .inner_join(community_join)
      .inner_join(report_creator_join)
      .inner_join(post_creator_join)
      .left_join(creator_community_actions_join)
      .left_join(community_actions_join)
      .left_join(local_user_join)
      .left_join(post_actions_join)
      .left_join(person_actions_join)
      .left_join(resolver_join)
  }

  /// returns the PostReportView for the provided report_id
  ///
  /// * `report_id` - the report id to obtain
  pub async fn read(
    pool: &mut DbPool<'_>,
    report_id: PostReportId,
    my_person_id: PersonId,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;

    Self::joins(my_person_id)
      .filter(post_report::id.eq(report_id))
      .select(Self::as_select())
      .first(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }
}
