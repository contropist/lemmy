use crate::protocol::collections::group_followers::GroupFollowers;
use activitypub_federation::{
  config::Data,
  kinds::collection::CollectionType,
  protocol::verification::verify_domains_match,
  traits::Collection,
};
use lemmy_api_utils::{context::LemmyContext, utils::generate_followers_url};
use lemmy_apub_objects::objects::community::ApubCommunity;
use lemmy_db_schema::source::community::Community;
use lemmy_db_views_community_follower::CommunityFollowerView;
use lemmy_utils::error::LemmyError;
use url::Url;

#[derive(Clone, Debug)]
pub(crate) struct ApubCommunityFollower(());

#[async_trait::async_trait]
impl Collection for ApubCommunityFollower {
  type Owner = ApubCommunity;
  type DataType = LemmyContext;
  type Kind = GroupFollowers;
  type Error = LemmyError;

  async fn read_local(
    community: &Self::Owner,
    context: &Data<Self::DataType>,
  ) -> Result<Self::Kind, Self::Error> {
    let community_id = community.id;
    let community_followers =
      CommunityFollowerView::count_community_followers(&mut context.pool(), community_id).await?;

    Ok(GroupFollowers {
      id: generate_followers_url(&community.ap_id)?.into(),
      r#type: CollectionType::Collection,
      total_items: community_followers,
      items: vec![],
    })
  }

  async fn verify(
    json: &Self::Kind,
    expected_domain: &Url,
    _data: &Data<Self::DataType>,
  ) -> Result<(), Self::Error> {
    verify_domains_match(expected_domain, &json.id)?;
    Ok(())
  }

  async fn from_json(
    json: Self::Kind,
    community: &Self::Owner,
    context: &Data<Self::DataType>,
  ) -> Result<Self, Self::Error> {
    Community::update_federated_followers(&mut context.pool(), community.id, json.total_items)
      .await?;

    Ok(ApubCommunityFollower(()))
  }
}
