use crate::{
  activities::{generate_activity_id, send_lemmy_activity, verify_person},
  insert_received_activity,
  protocol::activities::following::{follow::Follow, undo_follow::UndoFollow},
};
use activitypub_federation::{
  config::Data,
  kinds::activity::UndoType,
  protocol::verification::verify_urls_match,
  traits::{ActivityHandler, Actor},
};
use lemmy_api_utils::context::LemmyContext;
use lemmy_apub_objects::objects::{community::ApubCommunity, person::ApubPerson, UserOrCommunity};
use lemmy_db_schema::{
  source::{activity::ActivitySendTargets, community::CommunityActions, person::PersonActions},
  traits::Followable,
};
use lemmy_utils::error::{LemmyError, LemmyResult};
use url::Url;

impl UndoFollow {
  pub async fn send(
    actor: &ApubPerson,
    community: &ApubCommunity,
    context: &Data<LemmyContext>,
  ) -> LemmyResult<()> {
    let object = Follow::new(actor, community, context)?;
    let undo = UndoFollow {
      actor: actor.id().into(),
      to: Some([community.id().into()]),
      object,
      kind: UndoType::Undo,
      id: generate_activity_id(
        UndoType::Undo,
        &context.settings().get_protocol_and_hostname(),
      )?,
    };
    let inbox = if community.local {
      ActivitySendTargets::empty()
    } else {
      ActivitySendTargets::to_inbox(community.shared_inbox_or_inbox())
    };
    send_lemmy_activity(context, undo, actor, inbox, true).await
  }
}

#[async_trait::async_trait]
impl ActivityHandler for UndoFollow {
  type DataType = LemmyContext;
  type Error = LemmyError;

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    self.actor.inner()
  }

  async fn verify(&self, context: &Data<LemmyContext>) -> LemmyResult<()> {
    verify_urls_match(self.actor.inner(), self.object.actor.inner())?;
    verify_person(&self.actor, context).await?;
    self.object.verify(context).await?;
    if let Some(to) = &self.to {
      verify_urls_match(to[0].inner(), self.object.object.inner())?;
    }
    Ok(())
  }

  async fn receive(self, context: &Data<LemmyContext>) -> LemmyResult<()> {
    insert_received_activity(&self.id, context).await?;
    let person = self.actor.dereference(context).await?;
    let object = self.object.object.dereference(context).await?;

    match object {
      UserOrCommunity::Left(u) => {
        PersonActions::unfollow(&mut context.pool(), person.id, u.id).await?;
      }
      UserOrCommunity::Right(c) => {
        CommunityActions::unfollow(&mut context.pool(), person.id, c.id).await?;
      }
    }

    Ok(())
  }
}
