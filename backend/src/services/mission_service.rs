use sqlx::PgPool;
use uuid::Uuid;

use crate::db::models::Mission;
use crate::error::AppError;

/// Validate that a mission status transition is allowed.
fn validate_transition(current: &str, target: &str) -> Result<(), AppError> {
    let valid = match (current, target) {
        ("created", "assigned") => true,
        ("created", "cancelled") => true,
        ("assigned", "executing") => true,
        ("assigned", "cancelled") => true,
        ("executing", "completed") => true,
        ("executing", "failed") => true,
        ("executing", "cancelled") => true,
        _ => false,
    };
    if !valid {
        return Err(AppError::Validation(format!(
            "Invalid transition: {} → {}",
            current, target
        )));
    }
    Ok(())
}

/// Fetch a mission by ID or return NotFound.
async fn fetch_mission(pool: &PgPool, id: Uuid) -> Result<Mission, AppError> {
    let mission: Mission = sqlx::query_as(
        "SELECT id, robot_id, status, priority, start_node_id, end_node_id, path, metadata, created_at, updated_at FROM missions WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Mission {} not found", id)))?;
    Ok(mission)
}

/// Transition a mission to a new status, optionally setting robot_id and metadata fields.
async fn transition(
    pool: &PgPool,
    id: Uuid,
    target_status: &str,
    robot_id: Option<Uuid>,
    fail_reason: Option<&str>,
) -> Result<Mission, AppError> {
    let existing = fetch_mission(pool, id).await?;
    validate_transition(&existing.status, target_status)?;

    let new_robot_id = robot_id.or(existing.robot_id);
    let mut new_metadata = existing.metadata.clone();
    if let Some(reason) = fail_reason {
        if let Some(obj) = new_metadata.as_object_mut() {
            obj.insert(
                "fail_reason".to_string(),
                serde_json::Value::String(reason.to_string()),
            );
        }
    }

    let mission: Mission = sqlx::query_as(
        r#"UPDATE missions SET status = $1, robot_id = $2, metadata = $3, updated_at = NOW()
           WHERE id = $4
           RETURNING id, robot_id, status, priority, start_node_id, end_node_id, path, metadata, created_at, updated_at"#,
    )
    .bind(target_status)
    .bind(new_robot_id)
    .bind(&new_metadata)
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(mission)
}

pub async fn list(
    pool: &PgPool,
    status_filter: Option<&str>,
    page: i64,
    limit: i64,
) -> Result<Vec<Mission>, AppError> {
    let offset = (page - 1) * limit;

    let missions: Vec<Mission> = if let Some(status) = status_filter {
        sqlx::query_as(
            r#"SELECT id, robot_id, status, priority, start_node_id, end_node_id, path, metadata, created_at, updated_at
               FROM missions WHERE status = $1 ORDER BY priority DESC, created_at ASC LIMIT $2 OFFSET $3"#,
        )
        .bind(status)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as(
            r#"SELECT id, robot_id, status, priority, start_node_id, end_node_id, path, metadata, created_at, updated_at
               FROM missions ORDER BY priority DESC, created_at ASC LIMIT $1 OFFSET $2"#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?
    };

    Ok(missions)
}

pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<Mission, AppError> {
    fetch_mission(pool, id).await
}

pub async fn create(
    pool: &PgPool,
    start_node_id: Option<String>,
    end_node_id: Option<String>,
    priority: i32,
    metadata: serde_json::Value,
) -> Result<Mission, AppError> {
    let mission: Mission = sqlx::query_as(
        r#"INSERT INTO missions (start_node_id, end_node_id, priority, metadata)
           VALUES ($1, $2, $3, $4)
           RETURNING id, robot_id, status, priority, start_node_id, end_node_id, path, metadata, created_at, updated_at"#,
    )
    .bind(&start_node_id)
    .bind(&end_node_id)
    .bind(priority)
    .bind(&metadata)
    .fetch_one(pool)
    .await?;

    Ok(mission)
}

pub async fn assign(pool: &PgPool, mission_id: Uuid, robot_id: Uuid) -> Result<Mission, AppError> {
    transition(pool, mission_id, "assigned", Some(robot_id), None).await
}

pub async fn start_executing(pool: &PgPool, mission_id: Uuid) -> Result<Mission, AppError> {
    transition(pool, mission_id, "executing", None, None).await
}

pub async fn complete(pool: &PgPool, mission_id: Uuid) -> Result<Mission, AppError> {
    transition(pool, mission_id, "completed", None, None).await
}

pub async fn fail(pool: &PgPool, mission_id: Uuid, reason: &str) -> Result<Mission, AppError> {
    transition(pool, mission_id, "failed", None, Some(reason)).await
}

pub async fn cancel(pool: &PgPool, mission_id: Uuid) -> Result<Mission, AppError> {
    transition(pool, mission_id, "cancelled", None, None).await
}

pub async fn delete(pool: &PgPool, mission_id: Uuid) -> Result<(), AppError> {
    let existing = fetch_mission(pool, mission_id).await?;
    if existing.status != "created" {
        return Err(AppError::Validation(format!(
            "Can only delete missions in 'created' status, current status: {}",
            existing.status
        )));
    }

    sqlx::query("DELETE FROM missions WHERE id = $1")
        .bind(mission_id)
        .execute(pool)
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_transitions() {
        assert!(validate_transition("created", "assigned").is_ok());
        assert!(validate_transition("created", "cancelled").is_ok());
        assert!(validate_transition("assigned", "executing").is_ok());
        assert!(validate_transition("assigned", "cancelled").is_ok());
        assert!(validate_transition("executing", "completed").is_ok());
        assert!(validate_transition("executing", "failed").is_ok());
        assert!(validate_transition("executing", "cancelled").is_ok());
    }

    #[test]
    fn test_invalid_transitions() {
        assert!(validate_transition("created", "executing").is_err());
        assert!(validate_transition("created", "completed").is_err());
        assert!(validate_transition("created", "failed").is_err());
        assert!(validate_transition("assigned", "assigned").is_err());
        assert!(validate_transition("assigned", "completed").is_err());
        assert!(validate_transition("assigned", "failed").is_err());
        assert!(validate_transition("executing", "assigned").is_err());
        assert!(validate_transition("executing", "created").is_err());
        assert!(validate_transition("completed", "cancelled").is_err());
        assert!(validate_transition("failed", "cancelled").is_err());
        assert!(validate_transition("cancelled", "created").is_err());
    }
}
