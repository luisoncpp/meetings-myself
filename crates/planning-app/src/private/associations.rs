use super::error::AppError;
use super::service::PlanningApp;
use planning_core::{Association, AssociationEnd, AssociationId, Lifecycle, Link};

pub struct LinkEnds {
    pub left: AssociationEnd,
    pub right: AssociationEnd,
}

impl PlanningApp {
    /// Idempotent: the canonical ordering in `Association::link` means an existing
    /// pair compares equal regardless of the direction the caller supplied.
    pub async fn link(&self, ends: LinkEnds) -> Result<Association, AppError> {
        let candidate = Association::link(Link {
            left: ends.left,
            right: ends.right,
            clock: self.clock.as_ref(),
        })?;

        let existing = self
            .all_associations()
            .await?
            .into_iter()
            .find(|found| found.left == candidate.left && found.right == candidate.right);

        if let Some(mut found) = existing {
            if found.lifecycle.is_active() {
                return Ok(found);
            }
            found.lifecycle = Lifecycle::Active;
            self.store(AssociationId::TABLE, found.id.as_str(), &found)
                .await?;
            return Ok(found);
        }

        self.store(AssociationId::TABLE, candidate.id.as_str(), &candidate)
            .await?;
        Ok(candidate)
    }

    /// Archives the link. ADR 0002 has no delete path, not even for links.
    pub async fn unlink(&self, link: &AssociationId) -> Result<(), AppError> {
        self.mutate::<Association>((AssociationId::TABLE, link.to_string()), |found| {
            found.lifecycle = Lifecycle::Archived;
        })
        .await?;
        Ok(())
    }

    pub async fn all_associations(&self) -> Result<Vec<Association>, AppError> {
        self.load_all(AssociationId::TABLE).await
    }

    /// Active links touching `end`. Archiving an entity does not archive its
    /// links, so a dormant link reappears the moment that entity is restored.
    pub async fn associations_for(
        &self,
        end: &AssociationEnd,
    ) -> Result<Vec<Association>, AppError> {
        Ok(self
            .all_associations()
            .await?
            .into_iter()
            .filter(|found| found.lifecycle.is_active() && found.touches(end))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::private::library::NewGoal;
    use crate::private::test_support::ready_app;
    use planning_core::AssociationEnd;

    #[tokio::test]
    async fn linking_the_same_pair_twice_returns_the_same_link() {
        let (_home, _drive, app) = ready_app().await;
        let goal = app
            .create_goal(NewGoal {
                title: "Career".into(),
                target_date: None,
            })
            .await
            .unwrap();
        let task = app.create_task("Prepare portfolio".into()).await.unwrap();

        let ends = || LinkEnds {
            left: AssociationEnd::Task(task.id.clone()),
            right: AssociationEnd::Goal(goal.id.clone()),
        };

        let first = app.link(ends()).await.unwrap();
        let second = app.link(ends()).await.unwrap();
        assert_eq!(first.id, second.id);
        assert_eq!(
            app.associations_for(&AssociationEnd::Goal(goal.id.clone()))
                .await
                .unwrap()
                .len(),
            1
        );
    }

    #[tokio::test]
    async fn archiving_one_side_never_cascades_and_the_link_returns_on_restore() {
        let (_home, _drive, app) = ready_app().await;
        let goal = app
            .create_goal(NewGoal {
                title: "Career".into(),
                target_date: None,
            })
            .await
            .unwrap();
        let task = app.create_task("Prepare portfolio".into()).await.unwrap();
        app.link(LinkEnds {
            left: AssociationEnd::Task(task.id.clone()),
            right: AssociationEnd::Goal(goal.id.clone()),
        })
        .await
        .unwrap();

        app.archive_task(&task.id).await.unwrap();

        assert_eq!(
            app.goal(&goal.id).await.unwrap().unwrap().lifecycle,
            Lifecycle::Active
        );
        assert_eq!(
            app.associations_for(&AssociationEnd::Goal(goal.id.clone()))
                .await
                .unwrap()
                .len(),
            1,
            "the link is preserved, not deleted"
        );

        app.restore_task(&task.id).await.unwrap();
        assert_eq!(
            app.associations_for(&AssociationEnd::Task(task.id.clone()))
                .await
                .unwrap()
                .len(),
            1
        );
    }

    #[tokio::test]
    async fn unlinking_archives_the_link_rather_than_deleting_it() {
        let (_home, _drive, app) = ready_app().await;
        let goal = app
            .create_goal(NewGoal {
                title: "Career".into(),
                target_date: None,
            })
            .await
            .unwrap();
        let task = app.create_task("Prepare portfolio".into()).await.unwrap();
        let link = app
            .link(LinkEnds {
                left: AssociationEnd::Task(task.id.clone()),
                right: AssociationEnd::Goal(goal.id.clone()),
            })
            .await
            .unwrap();

        app.unlink(&link.id).await.unwrap();
        assert!(app
            .associations_for(&AssociationEnd::Goal(goal.id))
            .await
            .unwrap()
            .is_empty());

        let all: Vec<Association> = app.all_associations().await.unwrap();
        assert_eq!(all.len(), 1, "the record still exists, archived");
    }
}
